use failure::Error;
use oatie::doc::*;

fn remove_carets_span(span: &DocSpan) -> Result<DocSpan, Error> {
    let mut ret: DocSpan = vec![];

    for elem in span {
        match *elem {
            DocGroup(ref attrs, ref span) => {
                if attrs["tag"] != "caret" {
                    let res = remove_carets_span(span)?;
                    ret.place(&DocGroup(attrs.clone(), res));
                }
            }
            DocChars(ref text) => {
                ret.place(&DocChars(text.clone()));
            }
        }
    }
    Ok(ret)
}

// Removes carets from docs
pub fn remove_carets(doc: &Doc) -> Result<Doc, Error> {
    Ok(Doc(remove_carets_span(&doc.0)?))
}
