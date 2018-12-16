#![feature(custom_attribute, nll)]

#[macro_use]
extern crate maplit;

#[macro_use]
extern crate serde_derive;

use serde_json;

#[macro_use]
extern crate wasm_typescript_definition;

pub mod commands;
pub mod markdown;
#[cfg(not(target_arch = "wasm32"))]
pub mod simple_ws;

use htmlescape::encode_minimal;
use oatie::doc::*;
use std::collections::{
    HashMap,
    HashSet,
};

type CaretIndex = HashMap<String, usize>;
type SelectionActive = HashSet<String>;

// TODO unify with its counterpart in edit-client/src/walkers.rs?
fn is_caret(attrs: &Attrs, client_id: Option<&str>) -> bool {
    if let Attrs::Caret { client_id: caret_client_id, .. } = attrs {
        client_id.is_none() || caret_client_id == client_id.unwrap()
    } else {
        false
    }
    // && attrs.get("focus").unwrap_or(&"false".to_string()).parse::<bool>().map(|x| x == focus).unwrap_or(false)
}

fn html_start_tag(tag: &str, attrs: HashMap<String, String>) -> String {
    format!("<{} {}>", tag, attrs.into_iter().map(|(k, v)| {
        format!("{}={}", k, serde_json::to_string(&v).unwrap_or("".to_string()))
    }).collect::<Vec<String>>().join(" "))
}

// TODO move this to a different module
/// Converts a DocSpan to an HTML string.
pub fn doc_as_html(doc: &DocSpan) -> String {
    // Count all carets in tree.
    let mut caret_index: CaretIndex = HashMap::new();
    let mut stepper = ::oatie::stepper::DocStepper::new(doc);
    loop {
        match stepper.head() {
            Some(DocGroup(attrs, _)) => {
                if let Attrs::Caret { ref client_id, .. } = attrs {
                    *caret_index.entry(client_id.to_owned()).or_insert(0) += 1;
                }
                stepper.enter();
            }
            Some(DocChars(ref text, _)) => {
                stepper.skip(text.char_len());
            }
            None => {
                if stepper.is_done() {
                    break;
                } else {
                    stepper.exit();
                }
            }
        }
    }

    let mut remote_select_active = hashset![];
    doc_as_html_inner(doc, &caret_index, &mut remote_select_active)
}

pub fn doc_as_html_inner(
    doc: &DocSpan,
    caret_index: &CaretIndex,
    remote_select_active: &mut SelectionActive,
) -> String {
    use oatie::doc::*;

    // let mut select_active = false;
    let mut out = String::new();
    for elem in doc {
        match elem {
            &DocGroup(ref attrs, ref span) => {
                out.push_str(&match attrs {
                    Attrs::Text => html_start_tag("div", hashmap!{ "data-tag".into() => "p".into() }),
                    Attrs::Code => html_start_tag("div", hashmap!{ "data-tag".into() => "pre".into() }),
                    Attrs::Html => html_start_tag("div", hashmap!{ "data-tag".into() => "html".into() }),
                    Attrs::Header(level) => {
                        html_start_tag("div", hashmap!{ "data-tag".into() => format!("h{}", level) })
                    },
                    Attrs::ListItem => html_start_tag("div", hashmap!{ "data-tag".into() => "bullet".into() }),
                    Attrs::Rule => html_start_tag("div", hashmap!{ "data-tag".into() => "hr".into() }),
                    Attrs::Caret { ref client_id, ref focus } => {
                        html_start_tag("div", hashmap!{
                            "data-tag".into() => "caret".to_string(),
                            "data-client".into() => client_id.to_string(),
                            "data-focus".into() => if *focus { "true".into() } else { "false".into() },
                            "data-anchor".into() => if !*focus { "true".into() } else { "false".into() },
                        })
                    },
                });

                if let Attrs::Caret { client_id, .. } = attrs {
                    if caret_index[client_id] == 2 {
                        // Toggle this ID.
                        if !remote_select_active.insert(client_id.clone()) {
                            remote_select_active.remove(&client_id.clone());
                        }
                    }
                }

                out.push_str(&doc_as_html_inner(span, caret_index, remote_select_active));
                out.push_str(r"</div>");
            }
            &DocChars(ref text, ref styles) => {
                let mut classes = styles.styles();
                if !remote_select_active.is_empty() {
                    classes.insert(Style::Selected);
                }

                out.push_str(&format!(
                    r#"<span class="{}" {}>"#,
                    classes
                        .into_iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                    styles
                        .iter()
                        .filter(|(_, v)| v.is_some())
                        .map(|(k, v)| format!(
                            "data-style-{k}={v}",
                            k = k,
                            v = serde_json::to_string(&v).unwrap()
                        ))
                        .collect::<Vec<String>>()
                        .join(" "),
                ));
                out.push_str(&encode_minimal(text.as_str()));
                out.push_str(r"</span>");
            }
        }
    }
    out
}
