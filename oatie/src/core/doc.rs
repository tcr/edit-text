//! Defines document types, operation types, and cursor types.

use serde::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use crate::style::OpaqueStyleMap;

// Re-exports
pub use super::place::*;
pub use crate::string::*;

pub type Attrs = HashMap<String, String>;

pub type DocSpan = Vec<DocElement>;
pub type DelSpan = Vec<DelElement>;
pub type AddSpan = Vec<AddElement>;
pub type CurSpan = Vec<CurElement>;

pub type Op = (DelSpan, AddSpan);

fn style_map_is_empty(map: &OpaqueStyleMap) -> bool {
    map.iter().count() == 0
}

// fn docchars_deserialize<'de, D>(mut d: D) -> Result<(DocString, OpaqueStyleMap), D::Error>
//     where D: Deserializer<'de> + Clone
// {
//     <(DocString, OpaqueStyleMap) as Deserialize>::deserialize(d)
//         .or_else(|_| <DocString as Deserialize>::deserialize(d).map(|string| (string, OpaqueStyleMap::new())))
// }

use serde::de::Deserializer;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DocElement {
    // #[serde(deserialize_with = "docchars_deserialize")]
    DocChars(DocString, #[serde(skip_serializing_if = "style_map_is_empty", skip_deserializing)] OpaqueStyleMap),
    DocGroup(Attrs, DocSpan),
}

pub use self::DocElement::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Doc(pub Vec<DocElement>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DelElement {
    DelSkip(usize),
    DelWithGroup(DelSpan),
    DelChars(usize),
    DelGroup(DelSpan),
    DelStyles(usize, StyleSet),
    // TODO Implement these
    // DelGroupAll,
    // DelMany(usize),
    // DelObject,
}

pub use self::DelElement::*;


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AddElement {
    AddSkip(usize),
    AddWithGroup(AddSpan),
    // #[serde(deserialize_with = "docchars_deserialize")]
    AddChars(DocString, #[serde(skip_serializing_if = "style_map_is_empty", skip_deserializing)] OpaqueStyleMap),
    AddGroup(Attrs, AddSpan),
    AddStyles(usize, StyleMap),
}

pub use self::AddElement::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CurElement {
    CurSkip(usize),
    CurWithGroup(CurSpan),
    CurGroup,
    CurChar,
}

pub use self::CurElement::*;
