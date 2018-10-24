#![feature(extern_in_paths, nll, plugin)]
#![allow(proc_macro_derive_resolution_fallback)]

extern crate colored;
extern crate crossbeam_channel;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
extern crate edit_common;
#[macro_use]
extern crate oatie;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;
extern crate rand;
extern crate ron;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate reqwest;
extern crate take_mut;
#[macro_use]
extern crate taken;
extern crate url;
extern crate ws;
#[macro_use]
extern crate rouille;
#[macro_use]
extern crate juniper;
extern crate r2d2;
extern crate r2d2_diesel;

#[macro_use]
pub mod log;

// Macros can only be used after they are defined
pub mod carets;
pub mod db;
pub mod graphql;
pub mod state;
pub mod sync;
