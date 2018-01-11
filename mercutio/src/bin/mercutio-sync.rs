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
extern crate crossbeam_channel;
extern crate bus;
extern crate mercutio;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use structopt::StructOpt;
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
use mercutio::wasm::start_websocket_server;
use std::thread;
use mercutio::sync::*;
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

#[derive(StructOpt, Debug)]
#[structopt(name = "mercutio-wasm", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(long="port", help = "Port", default_value = "3010")]
    port: u16,

    #[structopt(long="period", help = "Sync period", default_value = "100")]
    period: usize,
}

fn main() {
    let opt = Opt::from_args();

    let mercutio_state = MoteState {
        body: Arc::new(Mutex::new(default_doc())),
    };

    sync_socket_server(opt.port, opt.period, mercutio_state.clone());

    // thread::spawn(|| {
    //     start_websocket_server();
    // });

    loop {
        ::std::thread::sleep(::std::time::Duration::from_millis(1000));
    }

    // rocket::ignite()
    //     .mount("/", routes![root, client, files, favicon])
    //     .launch();
}
