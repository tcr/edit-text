//! Defines utility functions and operation application.

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
extern crate yansi;
extern crate serde_json;
extern crate term_painter;
#[macro_use]
extern crate failure;
extern crate regex;
extern crate either;

/* logging */

// Macros can only be used after they are defined

macro_rules! log_transform {
    ( $( $x:expr ),* $(,)* ) => {
        // println!( $( $x ),* );
    };
}

macro_rules! log_compose {
    ( $( $x:expr ),* $(,)* ) => {
        // println!( $( $x ),* );
    };
}

/* /logging */


pub mod compose;
pub mod doc;
//pub mod random;
pub mod schema;
pub mod stepper;
pub mod transform;
pub mod writer;
pub mod transform_test;
pub mod macros;
pub mod apply;
pub mod parse;
pub mod validate;

pub use apply::*;
use doc::*;
use compose::*;
use std::collections::HashMap;
use std::fmt::Debug;
pub use transform::{Schema, Track};
use transform::transform;

pub trait OT where Self: Sized {
    type Doc;
    
    fn apply(&Self::Doc, &Self) -> Self::Doc;
    fn empty() -> Self;
    fn compose(&Self, &Self) -> Self;
    fn compose_iter<'a, I>(iter: I) -> Self where I: Iterator<Item=&'a Self>, Self: 'a;
    fn transform<S: Schema>(&Self, &Self) -> (Self, Self);
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

    fn compose_iter<'a, I>(iter: I) -> Self where I: Iterator<Item=&'a Self> {
        let mut base = Self::empty();
        for item in iter {
            base = Self::compose(&base, item);
        }
        base
    }

    fn transform<S: Schema>(a: &Self, b: &Self) -> (Self, Self) {
        transform::<S>(&a, &b)
    }
}
