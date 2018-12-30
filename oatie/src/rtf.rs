//! Contains types that define the different groups and styles the document
//! tree can contain. This code should live outside of `oatie`.

use super::schema::*;
use std::fmt;
use std::collections::HashSet;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Attrs {
    Header(u8),
    Para,
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
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub enum RtfStyle {
    Bold,
    Italic,
}

// impl Hash for RtfStyle {
// }

// impl Eq for RtfStyle {
// }

impl fmt::Display for RtfStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use the Debug implementation for Display.
        fmt::Debug::fmt(self, f)
    }
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleSet(HashSet<RtfStyle>);

impl StyleSet {
    pub fn new() -> Self {
        StyleSet(HashSet::new())
    }

    pub fn from(set: HashSet<RtfStyle>) -> Self {
        StyleSet(set)
    }

    pub fn insert(&mut self, style: RtfStyle) {
        self.0.insert(style);
    }

    pub fn remove(&mut self, style: &RtfStyle) {
        self.0.remove(style);
    }

    pub fn contains(&self, style: &RtfStyle) -> bool {
        self.0.contains(style)
    }
}

impl Default for StyleSet {
    fn default() -> Self {
        StyleSet::new()
    }
}

impl Serialize for StyleSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.styles().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StyleSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(StyleSet(<HashSet<RtfStyle> as Deserialize>::deserialize(deserializer)?))
    }
}


impl StyleTrait for StyleSet {
    type Style = RtfStyle;

    fn styles(&self) -> HashSet<Self::Style> {
        self.0.clone()
    }

    fn is_empty(&self) -> bool {
        self.0.iter().count() == 0
    }

    fn extend(&mut self, set: &Self) {
        for item in set.styles() {
            self.0.insert(item);
        }
    }

    fn remove(&mut self, set: &Self) {
        for item in set.styles() {
            self.0.remove(&item);
        }
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RtfSchema;

impl Schema for RtfSchema {
    type Track = RtfTrack;

    type GroupProperties = Attrs;
    type CharsProperties = StyleSet;

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
            Attrs::Para | Attrs::Header(..) | Attrs::Code | Attrs::Html => {
                Some(RtfTrack::Blocks)
            }
            // "span" => Some(RtfTrack::Inlines),
            Attrs::Caret { .. } => Some(RtfTrack::InlineObjects),
            Attrs::Rule => Some(RtfTrack::BlockObjects),
            // _ => None,
        }
    }
}
