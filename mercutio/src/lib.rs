#![feature(crate_in_paths)]

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
extern crate colored;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;

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
