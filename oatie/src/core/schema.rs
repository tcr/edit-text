//! Performs operational transform.

use super::transform::{
    Schema,
    Track,
};
use std::fmt;
use enumset::EnumSetType;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Attrs {
    Header(u8),
    Text,
    Code,
    Html,
    ListItem,
    Rule,
    Caret {
        client_id: String,
        focus: bool,
    },
}

#[repr(u8)]
#[derive(Debug, Serialize, Deserialize, EnumSetType)]
pub enum Style {
    Normie,   // Sentinel (if this isn't present on a DocString, something went wrong somewhere)
    Selected, // Never used in server, added on client to show selected text
    Bold,
    Italic,
    Link,     // Needs attached link data
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use the Debug implementation for Display.
        fmt::Debug::fmt(self, f)
    }
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
    // TODO Rename this do close split? if applicable? When is this used?
    fn do_split(&self) -> bool {
        match *self {
            _ => true,
        }
    }

    // TODO Unsure about this naming
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

    #[allow(clippy::match_same_arms)]
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
    #[allow(clippy::match_same_arms)]
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

#[derive(Clone, Debug)]
pub struct RtfSchema;

impl Schema for RtfSchema {
    type Track = RtfTrack;

    fn attrs_eq(a: &Attrs, b: &Attrs) -> bool {
        // TODO normalize?
        a == b
    }

    fn merge_attrs(_a: &Attrs, _b: &Attrs) -> Option<Attrs> {
        None
    }

    fn track_type_from_attrs(attrs: &Attrs) -> Option<Self::Track> {
        match attrs {
            Attrs::ListItem => Some(RtfTrack::ListItems),
            Attrs::Text | Attrs::Header(..) | Attrs::Code | Attrs::Html => {
                Some(RtfTrack::Blocks)
            }
            // "span" => Some(RtfTrack::Inlines),
            Attrs::Caret { .. } => Some(RtfTrack::InlineObjects),
            Attrs::Rule => Some(RtfTrack::BlockObjects),
            // _ => None,
        }
    }
}
