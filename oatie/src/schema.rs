//! Performs operational transform.

use std::collections::HashMap;
use std::borrow::ToOwned;
use std::cmp;

use super::doc::*;
use super::stepper::*;
use super::compose;
use super::normalize;
use super::writer::*;

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
            TrackType::ListItems | TrackType::Inlines => true,
            _ => false,
        }
    }

    #[allow(match_same_arms)]
    pub fn parents(&self) -> Vec<TrackType> {
        use self::TrackType::*;
        match *self {
            Lists => vec![ListItems, BlockQuotes],
            ListItems => vec![Lists],
            BlockQuotes => vec![ListItems, BlockQuotes],
            Blocks => vec![ListItems, BlockObjects],
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
            Lists => vec![Lists, ListItems, BlockQuotes],
            ListItems => vec![Lists, ListItems, BlockQuotes],
            BlockQuotes => vec![Lists, ListItems, BlockQuotes],
            Blocks => vec![Lists, ListItems, BlockObjects],
            BlockObjects => vec![Lists, ListItems, BlockQuotes],
            Inlines | InlineObjects => vec![Lists, ListItems, BlockQuotes, Blocks],
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
            "ul" => Some(TrackType::Lists),
            "li" => Some(TrackType::ListItems),
            "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "pre" => Some(TrackType::Blocks),
            "span" | "b" => Some(TrackType::Inlines),
            "cursor" => Some(TrackType::InlineObjects),
            _ => None,
        }
    }
}
