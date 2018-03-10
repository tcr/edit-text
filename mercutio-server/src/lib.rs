#![feature(crate_in_paths)]

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
                use $crate::LOG_SYNC_FILE;
                use $crate::LogSync::*;
                let mut file_guard = LOG_SYNC_FILE.lock().unwrap();
                let _ = writeln!(*file_guard, "{}", ron::ser::to_string(&$x).unwrap());
                let _ = file_guard.sync_data();
            }
        }
    };
}

extern crate bus;
extern crate crossbeam_channel;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate oatie;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate taken;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate take_mut;
#[macro_use]
extern crate lazy_static;
extern crate ron;
extern crate ws;
extern crate colored;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate url;
extern crate mercutio;

// Macros can only be used after they are defined
pub mod sync;
pub mod schema;

pub use mercutio::*;