#![feature(crate_in_paths)]

#[cfg(not(target_arch="wasm32"))]
extern crate bus;
#[cfg(not(target_arch="wasm32"))]
extern crate crossbeam_channel;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate oatie;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate taken;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate take_mut;
#[macro_use]
extern crate lazy_static;
extern crate ron;
#[cfg(not(target_arch="wasm32"))]
extern crate ws;

pub mod server;
#[macro_use]
pub mod wasm;

#[cfg(not(target_arch="wasm32"))]
pub use server::sync;

use oatie::doc::*;


// TODO move the below to a file

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncServerCommand {
    // Connect(String),
    Commit(String, Op, usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncClientCommand {
    Update(DocSpan, usize, String, Op),
}
