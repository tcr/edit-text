//! Performs operational transform.

use super::compose;
use super::doc::*;
use super::normalize;
use super::stepper::*;
use super::transform::{
    Schema,
    Track,
};
use super::writer::*;
use failure::Error;
use std::borrow::ToOwned;
use std::cmp;
use std::collections::{
    HashMap,
    HashSet,
};
use std::fmt::Debug;
use term_painter::Attr::*;
use term_painter::Color::*;
use term_painter::ToStyle;

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
pub enum RtfTrack {
    ListItems,     // bullet
    BlockQuotes,   // blockquote
    Blocks,        // h1, h2, h3, h4, h5, h6, p, pre
    BlockObjects,  // hr
    Inlines,       // span
    InlineObjects, // caret
}

impl Track for RtfTrack {
    // Rename this do close split? if applicable?
    fn do_split(&self) -> bool {
        use self::RtfTrack::*;
        match *self {
            _ => true,
        }
    }

    // Unsure about this naming
    fn do_open_split(&self) -> bool {
        use self::RtfTrack::*;
        match *self {
            Inlines => true,
            _ => false,
        }
    }

    fn supports_text(&self) -> bool {
        use self::RtfTrack::*;
        match *self {
            Blocks | Inlines => true,
            _ => false,
        }
    }

    fn allowed_in_root(&self) -> bool {
        use self::RtfTrack::*;
        match *self {
            Blocks | ListItems | BlockObjects => true,
            _ => false,
        }
    }

    // TODO is this how this should work
    fn is_object(&self) -> bool {
        use self::RtfTrack::*;
        match *self {
            BlockObjects | InlineObjects => true,
            _ => false,
        }
    }

    #[allow(match_same_arms)]
    fn parents(&self) -> Vec<Self> {
        use self::RtfTrack::*;
        match *self {
            ListItems => vec![ListItems, BlockQuotes],
            BlockQuotes => vec![ListItems, BlockQuotes],
            Blocks => vec![ListItems, BlockQuotes],
            BlockObjects => vec![ListItems, BlockQuotes],
            Inlines | InlineObjects => vec![Blocks],
        }
    }

    // TODO extrapolate this from parents()
    #[allow(match_same_arms)]
    fn ancestors(&self) -> Vec<Self> {
        use self::RtfTrack::*;
        match *self {
            ListItems => vec![ListItems, BlockQuotes],
            BlockQuotes => vec![ListItems, BlockQuotes],
            Blocks => vec![ListItems, BlockObjects],
            BlockObjects => vec![ListItems, BlockQuotes],
            Inlines | InlineObjects => vec![ListItems, BlockQuotes, Blocks],
        }
    }
}

// TODO why are these attributes necessary?
#[derive(Clone, Debug)]
pub struct RtfSchema;

impl Schema for RtfSchema {
    type Track = RtfTrack;

    fn attrs_eq(a: &Attrs, b: &Attrs) -> bool {
        // TODO normalize?
        a == b
    }

    fn merge_attrs(a: &Attrs, b: &Attrs) -> Option<Attrs> {
        if a.get("tag") == b.get("tag") && a.get("tag").map(|x| x == "span").unwrap_or(false) {
            let c_a: String = a.get("class").unwrap_or(&"".to_string()).clone();
            let c_b: String = b.get("class").unwrap_or(&"".to_string()).clone();

            let mut c = parse_classes(&c_a);
            c.extend(parse_classes(&c_b));
            Some(hashmap! {
                "tag".to_string() => "span".to_string(),
                "class".to_string() => format_classes(&c),
            })
        } else {
            None
        }
    }

    fn track_type_from_attrs(attrs: &Attrs) -> Option<Self::Track> {
        match &*attrs["tag"] {
            "bullet" => Some(RtfTrack::ListItems),
            "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "pre" | "html" => {
                Some(RtfTrack::Blocks)
            }
            "span" => Some(RtfTrack::Inlines),
            "caret" => Some(RtfTrack::InlineObjects),
            "hr" => Some(RtfTrack::BlockObjects),
            _ => None,
        }
    }
}
