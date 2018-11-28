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
extern crate js_sys;

use super::client::*;
use super::monkey::*;
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
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys;
use wasm_bindgen::JsCast;

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
    pub fn client_id(&self) -> String {
        self.state.client_id.clone()
    }

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

    pub fn subscribeServer(&mut self, ws_url: String, command_callback: js_sys::Function) -> Result<(), JsValue> {
        let command_callback = Rc::new(command_callback);

        let ws = Rc::new(web_sys::WebSocket::new(&ws_url)?);

        {
            let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {

                // console.debug('server socket opened.');
                // DEBUG.measureTime('connect-ready');

                console_log!("####### SERVER SOCKET OPENED");

            }) as Box<FnMut(_)>);
            ws.add_event_listener_with_callback("open", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let command_callback = command_callback.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
                // console_log!("message {:?}", event.data().as_string().unwrap());

                command_callback.call1(&JsValue::NULL, &event.data());

                // command_callback.call1(&JsValue::NULL, &JsValue::from(r##"{"tag": "Error", "fields": "Big error today boys"}"##));

                // // console.log('Got message from sync:', event.data);
                // try {
                //     if (getForwardWasmTaskCallback() != null) {
                //     if (server.client != null) {
                //         let command = JSON.parse(event.data);
                //         console.groupCollapsed('[client]', command.tag);
                //         console.debug(command);
                //         console.groupEnd();
                //         server.client.clientBindings.command(JSON.stringify({
                //         ClientCommand: command,
                //         }));
                //     }
                //     }
                // } catch (e) {
                //     // Kill the current process, we triggered an exception.
                //     setForwardWasmTaskCallback(null);
                //     if (server.client != null) {
                //     server.client.Module.wasm_close();
                //     }
                //     // syncSocket.close();

                //     // TODO this is the wrong place to put this
                //     (document as any).body.background = 'red';

                //     if (server.editorFrame) {
                //     onError(
                //         <div>The client experienced an error talking to the server and you are now disconnected. We're sorry. You can <a href="?">refresh your browser</a> to continue.</div>
                //     );
                //     }

                //     throw new WasmError(e, `Error during sync command: ${e.message}`);
                // }

            }) as Box<FnMut(_)>);
            ws.add_event_listener_with_callback("message", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let command_callback = command_callback.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::CloseEvent| {

                let command = FrontendCommand::ServerDisconnect;

                command_callback.call1(&JsValue::NULL, &JsValue::from_serde(&command).unwrap());


                // if (server.editorFrame) { 
                //     onError(
                //     <div>The editor has disconnected from the server. We're sorry. You can <a href="?">refresh your browser</a>, or we'll refresh once the server is reachable.</div>
                //     );
                // }

                // setTimeout(() => {
                //     setInterval(() => {
                //     app.graphqlPage('home').then(() => {
                //         // Can access server, continue
                //         window.location.reload();
                //     });
                //     }, 2000);
                // }, 3000);

                // server.onClose();
                
            }) as Box<FnMut(_)>);
            ws.add_event_listener_with_callback("close", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        Ok(())
    }
}

#[wasm_bindgen]
pub fn wasm_close() {
    WASM_ALIVE.store(false, Ordering::Relaxed);
    WASM_MONKEY.store(false, Ordering::Relaxed);
}
