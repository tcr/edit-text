#![feature(crate_in_paths, nll, proc_macro, wasm_custom_section, wasm_import_module)]
#![feature(extern_in_paths)]
#![allow(unused_imports)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate oatie;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate take_mut;
#[macro_use]
extern crate lazy_static;
extern crate colored;
extern crate edit_common;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;
extern crate ron;
extern crate wbg_rand;

#[allow(unused)]
#[macro_use]
extern crate wasm_bindgen;

// Macros has to come first
#[macro_use]
pub mod log;

#[cfg(target_arch = "wasm32")]
#[macro_use]
pub mod wasm;

pub mod actions;
pub mod client;
pub mod monkey;
pub mod random;
pub mod state;
pub mod walkers;

#[cfg(not(target_arch = "wasm32"))]
pub mod proxy;

pub use self::actions::*;
pub use self::client::*;
pub use self::random::*;
pub use self::state::*;
