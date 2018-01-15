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
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate yansi;
extern crate serde_json;
extern crate term_painter;
#[macro_use]
extern crate failure;
extern crate regex;
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
pub mod parse;
pub mod validate;

pub use apply::*;
use doc::*;
use compose::*;
use std::collections::HashMap;
use std::fmt::Debug;

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

// TODO move this obviously somewhere better
pub fn debug_pretty<D: Debug>(input: &D) -> String {
    let input = format!("{:?}", input);
    
    let mut out = String::new();
    let mut len = "".to_string();
    let mut chars = input.chars().peekable();
    loop {
        let mut c = if let Some(c) = chars.next() { c } else { break; };
        if c == '[' {
            out.push(c);

            while chars.peek().unwrap().is_whitespace() {
                let _ = chars.next();
            }
            if chars.peek() == Some(&']') {
                c = chars.next().unwrap();
            } else {
                out.push_str("\n");

                len.push_str("    ");
                out.push_str(&len);
            }
        } else if c == ']' {
            len = len[0..len.len()-4].to_string();
            out.push_str("\n");
            out.push_str(&len);
        } else if c == '\n' {
            out.push(c);
            out.push_str(&len);
        } else {
            out.push(c);
        }

        if c == ']' {
            out.push(c);
            if chars.peek() == Some(&')') {
                out.push(chars.next().unwrap());
                if chars.peek() == Some(&',') {
                    out.push(chars.next().unwrap());
                    while chars.peek().unwrap().is_whitespace() {
                        let _ = chars.next();
                    }
                    out.push_str("\n");
                    out.push_str(&len);
                }
            }
        }
    }
    out
}