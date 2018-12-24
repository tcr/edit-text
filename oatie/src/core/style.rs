use serde::{
    Deserialize,
    Serialize,
};
use serde::de::Deserializer;
use serde::ser::Serializer;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use enumset::*;
pub use crate::core::schema::*;

pub type StyleMap = HashMap<Style, Option<String>>;
pub type StyleSet = HashSet<Style>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpaqueStyleMap(EnumSet<Style>, Option<Arc<String>>);

pub trait IntoStyle {
    fn into_style(&self) -> OpaqueStyleMap;
}

impl IntoStyle for OpaqueStyleMap {
    fn into_style(&self) -> OpaqueStyleMap {
        self.clone()
    }
}

impl IntoStyle for HashSet<Style> {
    fn into_style(&self) -> OpaqueStyleMap {
        let mut set = EnumSet::new();
        self.iter().for_each(|k| { set.insert(*k); });
        OpaqueStyleMap(set, None)
    }
}

impl IntoStyle for HashMap<Style, Option<String>> {
    fn into_style(&self) -> OpaqueStyleMap {
        let mut map = self.clone();
        let link = map.remove_entry(&Style::Link)
            .map(|(_k, link)| {
                Arc::new(link.unwrap())
            });
        let mut set = EnumSet::new();
        map.keys().for_each(|k| { set.insert(*k); });
        OpaqueStyleMap(set, link)
    }
}

impl OpaqueStyleMap {
    pub fn new() -> Self {
        OpaqueStyleMap(EnumSet::new(), None)
    }

    pub fn from<I: IntoStyle>(map: I) -> Self {
        map.into_style()
    }

    pub fn iter(&self) -> impl Iterator<Item=(Style, Option<String>)> {
        // TODO OpaqueStyleMap::iter needs to support Link values (self.1)
        self.0.iter().map(|k| (k, None))
    }
}

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
        Ok(OpaqueStyleMap::from(StyleSet::deserialize(deserializer)?.into_iter().map(|k| (k, None)).collect::<StyleMap>()))
    }
}

pub trait StyleTrait {
    // fn new() -> Self;
    // fn from(mut map: StyleMap) -> Self;
    fn styles(&self) -> StyleSet;
    fn contains(&self, style: Style) -> bool;
    fn to_map(&self) -> StyleMap;
    // fn iter(&self) -> impl Iterator<Item=(Style, Option<String>)>;
    fn is_empty(&self) -> bool;
    fn extend(&mut self, map: &Self);
    fn remove(&mut self, set: &Self);
}

impl StyleTrait for OpaqueStyleMap {
    fn styles(&self) -> StyleSet {
        self.0.iter().collect()
    }

    fn contains(&self, style: Style) -> bool {
        self.0.contains(style)
    }

    fn to_map(&self) -> StyleMap {
        let mut hashmap: StyleMap = self.0.iter().map(|s| (s.to_owned(), None)).collect();
        if let Some(ref string) = self.1 {
            hashmap.insert(Style::Link, Some((*string).to_string()));
        }
        hashmap
    }

    fn is_empty(&self) -> bool {
        self.iter().count() == 0
    }

    fn extend(&mut self, map: &Self) {
        // for (k, v) in map {
        //     self.0.insert(*k);
        //     if *k == Style::Link {
        //         self.1 = v.to_owned().map(|s| Arc::new(s));
        //     }
        // }
        // FIXME
        unimplemented!();
    }

    fn remove(&mut self, set: &Self) {
        // for item in set {
        //     self.0.remove(*item);
        //     if item == &Style::Link {
        //         self.1 = None;
        //     }
        // }
        // FIXME
        unimplemented!();
    }
}
