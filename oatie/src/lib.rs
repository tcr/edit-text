//! Defines types for using operational tranform to modify HTML-like documents.
//!
//! See the book for more details: http://tcr.github.io/edit-text/

#![feature(nll)]
#![allow(unknown_lints)]
#![allow(single_char_pattern)]
#![allow(ptr_arg)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate maplit;
// extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate term_painter;
extern crate yansi;
#[macro_use]
extern crate failure;
extern crate either;
extern crate regex;
extern crate ron;
extern crate serde;

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

pub mod compose;
pub mod doc;
//pub mod random;
pub mod apply;
pub mod macros;
mod parse;
pub mod schema;
pub mod stepper;
pub mod transform;
pub mod transform_test;
pub mod validate;
pub mod writer;

use apply::*;
use compose::*;
use doc::*;
use std::collections::HashMap;
use std::fmt::Debug;
use transform::transform;
pub use transform::{Schema, Track};

/// A type that can have operational transform applied to it.
/// The `OT` trait is implemented on an operation object, and its
/// associated type `Doc` is what the operation should operate on.
pub trait OT
where
    Self: Sized,
{
    type Doc;

    /// Applies an operation to a `Self::Doc`, returning the modified `Self::Doc`.
    fn apply(&Self::Doc, &Self) -> Self::Doc;

    /// Returns an empty operation.
    fn empty() -> Self;

    /// Composes two operations, returning a single operation encapsulating them
    /// both.
    fn compose(&Self, &Self) -> Self;

    /// Composes an iterator of operations into a single operation.
    /// If no operations are returned from the iterator, the OT::empty() should be
    /// returned.
    fn compose_iter<'a, I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
        Self: 'a;

    /// Transform a document given the corresponding Schema trait.
    fn transform<S: Schema>(&Self, &Self) -> (Self, Self);

    /// Utility function to transform an operation against a competing one,
    /// returning the results of composing them both.
    fn transform_advance<S: Schema>(a: &Self, b: &Self) -> Self;
}

impl OT for Op {
    type Doc = Doc;

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
        I: Iterator<Item = &'a Self>,
    {
        let mut base = Self::empty();
        for item in iter {
            base = Self::compose(&base, item);
        }
        base
    }

    fn transform<S: Schema>(a: &Self, b: &Self) -> (Self, Self) {
        transform::<S>(&a, &b)
    }

    fn transform_advance<S: Schema>(a: &Self, b: &Self) -> Self {
        let (a_transform, b_transform) = Self::transform::<S>(a, b);
        let a_res = Self::compose(a, &a_transform);
        let b_res = Self::compose(a, &a_transform);
        // assert_eq!(a_res, b_res);
        a_res
    }
}
