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
mod parse;
mod string;
mod ot;
pub mod deserialize;
pub mod normalize;
pub mod rtf;
pub mod stepper;
pub mod transform_test;
pub mod validate;
pub mod writer;

// Re-export core items.
pub use self::core::*;
