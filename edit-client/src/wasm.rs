//! Contains the bindings needed for WASM.

extern crate console_error_panic_hook;
extern crate edit_common;
extern crate failure;
extern crate maplit;
extern crate oatie;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate take_mut;
extern crate wbg_rand;

use super::client::*;
use super::monkey::*;
use super::state::*;
use edit_common::{
    doc_as_html,
    commands::*,
    markdown::markdown_to_doc,
};
use failure::Error;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use serde_json::Value;

lazy_static! {
    static ref WASM_ALIVE: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    static ref WASM_MONKEY: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

// JS imports

#[wasm_bindgen(module = "./../editor/wasm")]
extern "C" {
    /// Send a command *from* the client *to* the frontend.
    pub fn sendCommandToJS(input: &str) -> u32;

    pub fn forwardWasmTask(input: &str);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(msg: &str);

    pub fn setTimeout(closure: &Closure<FnMut()>, time: u32);
}

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => ($crate::wasm::log(&format!($($t)*)))
}


#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn convertMarkdownToHtml(input: &str) -> String {
    doc_as_html(&markdown_to_doc(input).unwrap())
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn convertMarkdownToDoc(input: &str) -> String {
    serde_json::to_string(&markdown_to_doc(input).unwrap()).unwrap()
}

// WebAssembly client.

#[wasm_bindgen]
pub struct WasmClient {
    state: Client,
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

// Entry point.

#[wasm_bindgen]
pub fn wasm_setup() -> WasmClient {
    // Set the panic hook to log to console.error.
    console_error_panic_hook::set_once();

    let editor_id = "$$$$$$".to_string();

    // Setup monkey tasks.
    // setup_monkey::<WasmClient>(Scheduler::new(WASM_ALIVE.clone(), WASM_MONKEY.clone()));

    let client = WasmClient {
        state: Client {
            client_id: editor_id,
            client_doc: ClientDoc::new(),

            monkey: WASM_MONKEY.clone(),
            alive: WASM_ALIVE.clone(),
            task_count: 0,
        },
    };

    client.setup_controls(None);

    client
}

#[wasm_bindgen]
impl WasmClient {
    /// Send a command *from* the frontend *to* the client.
    fn client_task(&mut self, input: Task) -> Result<(), Error> {
        // Do a random roll to see how we react when panicking.
        // use wbg_rand::Rng;
        // let mut rng = wbg_rand::wasm_rng();
        // if rng.gen_range(0, 100) == 0 {
        //     panic!("{} encountered a Panic Monkey!!!!!!!!!!!!", self.state().client_id);
        // }

        match self.handle_task(input) {
            Ok(_) => {}
            Err(err) => {
                console_log!("error handling task: {:?}", err);
                return Err(err);
            }
        }

        // Default status
        Ok(())
    }

    /// Send a command *from* the frontend *to* the client.
    pub fn command(&mut self, input: &str) -> u32 {
        let req_parse: Result<Task, _> = serde_json::from_slice(&input.as_bytes());

        match req_parse {
            Ok(task) => {
                if self.client_task(task).is_err() {
                    return 1;
                }
            }
            Err(err) => {
                console_log!("error parsing task: {:?}", err);
                return 1;
            }
        }

        // Default status
        0
    }
}

#[wasm_bindgen]
pub fn wasm_close() {
    WASM_ALIVE.store(false, Ordering::Relaxed);
    WASM_MONKEY.store(false, Ordering::Relaxed);
}
