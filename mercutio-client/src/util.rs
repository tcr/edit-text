use oatie::doc::*;
use serde_json;

/// Converts a DocSpan to an HTML string.
pub fn doc_as_html(doc: &DocSpan) -> String {
    let mut out = String::new();
    for elem in doc {
        match elem {
            &DocGroup(ref attrs, ref span) => {
                out.push_str(&format!(
                    r#"<div
                        data-tag={}
                        data-client={}
                        class={}
                    >"#,
                    serde_json::to_string(attrs.get("tag").unwrap_or(&"".to_string())).unwrap(),
                    serde_json::to_string(attrs.get("client").unwrap_or(&"".to_string())).unwrap(),
                    serde_json::to_string(attrs.get("class").unwrap_or(&"".to_string())).unwrap(),
                ));
                out.push_str(&doc_as_html(span));
                out.push_str(r"</div>");
            }
            &DocChars(ref text) => for c in text.as_str().chars() {
                // out.push_str(r"<span>");
                out.push(c);
                // out.push_str(r"</span>");
            },
        }
    }
    out
}
