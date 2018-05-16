#![feature(crate_in_paths, extern_in_paths, nll)]
#![feature(non_modrs_mods)]
#![feature(plugin, proc_macro)]

use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use edit_common::commands::*;

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
    ClientPacket(UserToSyncCommand),
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

extern crate thread_spawn;
extern crate colored;
extern crate crossbeam_channel;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
extern crate edit_common;
#[macro_use]
extern crate oatie;
extern crate pulldown_cmark;
extern crate pulldown_cmark_to_cmark;
extern crate rand;
extern crate ron;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate simple_ws;
extern crate reqwest;
extern crate take_mut;
#[macro_use]
extern crate taken;
extern crate url;
extern crate ws;
#[macro_use] extern crate rouille;
#[macro_use] extern crate juniper;
extern crate r2d2;
extern crate r2d2_diesel;

// Macros can only be used after they are defined
pub mod db;
pub mod graphql;
pub mod sync;
pub mod state;
pub mod carets;
