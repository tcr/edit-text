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
extern crate serde_derive;
extern crate serde_json;
extern crate take_mut;
#[macro_use]
extern crate lazy_static;

#[cfg(not(target_arch="wasm32"))]
extern crate ws;
#[macro_use]
pub mod wasm;
#[cfg(not(target_arch="wasm32"))]
pub mod sync;

use oatie::doc::*;


// TODO move the below to a file

#[derive(Serialize, Deserialize, Debug)]
pub enum SyncServerCommand {
    // Connect(String),
    Commit(String, Op, usize),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SyncClientCommand {
    Update(DocSpan, usize, String, Op),
}
