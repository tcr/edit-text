//! Defines document types, operation types, and cursor types.

use serde::{
    Deserialize,
    Serialize,
};
use crate::ot::*;
use crate::apply::*;
use crate::compose::*;
use crate::transform::transform;

// Re-exports
pub use super::place::*;
pub use crate::core::schema::*;
pub use crate::string::*;

pub type DocSpan<S> = Vec<DocElement<S>>;
pub type DelSpan<S> = Vec<DelElement<S>>;
pub type AddSpan<S> = Vec<AddElement<S>>;
pub type CurSpan = Vec<CurElement>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Op<S: Schema>(pub DelSpan<S>, pub AddSpan<S>);

impl<S: Schema> OT<S> for Op<S> {
    type Doc = Doc<S>;

    fn apply(doc: &Self::Doc, op: &Self) -> Self::Doc {
        Doc(apply_operation(&doc.0, op))
    }

    fn empty() -> Self {
        Op(vec![], vec![])
    }

    fn compose(a: &Self, b: &Self) -> Self {
        compose(a, b)
    }

    fn compose_iter<'a, I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
        S: 'a,
    {
        let mut base = Self::empty();
        for item in iter {
            base = Self::compose(&base, item);
        }
        base
    }

    fn transform(a: &Self, b: &Self) -> (Self, Self) {
        transform::<S>(&a, &b)
    }

    fn transform_advance(a: &Self, b: &Self) -> Self {
        let (a_transform, _b_transform) = Self::transform(a, b);
        let a_res = Self::compose(a, &a_transform);
        // let b_res = Self::compose(b, &b_transform);
        // assert_eq!(a_res, b_res);
        a_res
    }
}


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
