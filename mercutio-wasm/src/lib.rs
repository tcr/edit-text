//! Connecting to wasm.

#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate mercutio_common;
extern crate mercutio_client;
extern crate failure;
extern crate maplit;
extern crate oatie;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate take_mut;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate wasm_bindgen;

use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use mercutio_client::client::*;
use mercutio_client::state::*;
use mercutio_common::*;
use failure::Error;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "../network")]
extern "C" {
    /// Send a command *to* the js client.
    pub fn sendCommandToJS(input: &str) -> u32;
}

lazy_static! {
    static ref WASM_CLIENT: Mutex<Option<WasmClient>> = Mutex::new(None);
}


pub struct WasmClient {
    pub state: Client,
}

impl ClientImpl for WasmClient {
    fn state(&mut self) -> &mut Client {
        &mut self.state
    }

    fn send_client(&self, req: &ClientCommand) -> Result<(), Error> {
        let data = serde_json::to_string(&req)?;
        let _ = sendCommandToJS(&data);

        Ok(())
    }

    fn send_sync(&self, req: SyncServerCommand) -> Result<(), Error> {
        self.send_client(&ClientCommand::SyncServerCommand(req))
    }
}

#[wasm_bindgen]
pub fn wasm_setup() -> u32 { //input_ptr: *mut c_char) -> u32 {

    let editor_id = "$$$$$$".to_string();

    {
        let mut client_lock = WASM_CLIENT.lock().unwrap();

        let monkey = Arc::new(AtomicBool::new(false));
        let alive = Arc::new(AtomicBool::new(true));

        let client = WasmClient {
            state: Client {
                client_id: editor_id,
                client_doc: ClientDoc::new(),

                monkey,
                alive,
            }
        };

        client.setup_controls(None);

        *client_lock = Some(client);
    }

    0
}

/// Send a command *to* the wasm client.
#[wasm_bindgen]
pub fn wasm_command(input: &str) -> u32 {
    let req_parse: Result<Task, _> = serde_json::from_slice(&input.as_bytes());
    
    let mut client_lock = WASM_CLIENT.lock().unwrap();
    let client = client_lock.as_mut().unwrap();

    match req_parse.map(|task| client.handle_task(task)) {
        Ok(_) => {},
        Err(err) => {
            println!("{:?}", err);
            return 1;
        }
    }

    // Default status
    0
}
