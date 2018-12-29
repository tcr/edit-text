#![feature(custom_attribute, nll)]

#[macro_use]
extern crate maplit;
#[macro_use]
extern crate serde_derive;
#[allow(unused)]
#[macro_use]
extern crate wasm_typescript_definition;

pub mod commands;
pub mod markdown;
#[cfg(not(target_arch = "wasm32"))]
pub mod simple_ws;

use serde_json;
use htmlescape::encode_minimal;
use oatie::doc::*;
use std::collections::{HashMap};
use oatie::rtf::*;

fn html_start_tag(tag: &str, attrs: HashMap<String, String>) -> String {
    format!("<{} {}>", tag, attrs.into_iter().map(|(k, v)| {
        format!("{}={}", k, serde_json::to_string(&v).unwrap_or("".to_string()))
    }).collect::<Vec<String>>().join(" "))
}

/// Converts a DocSpan to an HTML string.
pub fn doc_as_html(
    doc: &DocSpan<RtfSchema>,
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

                out.push_str(&doc_as_html(span));
                out.push_str(r"</div>");
            }
            &DocText(ref styles, ref text) => {
                let classes = styles.styles();

                out.push_str(&format!(
                    r#"<span class="{}" {}>"#,
                    classes
                        .into_iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                    // FIXME
                    // styles
                    //     .styles()
                    //     .iter()
                    //     .map(|(k, v)| format!(
                    //         "data-style-{k}={v}",
                    //         k = k,
                    //         v = serde_json::to_string(&v).unwrap()
                    //     ))
                    //     .collect::<Vec<String>>()
                    //     .join(" "),
                    "",
                ));
                out.push_str(&encode_minimal(text.as_str()));
                out.push_str(r"</span>");
            }
        }
    }
    out
}
