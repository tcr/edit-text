use serde::{
    Deserialize,
    Serialize,
};
use serde::de::Deserializer;
use serde::ser::Serializer;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use enumset::*;

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

pub type StyleMap = HashMap<Style, Option<String>>;
pub type StyleSet = HashSet<Style>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpaqueStyleMap(EnumSet<Style>, Option<Arc<String>>);

impl Default for OpaqueStyleMap {
    fn default() -> Self {
        OpaqueStyleMap::new()
    }
}

impl Serialize for OpaqueStyleMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.styles().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for OpaqueStyleMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(OpaqueStyleMap::from(StyleSet::deserialize(deserializer)?.into_iter().map(|k| (k, None)).collect()))
    }
}

impl OpaqueStyleMap {
    pub fn new() -> Self {
        OpaqueStyleMap(EnumSet::new(), None)
    }

    pub fn from(mut map: StyleMap) -> Self {
        let link = map.remove_entry(&Style::Link)
            .map(|(_k, link)| {
                Arc::new(link.unwrap())
            });
        let mut set = EnumSet::new();
        map.keys().for_each(|k| { set.insert(*k); });
        OpaqueStyleMap(set, link)
    }

    pub fn styles(&self) -> StyleSet {
        self.0.iter().collect()
    }

    pub fn contains(&self, style: Style) -> bool {
        self.0.contains(style)
    }

    pub fn to_map(&self) -> StyleMap {
        let mut hashmap: StyleMap = self.0.iter().map(|s| (s.to_owned(), None)).collect();
        if let Some(ref string) = self.1 {
            hashmap.insert(Style::Link, Some((*string).to_string()));
        }
        hashmap
    }

    pub fn iter(&self) -> impl Iterator<Item=(Style, Option<String>)> {
        // TODO OpaqueStyleMap::iter needs to support Link values (self.1)
        self.0.iter().map(|k| (k, None))
    }

    pub fn extend(&mut self, map: &StyleMap) {
        for (k, v) in map {
            self.0.insert(*k);
            if *k == Style::Link {
                self.1 = v.to_owned().map(|s| Arc::new(s));
            }
        }
    }

    pub fn remove(&mut self, set: &StyleSet) {
        for item in set {
            self.0.remove(*item);
            if item == &Style::Link {
                self.1 = None;
            }
        }
    }
}
