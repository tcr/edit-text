extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

use std::mem;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

pub fn fact(n: u32) -> u64 {
    let mut n = n as u64;
    let mut result = 1;
    while n > 0 {
        result = result * n;
        n = n - 1;
    }
    result
}

#[derive(Serialize, Deserialize)]
enum NativeRequest {
    Factorial(u32),
    Invalid,
}

#[derive(Serialize, Deserialize)]
enum NativeResponse {
    Factorial(u64),
    Error(String),
}

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

#[no_mangle]
pub fn command(input_ptr: *mut c_char) -> *mut c_char {
    let input = unsafe {
        CString::from_raw(input_ptr)
    };
    let req: NativeRequest = serde_json::from_slice(&input.into_bytes())
        .unwrap_or(NativeRequest::Invalid);

    let res = match req {
        NativeRequest::Factorial(factor) => {
            NativeResponse::Factorial(fact(factor))
        }
        _ => {
            NativeResponse::Error("Unknown request".to_string())
        }
    };
    let json = serde_json::to_string(&res).unwrap();
    let s = CString::new(json).unwrap();
    s.into_raw()
}