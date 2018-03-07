use crate::*;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::fs::File;

lazy_static! {
    static ref LOG_SYNC_FILE: Arc<Mutex<File>> = {
        let path = Path::new("./log/server");
        Arc::new(Mutex::new(File::create(path).unwrap()))
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LogSync {
    Launch,
    ServerSpawn,
    ClientConnect,
    ClientPacket(SyncServerCommand),
    Debug(String),
    Spawn,
}

#[macro_export]
macro_rules! log_sync {
    ( $x:expr ) => {
        {
            use std::io::prelude::*;
            use std::env::var;

            // Only if MERCUTIO_SYNC_LOG=1 is set
            if var("MERCUTIO_SYNC_LOG") == Ok("1".to_string()) {
                use $crate::server::LOG_SYNC_FILE;
                use $crate::server::LogSync::*;
                let mut file_guard = LOG_SYNC_FILE.lock().unwrap();
                let _ = writeln!(*file_guard, "{}", ron::ser::to_string(&$x).unwrap());
                let _ = file_guard.sync_data();
            }
        }
    };
}

// Macros can only be used after they are defined
#[cfg(not(target_arch="wasm32"))]
pub mod sync;
#[cfg(not(target_arch="wasm32"))]
pub mod schema;