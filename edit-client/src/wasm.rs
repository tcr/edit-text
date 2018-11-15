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
use edit_common::markdown::doc_to_markdown;
use edit_common::{
    commands::*,
    doc_as_html,
    markdown::markdown_to_doc,
};
use failure::Error;
use serde_json::Value;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

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

    #[wasm_bindgen(js_namespace = console)]
    pub fn error(msg: &str);

    pub fn setTimeout(closure: &Closure<FnMut()>, time: u32);
}

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => ($crate::wasm::log(&format!($($t)*)))
}
#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => ($crate::wasm::error(&format!($($t)*)))
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

    fn send_client(&self, req: &FrontendCommand) -> Result<(), Error> {
        let data = serde_json::to_string(&req)?;
        let _ = sendCommandToJS(&data);

        Ok(())
    }

    fn send_sync(&self, req: ServerCommand) -> Result<(), Error> {
        self.send_client(&FrontendCommand::ServerCommand(req))
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

    let mut client = WasmClient {
        state: Client {
            client_id: editor_id,
            client_doc: ClientDoc::new(),
            last_controls: None,

            monkey: WASM_MONKEY.clone(),
            alive: WASM_ALIVE.clone(),
            task_count: 0,
        },
    };

    client.setup_controls(None);

    client
}

#[wasm_bindgen]
#[allow(non_snake_case)]
impl WasmClient {
    /// Send a command *from* the frontend *to* the client.
    fn client_task(&mut self, input: Task) -> Result<(), Error> {
        // Do a random roll to see how we react when panicking.
        // use wbg_rand::Rng;
        // let mut rng = wbg_rand::wasm_rng();
        // if rng.gen_range(0, 100) == 0 {
        //     panic!("{} encountered a Panic Monkey!!!!!!!!!!!!", self.state().client_id);
        // }

        match self.handle_task(input.clone()) {
            Ok(_) => {}
            Err(err) => {
                // We could panic here, but some errors are resumable
                console_error!("Error handling task: {:?}\n{:?}", input, err);
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
                console_log!("error parsing task:\n  task: {:?}\n  error: {:?}", input, err);
                return 1;
            }
        }

        // Default status
        0
    }

    pub fn asMarkdown(&mut self) -> String {
        doc_to_markdown(&self.state().client_doc.doc.0).unwrap()
    }
}

#[wasm_bindgen]
pub fn wasm_close() {
    WASM_ALIVE.store(false, Ordering::Relaxed);
    WASM_MONKEY.store(false, Ordering::Relaxed);
}
