//! Defines types for using operational tranform to modify HTML-like documents.
//!
//! See the book for more details: http://tcr.github.io/edit-text/

#![feature(nll, range_is_empty)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate wasm_typescript_definition;

/* logging */

// Macros can only be used after they are defined

macro_rules! log_transform {
    ($($x:expr),* $(,)*) => {
        // println!( $( $x ),* );
    };
}

macro_rules! log_compose {
    ($($x:expr),* $(,)*) => {
        // println!( $( $x ),* );
    };
}

/* /logging */

// First enable macros.
#[macro_use]
pub mod macros;
#[macro_use]
pub mod wasm;

// Then import & re-export core items.
mod core;
pub use self::core::*;

pub mod normalize;
mod parse;
pub mod stepper;
mod string;
// FIXME pub mod transform_test;
pub mod validate;
pub mod writer;

use crate::apply::*;
use crate::compose::*;
use crate::doc::*;
use crate::transform::transform;
pub use crate::transform::{
    Schema,
    Track,
};

/// A type that can have operational transform applied to it.
/// The `OT` trait is implemented on an operation object, and its
/// associated type `Doc` is what the operation should operate on.
pub trait OT<S: Schema>
where
    Self: Sized,
{
    type Doc;

    /// Applies an operation to a `Self::Doc`, returning the modified `Self::Doc`.
    fn apply(doc: &Self::Doc, op: &Self) -> Self::Doc;

    /// Returns an empty operation.
    fn empty() -> Self;

    /// Composes two operations, returning a single operation encapsulating them
    /// both.
    fn compose(a: &Self, b: &Self) -> Self;

    /// Composes an iterator of operations into a single operation.
    /// If no operations are returned from the iterator, the Op::empty() should be
    /// returned.
    fn compose_iter<'a, I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
        Self: 'a;

    /// Transform a document given the corresponding Schema trait.
    fn transform(a: &Self, b: &Self) -> (Self, Self);

    /// Utility function to transform an operation against a competing one,
    /// returning the results of composing them both.
    fn transform_advance(a: &Self, b: &Self) -> Self;
}

impl<S: Schema> OT<S> for Op<S> {
    type Doc = Doc<S>;

    fn apply(doc: &Self::Doc, op: &Self) -> Self::Doc {
        Doc(apply_operation(&doc.0, op))
    }

    fn empty() -> Self {
        (vec![], vec![])
    }

    fn compose(a: &Self, b: &Self) -> Self {
        compose(a, b)
    }

    fn compose_iter<'a, I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a Self>, S: 'a
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
