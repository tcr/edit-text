#![feature(extern_in_paths, nll, plugin)]
#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate oatie;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate taken;
#[macro_use]
extern crate rouille;
#[macro_use]
extern crate juniper;

#[macro_use]
pub mod log;

// Macros can only be used after they are defined
pub mod carets;
pub mod db;
pub mod graphql;
pub mod state;
pub mod sync;
