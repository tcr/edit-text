//! Defines document types, operation types, and cursor types.

use serde::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

// Re-exports
pub use super::place::*;
pub use crate::string::*;

pub type Attrs = HashMap<String, String>;

pub type DocSpan = Vec<DocElement>;
pub type DelSpan = Vec<DelElement>;
pub type AddSpan = Vec<AddElement>;
pub type CurSpan = Vec<CurElement>;

pub type Op = (DelSpan, AddSpan);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DocElement {
    DocChars(DocString),
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
    AddChars(DocString),
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
