//! Connecting to wasm.

extern crate mercutio; 
#[macro_use]
extern crate failure;
#[macro_use]
extern crate maplit;
extern crate oatie;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate take_mut;
#[macro_use]
extern crate lazy_static;

use mercutio::wasm::actions::*;
use failure::Error;
use rand::Rng;
use std::{panic, process};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use mercutio::*;
use mercutio::wasm::*;
use oatie::doc::*;

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

use std::collections::HashMap;



extern "C" {
    /// Send a command *to* the js client.
    pub fn js_command(input_ptr: *mut c_char) -> u32;
}

lazy_static! {
    // TODO instantiate the client
    static ref WASM_CLIENT: Mutex<Option<Client>> = Mutex::new(None);
}

#[no_mangle]
pub fn wasm_setup(input_ptr: *mut c_char) -> u32 {
    let input = unsafe {
        CString::from_raw(input_ptr)
    };
    let editor_id = input.to_string_lossy().to_string();

    {
        let mut client_lock = WASM_CLIENT.lock().unwrap();

        let client = Client {
            name: editor_id,

            doc: Doc(vec![]),
            version: 100,

            original_doc: Doc(vec![]),
            original_ops: vec![],

            monkey: Arc::new(AtomicBool::new(false)),
            alive: Arc::new(AtomicBool::new(false)),
        };

        client.setup();

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
    let mut client = client_lock.as_mut().unwrap();

    match req_parse {
        Ok(task) => {
            client.handle_task(task);
        }
        Err(err) => {
            println!("{:?}", err);
            return 1;
        }
        // _ => command_safe(NativeRequest::Invalid),
    }

    // Default status
    0

    // Fetch lazy_static intitialized client.
    // send it the payload.
    // if it needs to callback, it just calls the client extern function.

    // let res = "hi";

    // let json = serde_json::to_string(&res).unwrap();
    // let s = CString::new(json).unwrap();
    // s.into_raw()
}
