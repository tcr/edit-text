//! Validates a document matches a given Schema.

use super::doc::*;
use super::schema::{
    Schema,
    Track,
};
use failure::Error;
use std::collections::HashSet;

#[derive(Clone)]
pub struct ValidateContext<S: Schema> {
    stack: Vec<S::GroupProperties>,
    carets: HashSet<String>,
}

impl<S: Schema> ValidateContext<S> {
    pub fn new() -> ValidateContext<S> {
        ValidateContext {
            stack: vec![],
            carets: hashset![],
        }
    }
}

// TODO caret-specific validation should be moved out to the schema!
pub fn validate_doc_span<S: Schema>(
    ctx: &mut ValidateContext<S>,
    span: &DocSpan<S>,
) -> Result<(), Error> {
    for elem in span {
        match *elem {
            DocGroup(ref attrs, ref span) => {
                // if let Attrs::Caret { .. } == attrs {
                // TODO Allow validation that only one caret exists per document.
                // if !ctx.carets.insert(attrs["client"].clone()) {
                //     bail!("Multiple carets for {:?} exist", attrs["client"]);
                // }
                // }

                // TODO This is disabled with the removal of attribute
                // introspection, but should be re-instated
                // if let Attrs::ListItem = attrs {
                //     ensure!(!span.is_empty(), "Expected non-empty bullet");
                // }

                ctx.stack.push(attrs.clone());
                validate_doc_span(ctx, span)?;
                ctx.stack.pop();

                // Check parentage.
                if let Some(parent) = ctx.stack.last() {
                    let parent_type = S::track_type_from_attrs(parent).unwrap();
                    let cur_type = S::track_type_from_attrs(attrs).unwrap();
                    ensure!(
                        cur_type.parents().contains(&parent_type),
                        "Block has incorrect parent"
                    );
                } else {
                    // Top-level blocks
                    ensure!(
                        S::track_type_from_attrs(attrs).unwrap().allowed_in_root(),
                        "Root block has incorrect parent"
                    );
                }
            }
            DocText(ref _styles, ref text) => {
                ensure!(text.char_len() > 0, "Empty char string");

                if let Some(block) = ctx.stack.last() {
                    ensure!(
                        S::track_type_from_attrs(block).unwrap().supports_text(),
                        "Char found outside block"
                    );
                } else {
                    bail!("Found char in root");
                }
            }
        }
    }
    Ok(())
}

pub fn validate_doc<S: Schema>(doc: &Doc<S>) -> Result<(), Error> {
    let mut ctx = ValidateContext::new();
    validate_doc_span(&mut ctx, &doc.0)
}
