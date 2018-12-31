//! Defines document types, operation types, and cursor types.

use serde::{
    Deserialize,
    Serialize,
};

// Re-exports
pub use super::place::*;
pub use crate::core::schema::*;
pub use crate::string::*;

pub type DocSpan<S> = Vec<DocElement<S>>;
pub type DelSpan<S> = Vec<DelElement<S>>;
pub type AddSpan<S> = Vec<AddElement<S>>;
pub type CurSpan = Vec<CurElement>;

pub type Op<S> = (DelSpan<S>, AddSpan<S>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DocElement<S: Schema> {
    DocText(S::CharsProperties, DocString),
    DocGroup(S::GroupProperties, DocSpan<S>),
}

pub use self::DocElement::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Doc<S: Schema>(pub Vec<DocElement<S>>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DelElement<S: Schema> {
    DelSkip(usize),
    DelWithGroup(DelSpan<S>),
    DelText(usize),
    DelGroup(DelSpan<S>),
    DelStyles(usize, S::CharsProperties),
    // TODO Implement these
    // DelGroupAll,
    // DelMany(usize),
    // DelObject,
}

pub use self::DelElement::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AddElement<S: Schema> {
    AddSkip(usize),
    AddWithGroup(AddSpan<S>),
    AddText(S::CharsProperties, DocString),
    AddGroup(S::GroupProperties, AddSpan<S>),
    AddStyles(usize, S::CharsProperties),
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
