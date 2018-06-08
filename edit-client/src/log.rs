use super::client;
use extern::{
    crossbeam_channel::{unbounded, Sender},
    edit_common::commands::*,
    std::fs::File,
    std::path::PathBuf,
    std::sync::{Arc, Mutex},
    std::io::prelude::*,
};

lazy_static! {
    pub static ref CLIENT_LOG_TX: Arc<Mutex<Sender<String>>> = {
        use std::env::var;

        eprintln!("wtf ....");

        let (tx, rx) = unbounded();
        let _ = ::std::thread::spawn(move || {
            let path = if let Ok(log_file) = var("EDIT_CLIENT_LOG") {
                PathBuf::from(log_file)
            } else {
                // No file specified, all output is blackholed.
                return;
            };

            // eprintln!("------> {:?}", path);

            // Crate the file.
            let mut f = File::create(path).unwrap();

            // Write all input to the log file.
            while let Ok(value) = rx.recv() {
                let _ = writeln!(f, "{}", value);
                // let _ = f.sync_data();
            }
        });

        // The static variable is the transmission channel.
        Arc::new(Mutex::new(tx))
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
    ($x:expr) => (
        {
            // Load the logging enum variants locally.
            use $crate::log::LogWasm::*;

            // Only if MERCUTIO_WASM_LOG=1 is set
            let tx = $crate::log::CLIENT_LOG_TX.lock().unwrap();
            let mut ron = ::ron::ser::to_string(&$x).unwrap();
            ron = ron.replace("\n", "\\n"); // TODO escaping embedded string newlines was fixed in a recent version of ron
            let _ = tx.send(ron);
        }
    );
}
