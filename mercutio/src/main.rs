#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate oatie;
extern crate rand;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate take_mut;
extern crate ws;

pub mod wasm;
pub mod sync;

use std::sync::{Arc, Mutex};
use oatie::doc::*;
use oatie::{Operation, OT};
use rocket_contrib::Json;
use rocket::State;
use rocket::response::NamedFile;
use failure::Error;
use serde_json::Value;
use std::path::{Path, PathBuf};
use oatie::transform::transform;
use oatie::debug_pretty;
use wasm::start_websocket_server;
use std::thread;
use sync::*;
use oatie::schema::{validate_doc_span, ValidateContext};

#[get("/")]
fn root() -> Option<NamedFile> {
    Some(Path::new(".").join("src/templates/").join("index.html"))
        .and_then(|x| NamedFile::open(x).ok())
}

#[get("/client")]
fn client() -> Option<NamedFile> {
    Some(Path::new(".").join("src/templates/").join("client.html"))
        .and_then(|x| NamedFile::open(x).ok())
}

#[get("/favicon.png")]
fn favicon() -> Option<NamedFile> {
    Some(Path::new(".").join("src/templates/").join("favicon.png"))
        .and_then(|x| NamedFile::open(x).ok())
}

#[get("/<file..>", rank = 2)]
fn files(file: PathBuf) -> Option<NamedFile> {
    Some(Path::new(".").join("frontend/dist/").join(file)).and_then(|x| NamedFile::open(x).ok())
}

fn main() {
    let mercutio_state = MoteState {
        body: Arc::new(Mutex::new(default_doc())),
    };

    sync_socket_server(mercutio_state.clone());
    start_websocket_server();

    rocket::ignite()
        .mount("/", routes![root, client, files, favicon])
        .launch();
}
