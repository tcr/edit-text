use serde::{
    Deserialize,
    Serialize,
};
use serde::de::Deserializer;
use serde::ser::Serializer;
use std::collections::HashSet;
use enumset::*;
pub use crate::core::schema::*;

// FIXME
use crate::rtf::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpaqueStyleMap(EnumSet<RtfStyle>);

impl OpaqueStyleMap {
    pub fn new() -> Self {
        OpaqueStyleMap(EnumSet::new())
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
        // FIXME
        // Ok(OpaqueStyleMap::from(StyleSet::deserialize(deserializer)?.into_iter().map(|k|
        // (k, None)).collect::<StyleMap>()))
        unimplemented!();
    }
}

pub trait StyleTrait {
    fn styles(&self) -> HashSet<RtfStyle>;
    fn contains(&self, style: RtfStyle) -> bool;
    fn is_empty(&self) -> bool;
    fn extend(&mut self, map: &Self);
    fn remove(&mut self, set: &Self);
}

impl StyleTrait for OpaqueStyleMap {
    fn styles(&self) -> HashSet<RtfStyle> {
        self.0.iter().collect()
    }

    fn contains(&self, style: RtfStyle) -> bool {
        self.0.contains(style)
    }

    fn is_empty(&self) -> bool {
        self.0.iter().count() == 0
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
