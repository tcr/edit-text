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

    pub fn supports_text(&self) -> bool {
        match *self {
            TrackType::Blocks | TrackType::Inlines => true,
            _ => false,
        }
    }

    pub fn allowed_in_root(&self) -> bool {
        match *self {
            TrackType::Blocks | TrackType::ListItems => true,
            _ => false,
        }
    }

    // TODO is this how this should work
    pub fn is_object(&self) -> bool {
        match *self {
            TrackType::BlockObjects | TrackType::InlineObjects => true,
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

    // TODO extrapolate this from parents()
    #[allow(match_same_arms)]
    pub fn ancestors(&self) -> Vec<TrackType> {
        use self::TrackType::*;
        match *self {
            // Lists => vec![Lists, ListItems, BlockQuotes],
            ListItems => vec![ListItems, BlockQuotes,],
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

    pub fn merge(a: &Tag, b: &Tag) -> Option<Tag> {
        if a.0.get("tag") == b.0.get("tag") && a.0.get("tag").map(|x| x == "span").unwrap_or(false) {
            let c_a: String = a.0.get("class").unwrap_or(&"".to_string()).clone();
            let c_b: String = b.0.get("class").unwrap_or(&"".to_string()).clone();

            let mut c = parse_classes(&c_a);
            c.extend(parse_classes(&c_b));
            Some(Tag(hashmap! {
                "tag".to_string() => "span".to_string(),
                "class".to_string() => format_classes(&c),
            }))
        } else {
            None
        }
    }
}

fn parse_classes(input: &str) -> HashSet<String> {
    input
        .split_whitespace()
        .filter(|x| !x.is_empty())
        .map(|x| x.to_owned())
        .collect()
}

fn format_classes(set: &HashSet<String>) -> String {
    let mut classes: Vec<String> = set.iter().cloned().collect();
    classes.sort();
    classes.join(" ")
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