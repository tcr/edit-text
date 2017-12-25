//! Defines utility functions and operation application.

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
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate term_painter;
#[macro_use]
extern crate failure;
extern crate either;

pub mod compose;
pub mod doc;
pub mod random;
pub mod schema;
pub mod stepper;
pub mod transform;
pub mod writer;
pub mod transform_test;
pub mod macros;
pub mod apply;

pub use apply::*;
use doc::*;
use compose::*;
use std::collections::HashMap;

pub trait OT {
    type Op: Operation;
    
    fn apply(&self, &Self::Op) -> Self;
}

pub trait Operation where Self: Sized {
    fn compose(&Self, &Self) -> Self;
    fn transform(&Self, &Self) -> (Self, Self);
}

impl OT for Doc {
    type Op = Op;

    fn apply(&self, op: &Self::Op) -> Self {
        Doc(apply_operation(&self.0, op))
    }
}

impl Operation for Op {
    fn compose(a: &Self, b: &Self) -> Self {
        compose(a, b)
    }

    fn transform(a: &Self, b: &Self) -> (Self, Self) {
        unimplemented!();
    }
}
