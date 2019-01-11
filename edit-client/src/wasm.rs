//! Contains the bindings needed for WASM.

use super::client::*;
use console_error_panic_hook;
use edit_common::markdown::doc_to_markdown;
use edit_common::{
    commands::*,
    doc_as_html,
    markdown::markdown_to_doc,
};
use failure::Error;
use js_sys;
use serde_json;
use std::cell::{
    RefCell,
    RefMut,
};
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys;
use crate::monkey::setup_monkey;

lazy_static! {
    static ref WASM_ALIVE: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    static ref WASM_MONKEY: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

// JS imports

#[wasm_bindgen(module = "./../editor/wasm")]
extern "C" {
    /// Send a command *from* the client *to* the frontend.
    pub fn sendCommandToJS(input: &str) -> u32;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(msg: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn error(msg: &str);

    pub fn setTimeout(closure: &Closure<dyn FnMut()>, time: u32);
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
#[derive(Clone)]
pub struct WasmClientController {
    state: Rc<RefCell<Client>>,
    ws: Rc<RefCell<Option<web_sys::WebSocket>>>,
}

impl ClientController for WasmClientController {
    fn state(&mut self) -> RefMut<Client> {
        self.state.borrow_mut()
    }

    fn send_frontend(&self, command: &FrontendCommand) -> Result<(), Error> {
        let data = serde_json::to_string(&command)?;
        let _ = sendCommandToJS(&data);

        Ok(())
    }

    fn send_server(&self, command: &ServerCommand) -> Result<(), Error> {
        let command_data = serde_json::to_string(command).unwrap();
        let command_json: serde_json::Value = serde_json::from_str(&command_data).unwrap();
        let command_jsvalue = js_sys::JSON::parse(&command_data).unwrap();

        if cfg!(feature = "console_command_log") {
            console_group_collapsed_str_str(
                "[server]",
                command_json
                    .as_object()
                    .unwrap()
                    .get("tag")
                    .unwrap()
                    .as_str()
                    .unwrap(),
            );
            console_debug_jsvalue(command_jsvalue);
            console_group_end();
        }

        if let Some(ref mut ws) = *self.ws.borrow_mut() {
            let _ = ws.send_with_str(&command_data);
        } else {
            console_log!("THIS IS A FATAL ERROR SERVER COMMAND BEFORE CONNECTION");
        }
        Ok(())
        // self.send_client(&FrontendCommand::ServerCommand(req))
    }
}

#[wasm_bindgen]
pub struct WebsocketSend {
    closure: Box<FnMut(String)>,
}

#[wasm_bindgen]
impl WebsocketSend {
    pub fn call(&mut self, value: String) {
        (self.closure)(value);
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = "groupCollapsed")]
    fn console_group_collapsed_str_str(a: &str, b: &str);

    #[wasm_bindgen(js_namespace = console, js_name = "debug")]
    fn console_debug_jsvalue(a: JsValue);

    #[wasm_bindgen(js_namespace = console, js_name = "debug")]
    fn console_debug_str(a: &str);

    #[wasm_bindgen(js_namespace = console, js_name = "groupEnd")]
    fn console_group_end();
}

#[wasm_bindgen]
impl WasmClientController {
    #[wasm_bindgen(js_name = "clientID")]
    pub fn client_id(&self) -> String {
        self.state.borrow().client_doc.client_id.clone()
    }

    /// Handle a Client or Controller command. [sic]
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

    /// Send a command *from* the frontend *or* server *to* the client *or* controller. [sic]
    pub fn command(&mut self, input: &str) -> u32 {
        let req_parse: Result<Task, _> = serde_json::from_slice(&input.as_bytes());

        match req_parse {
            Ok(task) => {
                if self.client_task(task).is_err() {
                    return 1;
                }
            }
            Err(err) => {
                console_log!(
                    "error parsing task:\n  task: {:?}\n  error: {:?}",
                    input,
                    err
                );
                return 1;
            }
        }

        // Default status
        0
    }

    #[wasm_bindgen(js_name = "asMarkdown")]
    pub fn as_markdown(&mut self) -> String {
        doc_to_markdown(&self.state().client_doc.doc.0).unwrap()
    }

    #[wasm_bindgen(js_name = "asJSON")]
    pub fn as_json(&mut self) -> JsValue {
        JsValue::from_serde(&self.state().client_doc.doc).unwrap()
    }

    /// Creates a websocket connection to the server, forwarding server-received
    /// messages to the Client implementation and returning a method to write
    /// commands to the server.
    #[wasm_bindgen(js_name = "subscribeServer")]
    pub fn subscribe_server(&self, ws_url: String) -> Result<WebsocketSend, JsValue> {
        *self.ws.borrow_mut() = Some(web_sys::WebSocket::new(&ws_url)?);

        {
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                // console.debug('server socket opened.');
                // DEBUG.measureTime('connect-ready');
                console_log!("(W) Server socket opened.");
            }) as Box<dyn FnMut(_)>);
            if let Some(ref mut ws) = *self.ws.borrow_mut() {
                ws.add_event_listener_with_callback("open", closure.as_ref().unchecked_ref())?;
            } else {
                unreachable!();
            }
            closure.forget();
        }

        // let client = self.clone();
        {
            let mut controller = self.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
                let command_data = event.data().as_string().unwrap();

                let command: ClientCommand = serde_json::from_str(&command_data).unwrap();
                let command_json: serde_json::Value = serde_json::from_str(&command_data).unwrap();
                let command_jsvalue = js_sys::JSON::parse(&command_data).unwrap();

                if cfg!(feature = "console_command_log") {
                    console_group_collapsed_str_str(
                        "[client]",
                        command_json
                            .as_object()
                            .unwrap()
                            .get("tag")
                            .unwrap()
                            .as_str()
                            .unwrap(),
                    );
                    console_debug_str(&command_data);
                    console_debug_jsvalue(command_jsvalue);
                    console_group_end();
                }

                // TODO why do we have to create a whole wasmclient clone exactly
                // Handle the client command.
                controller
                    .handle_task(Task::ClientCommand(command))
                    .expect("Client task failed");
            }) as Box<dyn FnMut(_)>);
            if let Some(ref mut ws) = *self.ws.borrow_mut() {
                ws.add_event_listener_with_callback("message", closure.as_ref().unchecked_ref())?;
            } else {
                unreachable!();
            }
            closure.forget();
        }

        {
            let mut controller = self.clone();
            let closure = Closure::wrap(Box::new(move |_event: web_sys::CloseEvent| {
                console_log!("#### SERVER DISCONNECT");
                controller
                    .handle_task(Task::ClientCommand(ClientCommand::ServerDisconnect))
                    .expect("Client task failed");
            }) as Box<dyn FnMut(_)>);
            if let Some(ref mut ws) = *self.ws.borrow_mut() {
                ws.add_event_listener_with_callback("close", closure.as_ref().unchecked_ref())?;
            } else {
                unreachable!();
            }
            closure.forget();
        }

        let ws = self.ws.clone();
        Ok({
            WebsocketSend {
                closure: Box::new(move |value: String| {
                    if let Some(ref mut ws) = *ws.borrow_mut() {
                        let _ = ws.send_with_str(&value);
                    } else {
                        unreachable!();
                    }
                }),
            }
        })
    }
}

// Wasm free functions.

#[wasm_bindgen]
pub fn wasm_setup(server_url: String) -> WasmClientController {
    // Set the panic hook to log to console.error.
    console_error_panic_hook::set_once();

    let editor_id = "$$$$$$".to_string();

    // Setup monkey tasks.
    let client = Rc::new(RefCell::new(Client {
        client_doc: ClientDoc::new(editor_id.clone()),
        last_controls: None,
        last_caret_state: None,

        monkey: WASM_MONKEY.clone(),
        alive: WASM_ALIVE.clone(),
        task_count: 0,
    }));

    let mut controller = WasmClientController {
        state: client.clone(),
        ws: Rc::new(RefCell::new(None)),
    };

    setup_monkey::<WasmClientController>(client, crate::monkey::Scheduler::new(controller.clone(), WASM_ALIVE.clone(), WASM_MONKEY.clone()));

    // Subscriber to server via websockets.
    let _ = controller.subscribe_server(server_url);

    // Initialize controls.
    controller.setup_controls(None);

    controller
}

#[wasm_bindgen]
pub fn wasm_close() {
    WASM_ALIVE.store(false, Ordering::Relaxed);
    WASM_MONKEY.store(false, Ordering::Relaxed);
}
