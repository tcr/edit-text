#![feature(crate_in_paths)]

extern crate failure;
#[macro_use]
extern crate maplit;
extern crate oatie;
extern crate rand;
extern crate serde;
extern crate taken;
#[macro_use]
extern crate serde_derive;
extern crate colored;
extern crate htmlescape;
extern crate lazy_static;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;
extern crate ron;
extern crate serde_json;
extern crate take_mut;

pub mod commands;
pub mod markdown;

use htmlescape::encode_minimal;
use oatie::doc::*;

// TODO move this to a different module
/// Converts a DocSpan to an HTML string.
pub fn doc_as_html(doc: &DocSpan) -> String {
    let (res, res_alt) = doc_as_html_inner(doc, false);
    if res_alt {
        // Disable the carets
        res.replace(r#"<span class="selected">"#, "<span>")
    } else {
        res
    }
}

pub fn doc_as_html_inner(doc: &DocSpan, mut alt: bool) -> (String, bool) {
    use oatie::doc::*;

    let mut out = String::new();
    for elem in doc {
        match elem {
            &DocGroup(ref attrs, ref span) => {
                out.push_str(&format!(
                    r#"<div
                        data-tag={}
                        data-client={}
                        data-anchor={}
                        class={}
                    >"#,
                    serde_json::to_string(attrs.get("tag").unwrap_or(&"".to_string())).unwrap(),
                    serde_json::to_string(attrs.get("client").unwrap_or(&"".to_string())).unwrap(),
                    serde_json::to_string(attrs.get("anchor").unwrap_or(&"".to_string())).unwrap(),
                    serde_json::to_string(attrs.get("class").unwrap_or(&"".to_string())).unwrap(),
                ));
                if attrs.get("tag") == Some(&"caret".to_string()) {
                    alt = !alt;
                }
                let (inner, new_alt) = doc_as_html_inner(span, alt);
                alt = new_alt;
                out.push_str(&inner);
                out.push_str(r"</div>");
            }
            &DocChars(ref text) => {
                // TODO selected...
                // if alt {
                //     out.push_str(r#"<span class="selected">"#);
                // } else
                if let &Some(ref v) = &text.2 {
                    out.push_str(&format!(r#"<span class="{}">"#, v));
                } else {
                    out.push_str(r"<span>");
                }
                out.push_str(&encode_minimal(text.as_str()));
                out.push_str(r"</span>");
            }
        }
    }
    (out, alt)
}
