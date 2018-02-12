/* logging */

// Macros can only be used after they are defined

use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::io::prelude::*;
use ron;
use super::*;

lazy_static! {
    static ref LOG_WASM_FILE: Arc<Mutex<File>> = {
        use std::env::var;
        let path = Path::new(if var("MERCUTIO_WASM_LOG") != Ok("0".to_string()) {
            "./log/client"
        } else {
            "./log/replay"
        });
        Arc::new(Mutex::new(File::create(path).unwrap()))
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogWasm {
    Setup(String),
    Task(String, client::Task),
    SyncNew(String),

    SendClient(ClientCommand),
    SendSync(SyncServerCommand),
    Debug(String),
}

macro_rules! log_wasm {
    ( $x:expr ) => {
        {
            use std::io::prelude::*;

            // use $crate::wasm::LogWasm::*;
            let mut file_guard = LOG_WASM_FILE.lock().unwrap();
            use $crate::wasm::LogWasm::*;
            let mut ron = ::ron::ser::to_string(&$x).unwrap();
            ron = ron.replace("\n", "\\n"); // Escape newlines
            writeln!(*file_guard, "{}", ron);
            let _ = file_guard.sync_data();
        }
    };
}

/* /logging */

pub mod actions;
pub mod walkers;
#[cfg(not(target_arch="wasm32"))]
pub mod proxy;
pub mod util;
pub mod state;
pub mod client;

pub use self::client::*;
pub use self::state::*;
pub use self::util::*;
pub use self::actions::*;
