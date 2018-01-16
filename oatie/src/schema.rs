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
use std::fmt::Debug;
use super::transform::{TrackType, Tag};

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


#[derive(PartialEq, Copy, Clone, Debug)]
pub enum RtfTrackType {
    NoType,
    Lists,
    ListItems,
    BlockQuotes,
    Blocks,
    BlockObjects,
    Inlines,
    InlineObjects,
}

impl TrackType for RtfTrackType {
    // Rename this do close split? if applicable?
    fn do_split(&self) -> bool {
        match *self {
            RtfTrackType::Lists => false,
            _ => true,
        }
    }

    // Unsure about this naming
    fn do_open_split(&self) -> bool {
        match *self {
            RtfTrackType::Inlines => true,
            _ => false,
        }
    }

    fn supports_text(&self) -> bool {
        match *self {
            RtfTrackType::Blocks | RtfTrackType::Inlines => true,
            _ => false,
        }
    }

    fn allowed_in_root(&self) -> bool {
        match *self {
            RtfTrackType::Blocks | RtfTrackType::ListItems => true,
            _ => false,
        }
    }

    // TODO is this how this should work
    fn is_object(&self) -> bool {
        match *self {
            RtfTrackType::BlockObjects | RtfTrackType::InlineObjects => true,
            _ => false,
        }
    }

    #[allow(match_same_arms)]
    fn parents(&self) -> Vec<Self> {
        use self::RtfTrackType::*;
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
    fn ancestors(&self) -> Vec<Self> {
        use self::RtfTrackType::*;
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
pub struct RtfTag(pub Attrs);

impl Tag for RtfTag {
    type TrackType = RtfTrackType;

    fn to_attrs(&self) -> Attrs {
        self.0.clone()
    }

    fn from_attrs(attrs: &Attrs) -> Self {
        match attrs.get("tag") {
            Some(value) => (),
            None => panic!("expected tag in attrs list: {:?}", attrs),
        }
        RtfTag(attrs.clone())
    }

    fn tag_type(&self) -> Option<Self::TrackType> {
        match &*self.0["tag"] {
            // TODO remove these two
            "ul" => Some(RtfTrackType::Lists),
            "li" => Some(RtfTrackType::ListItems),

            "bullet" => Some(RtfTrackType::ListItems),
            "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "pre" => Some(RtfTrackType::Blocks),
            "span" | "b" => Some(RtfTrackType::Inlines),
            "caret" => Some(RtfTrackType::InlineObjects),
            _ => None,
        }
    }

    fn merge(a: &Self, b: &Self) -> Option<Self> {
        if a.0.get("tag") == b.0.get("tag") && a.0.get("tag").map(|x| x == "span").unwrap_or(false) {
            let c_a: String = a.0.get("class").unwrap_or(&"".to_string()).clone();
            let c_b: String = b.0.get("class").unwrap_or(&"".to_string()).clone();

            let mut c = parse_classes(&c_a);
            c.extend(parse_classes(&c_b));
            Some(RtfTag(hashmap! {
                "tag".to_string() => "span".to_string(),
                "class".to_string() => format_classes(&c),
            }))
        } else {
            None
        }
    }
}
