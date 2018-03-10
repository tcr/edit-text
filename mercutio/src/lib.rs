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
extern crate colored;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;
#[cfg(not(target_arch="wasm32"))]
#[macro_use]
extern crate diesel;
#[cfg(not(target_arch="wasm32"))]
extern crate dotenv;
#[cfg(not(target_arch="wasm32"))]
extern crate url;

#[macro_use]
pub mod wasm;
pub mod markdown;

use oatie::doc::*;


// TODO move the below to a file

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncServerCommand {
    // Connect(String),
    Keepalive,
    Commit(String, Op, usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncClientCommand {
    // Client id assignment, initial doc, initial version
    Init(String, DocSpan, usize),

    // New document, version, client-id, operation
    Update(DocSpan, usize, String, Op),
}
