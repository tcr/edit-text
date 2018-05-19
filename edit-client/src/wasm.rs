//! Contains the bindings needed for WASM.

extern crate edit_common;
extern crate failure;
extern crate maplit;
extern crate oatie;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate take_mut;

use super::client::*;
use super::state::*;
use edit_common::commands::*;
use failure::Error;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "./../network")]
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

    fn send_client(&self, req: &UserToFrontendCommand) -> Result<(), Error> {
        let data = serde_json::to_string(&req)?;
        let _ = sendCommandToJS(&data);

        Ok(())
    }

    fn send_sync(&self, req: UserToSyncCommand) -> Result<(), Error> {
        self.send_client(&UserToFrontendCommand::UserToSyncCommand(req))
    }
}

#[wasm_bindgen]
pub fn wasm_setup() -> u32 {
    //input_ptr: *mut c_char) -> u32 {

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
            },
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
        Ok(_) => {}
        Err(err) => {
            println!("{:?}", err);
            return 1;
        }
    }

    // Default status
    0
}
