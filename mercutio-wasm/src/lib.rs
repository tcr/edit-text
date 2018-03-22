//! Connecting to wasm.

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

use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use mercutio_client::client::*;
use mercutio_client::state::*;
use mercutio_common::*;
use failure::Error;
use std::mem;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr as *mut c_void;
}

#[no_mangle]
pub extern "C" fn dealloc_str(ptr: *mut c_char) {
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

extern "C" {
    /// Send a command *to* the js client.
    pub fn js_command(input_ptr: *mut c_char) -> u32;
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
        use std::ffi::CString;
        use std::os::raw::c_char;

        extern "C" {
            /// Send a command *to* the js client.
            pub fn js_command(input_ptr: *mut c_char) -> u32;
        }

        let data = serde_json::to_string(&req)?;
        let s = CString::new(data).unwrap().into_raw();

        unsafe {
            let _ = js_command(s);
        }

        Ok(())
    }

    fn send_sync(&self, req: SyncServerCommand) -> Result<(), Error> {
        self.send_client(&ClientCommand::SyncServerCommand(req))
    }
}

#[no_mangle]
pub fn wasm_setup() -> u32 { //input_ptr: *mut c_char) -> u32 {
    // let input = unsafe {
    //     CString::from_raw(input_ptr)
    // };
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
#[no_mangle]
pub fn wasm_command(input_ptr: *mut c_char) -> u32 {
    let input = unsafe {
        CString::from_raw(input_ptr)
    };
    let req_parse: Result<Task, _> = serde_json::from_slice(&input.into_bytes());
    
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
