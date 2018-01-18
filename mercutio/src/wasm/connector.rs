use super::actions::*;
#[cfg(not(target_arch="wasm32"))]
use super::super::{SyncClientCommand, SyncServerCommand};
#[cfg(not(target_arch="wasm32"))]
use crossbeam_channel::{unbounded, Sender};
use failure::Error;
use oatie::{Operation, OT};
use oatie::doc::*;
use rand;
use rand::Rng;
use serde_json;
use std::{panic, process};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use super::*;
#[cfg(not(target_arch="wasm32"))]
use ws;
#[macro_use]
use lazy_static;

#[cfg(not(target_arch="wasm32"))]
use self::proxy::*;

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

lazy_static! {
    // TODO instantiate the client
    static ref WASM_CLIENT: Mutex<Client> = Mutex::new(Client {
        name: "hello".to_owned(),

        doc: Doc(vec![]),
        version: 100,

        original_doc: Doc(vec![]),
        original_ops: vec![],

        monkey: Arc::new(AtomicBool::new(false)),
        alive: Arc::new(AtomicBool::new(false)),
    });
}

extern "C" {
    /// Send a command *to* the js client.
    pub fn js_command(input_ptr: *mut c_char) -> u32;
}

#[no_mangle]
pub fn wasm_test() -> u32 {
    1337
}

/// Send a command *to* the wasm client.
#[no_mangle]
pub fn wasm_command(input_ptr: *mut c_char) -> u32 {
    let input = unsafe {
        CString::from_raw(input_ptr)
    };
    let req_parse: Result<Task, _> = serde_json::from_slice(&input.into_bytes());
    
    let mut client = WASM_CLIENT.lock().unwrap();

    match req_parse {
        Ok(task) => {
            client.handle_task(task);
        }
        Err(err) => {
            println!("{:?}", err);
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
