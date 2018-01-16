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
use super::transform::{Schema, Track};

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
    NoType,
    Lists,
    ListItems,
    BlockQuotes,
    Blocks,
    BlockObjects,
    Inlines,
    InlineObjects,
}

impl Track for RtfTrack {
    // Rename this do close split? if applicable?
    fn do_split(&self) -> bool {
        match *self {
            RtfTrack::Lists => false,
            _ => true,
        }
    }

    // Unsure about this naming
    fn do_open_split(&self) -> bool {
        match *self {
            RtfTrack::Inlines => true,
            _ => false,
        }
    }

    fn supports_text(&self) -> bool {
        match *self {
            RtfTrack::Blocks | RtfTrack::Inlines => true,
            _ => false,
        }
    }

    fn allowed_in_root(&self) -> bool {
        match *self {
            RtfTrack::Blocks | RtfTrack::ListItems => true,
            _ => false,
        }
    }

    // TODO is this how this should work
    fn is_object(&self) -> bool {
        match *self {
            RtfTrack::BlockObjects | RtfTrack::InlineObjects => true,
            _ => false,
        }
    }

    #[allow(match_same_arms)]
    fn parents(&self) -> Vec<Self> {
        use self::RtfTrack::*;
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
        use self::RtfTrack::*;
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
            // TODO remove these two
            "ul" => Some(RtfTrack::Lists),
            "li" => Some(RtfTrack::ListItems),

            "bullet" => Some(RtfTrack::ListItems),
            "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "pre" => Some(RtfTrack::Blocks),
            "span" | "b" => Some(RtfTrack::Inlines),
            "caret" => Some(RtfTrack::InlineObjects),
            _ => None,
        }
    }
}
