use super::compose;
use super::doc::*;
use super::normalize;
use super::schema::*;
use super::stepper::*;
use super::writer::*;
use failure::Error;
use std::borrow::ToOwned;
use std::cmp;
use std::collections::{HashMap, HashSet};
use term_painter::Attr::*;
use term_painter::Color::*;
use term_painter::ToStyle;

#[derive(Clone)]
pub struct ValidateContext {
    stack: Vec<Attrs>,
    carets: HashSet<String>,
}

impl ValidateContext {
    pub fn new() -> ValidateContext {
        ValidateContext {
            stack: vec![],
            carets: hashset![],
        }
    }
}

// TODO caret-specific validation should be moved out to the schema!
pub fn validate_doc_span(ctx: &mut ValidateContext, span: &DocSpan) -> Result<(), Error> {
    for elem in span {
        match *elem {
            DocGroup(ref attrs, ref span) => {
                if attrs["tag"] == "caret" {
                    if !ctx.carets.insert(attrs["client"].clone()) {
                        bail!("Multiple carets for {:?} exist", attrs["client"]);
                    }
                }

                ctx.stack.push(attrs.clone());
                validate_doc_span(ctx, span)?;
                ctx.stack.pop();

                // Check parentage.
                if let Some(parent) = ctx.stack.last() {
                    let parent_type = Tag(parent.clone()).tag_type().unwrap();
                    let cur_type = Tag(attrs.clone()).tag_type().unwrap();
                    ensure!(
                        cur_type.parents().contains(&parent_type),
                        "Block has incorrect parent"
                    );
                } else {
                    // Top-level blocks
                    ensure!(
                        Tag(attrs.clone()).tag_type().unwrap().allowed_in_root(),
                        "Root block has incorrect parent"
                    );
                }
            }
            DocChars(ref text) => {
                ensure!(text.len() > 0, "Empty char string");

                if let Some(block) = ctx.stack.last() {
                    ensure!(
                        Tag(block.clone()).tag_type().unwrap().allowed_in_root(),
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
