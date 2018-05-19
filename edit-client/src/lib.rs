#![feature(crate_in_paths, nll, proc_macro, wasm_custom_section, wasm_import_module)]
#![feature(extern_in_paths)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate oatie;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate take_mut;
#[macro_use]
extern crate lazy_static;
extern crate colored;
extern crate edit_common;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;
extern crate ron;

#[allow(unused)]
#[macro_use]
extern crate wasm_bindgen;

/* logging */

// Macros can only be used after they are defined

use edit_common::commands::*;
use edit_common::*;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref LOG_WASM_FILE: Arc<Mutex<File>> = {
        let path = Path::new("./log/client");
        Arc::new(Mutex::new(File::create(path).unwrap()))
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogWasm {
    Setup(String),
    Task(String, client::Task),
    SyncNew(String),

    SendClient(UserToFrontendCommand),
    SendSync(UserToSyncCommand),
    Debug(String),
}

#[macro_export]
macro_rules! log_wasm {
    ($x:expr) => {{
        use std::env::var;
        use std::io::prelude::*;

        // Only if MERCUTIO_WASM_LOG=1 is set
        if var("MERCUTIO_WASM_LOG") == Ok("1".to_string()) {
            use $crate::LogWasm::*;
            let mut file_guard = $crate::LOG_WASM_FILE.lock().unwrap();
            let mut ron = ::ron::ser::to_string(&$x).unwrap();
            ron = ron.replace("\n", "\\n"); // Escape newlines
            let _ = writeln!(*file_guard, "{}", ron);
            let _ = file_guard.sync_data();
        }
    }};
}

/* /logging */

pub mod actions;
pub mod client;
pub mod random;
pub mod state;
pub mod walkers;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use self::actions::*;
pub use self::client::*;
pub use self::random::*;
pub use self::state::*;
