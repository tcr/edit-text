use failure::Error;
use oatie::doc::*;
use oatie::writer::*;

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

/// Removes carets from a doc.
// TODO maybe just merge this with the below function
pub fn remove_carets(doc: &Doc) -> Result<Doc, Error> {
    Ok(Doc(remove_carets_span(&doc.0)?))
}


fn remove_carets_op_span(writer: &mut DelWriter, span: &DocSpan, filter: &[String]) -> Result<(), Error> {
    let mut ret: DocSpan = vec![];

    for elem in span {
        match *elem {
            DocGroup(ref attrs, ref span) => {
                if attrs["tag"] == "caret" && filter.contains(&attrs["client"]) {
                    assert!(span.is_empty());
                    writer.begin();
                    writer.close();
                } else {
                    writer.begin();
                    remove_carets_op_span(writer, span, filter)?;
                    writer.exit();
                }
            }
            DocChars(ref text) => {
                writer.place(&DelSkip(text.char_len()));
            }
        }
    }
    Ok(())
}

/// Removes carets from a doc. Filter contains the client IDs to remove.
pub fn remove_carets_op(doc: &Doc, filter: Vec<String>) -> Result<Op, Error> {
    let mut writer = DelWriter::new();
    remove_carets_op_span(&mut writer, &doc.0, &filter)?;
    Ok((writer.result(), vec![]))
}
