//! Performs operational transform.

use std::collections::{HashMap, HashSet};
use std::borrow::ToOwned;
use std::cmp;
use super::doc::*;
use super::stepper::*;
use super::compose;
use super::normalize;
use super::writer::*;
use failure::Error;
use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TrackType {
    NoType,
    Lists,
    ListItems,
    BlockQuotes,
    Blocks,
    BlockObjects,
    Inlines,
    InlineObjects,
}

// TODO eventually, all this can be declarative (or at least moreso)
impl TrackType {
    // Rename this do close split? if applicable?
    pub fn do_split(&self) -> bool {
        match *self {
            TrackType::Lists => false,
            _ => true,
        }
    }

    // Unsure about this naming
    pub fn do_open_split(&self) -> bool {
        match *self {
            TrackType::Inlines => true,
            _ => false,
        }
    }

    #[allow(match_same_arms)]
    pub fn parents(&self) -> Vec<TrackType> {
        use self::TrackType::*;
        match *self {
            // Lists => vec![ListItems, BlockQuotes],
            ListItems => vec![ListItems, BlockQuotes],
            BlockQuotes => vec![ListItems, BlockQuotes],
            Blocks => vec![ListItems, BlockQuotes],
            BlockObjects => vec![ListItems, BlockQuotes],
            Inlines | InlineObjects => vec![Blocks],
            _ => {
                panic!("this shouldnt be");
            }
        }
    }

    #[allow(match_same_arms)]
    pub fn ancestors(&self) -> Vec<TrackType> {
        use self::TrackType::*;
        match *self {
            // Lists => vec![Lists, ListItems, BlockQuotes],
            ListItems => vec![ListItems, BlockQuotes],
            BlockQuotes => vec![ListItems, BlockQuotes],
            Blocks => vec![ListItems, BlockObjects],
            BlockObjects => vec![ListItems, BlockQuotes],
            Inlines | InlineObjects => vec![ListItems, BlockQuotes, Blocks],
            _ => {
                panic!("this shouldnt be");
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Tag(pub Attrs);

impl Tag {
    pub fn to_attrs(&self) -> Attrs {
        self.0.clone()
    }

    pub fn from_attrs(attrs: &Attrs) -> Tag {
        match attrs.get("tag") {
            Some(value) => (),
            None => panic!("expected tag in attrs list: {:?}", attrs),
        }
        Tag(attrs.clone())
    }

    pub fn tag_type(self: &Tag) -> Option<TrackType> {
        match &*self.0["tag"] {
            // TODO remove these two
            "ul" => Some(TrackType::Lists),
            "li" => Some(TrackType::ListItems),

            "bullet" => Some(TrackType::ListItems),
            "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "pre" => Some(TrackType::Blocks),
            "span" | "b" => Some(TrackType::Inlines),
            "caret" => Some(TrackType::InlineObjects),
            _ => None,
        }
    }
}


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
                        Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) ||
                        Tag(attrs.clone()).tag_type() == Some(TrackType::ListItems),
                        "Root block has incorrect parent"
                    );

                }
            }
            DocChars(ref text) => {
                ensure!(text.len() > 0, "Empty char string");

                if let Some(block) = ctx.stack.last() {
                    ensure!(
                        Tag(block.clone()).tag_type() == Some(TrackType::Blocks) ||
                        Tag(block.clone()).tag_type() == Some(TrackType::Inlines),
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