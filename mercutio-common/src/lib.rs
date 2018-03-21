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
extern crate serde_json;
extern crate take_mut;
extern crate lazy_static;
extern crate ron;
extern crate colored;
extern crate htmlescape;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;

pub mod markdown;

use htmlescape::encode_minimal;
use oatie::doc::*;


// TODO move the below to a file

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncServerCommand {
    // Connect(String),
    Keepalive,
    Commit(String, Op, usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncClientCommand {
    // Client id assignment, initial doc, initial version
    Init(String, DocSpan, usize),

    // New document, version, client-id, operation
    Update(DocSpan, usize, String, Op),
}


// TODO move this to a different module
/// Converts a DocSpan to an HTML string.
pub fn doc_as_html(doc: &DocSpan, current_client_id: Option<&str>) -> String {
    use oatie::doc::*;

    let mut out = String::new();
    for elem in doc {
        match elem {
            &DocGroup(ref attrs, ref span) => {
                let attr_client = attrs.get("client").unwrap_or(&"".to_string()).to_owned();
                let mut attr_class = attrs.get("class").unwrap_or(&"".to_string()).to_owned();

                if attr_client == current_client_id.unwrap_or("") {
                    // TODO: the client should be doing all the work to identify its own client by CSS
                    attr_class += " current";
                }

                out.push_str(&format!(
                    r#"<div
                        data-tag={}
                        data-client={}
                        class={}
                    >"#,
                    serde_json::to_string(attrs.get("tag").unwrap_or(&"".to_string())).unwrap(),
                    serde_json::to_string(&attr_client).unwrap(),
                    serde_json::to_string(&attr_class).unwrap(),
                ));
                out.push_str(&doc_as_html(span, current_client_id));
                out.push_str(r"</div>");
            }
            &DocChars(ref text) => {
                // out.push_str(r"<span>");
                out.push_str(&encode_minimal(text.as_str()));
                // out.push_str(r"</span>");
            },
        }
    }
    out
}
