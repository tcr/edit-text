#![feature(crate_in_paths, nll)]

use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

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
extern crate colored;
extern crate crossbeam_channel;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
extern crate mercutio_common;
#[macro_use]
extern crate oatie;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;
extern crate rand;
extern crate ron;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate simple_ws;
extern crate take_mut;
#[macro_use]
extern crate taken;
extern crate url;
extern crate ws;

// Macros can only be used after they are defined
pub mod sync;
pub mod schema;
pub mod db;
pub mod util;

pub use mercutio_common::*;
