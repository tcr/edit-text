#![feature(crate_in_paths, nll)]

extern crate failure;
#[macro_use]
extern crate maplit;
extern crate oatie;
extern crate rand;
extern crate regex;
extern crate serde;
extern crate taken;
#[macro_use]
extern crate serde_derive;
extern crate colored;
extern crate htmlescape;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;
extern crate ron;
extern crate serde_json;
extern crate take_mut;
#[cfg(not(target_arch = "wasm32"))]
extern crate ws;

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
    attrs["tag"] == "caret" && client_id.map(|id| attrs["client"] == id).unwrap_or(true)
    // && attrs.get("focus").unwrap_or(&"false".to_string()).parse::<bool>().map(|x| x == focus).unwrap_or(false)
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
                if is_caret(&attrs, None) {
                    *caret_index.entry(attrs["client"].to_owned()).or_insert(0) += 1;
                }
                stepper.enter();
            }
            Some(DocChars(ref text)) => {
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
                out.push_str(&format!(
                    r#"<div
                        data-tag={}
                        data-client={}
                        data-anchor={}
                        data-focus={}
                        class={}
                    >"#,
                    serde_json::to_string(attrs.get("tag").unwrap_or(&"".to_string())).unwrap(),
                    serde_json::to_string(attrs.get("client").unwrap_or(&"".to_string())).unwrap(),
                    serde_json::to_string(attrs.get("anchor").unwrap_or(&"".to_string())).unwrap(),
                    serde_json::to_string(attrs.get("focus").unwrap_or(&"".to_string())).unwrap(),
                    serde_json::to_string(attrs.get("class").unwrap_or(&"".to_string())).unwrap(),
                ));

                if attrs.get("tag") == Some(&"caret".to_string()) {
                    if let Some(client_id) = attrs.get("client") {
                        if caret_index[client_id] == 2 {
                            // Toggle this ID.
                            if !remote_select_active.insert(client_id.clone()) {
                                remote_select_active.remove(&client_id.clone());
                            }
                        }
                    }
                }

                out.push_str(&doc_as_html_inner(span, caret_index, remote_select_active));
                out.push_str(r"</div>");
            }
            &DocChars(ref text) => {
                if let &Some(ref styles) = &text.styles() {
                    let mut classes = styles.keys().cloned().collect::<Vec<Style>>();
                    // TODO Style::Selected could be selected here directly
                    if !remote_select_active.is_empty() {
                        classes.push(Style::Selected);
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
                } else {
                    out.push_str(r"<span>");
                }
                out.push_str(&encode_minimal(text.as_str()));
                out.push_str(r"</span>");
            }
        }
    }
    out
}
