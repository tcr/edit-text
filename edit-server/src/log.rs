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
    pub static ref SERVER_LOG_TX: Arc<Mutex<Sender<String>>> = {
        use std::env::var;

        eprintln!("wtf ....");

        let (tx, rx) = unbounded();
        let _ = ::std::thread::spawn(move || {
            let path = if let Ok(log_file) = var("EDIT_SERVER_LOG") {
                PathBuf::from(log_file)
            } else {
                // No file specified, all output is blackholed.
                return;
            };

            eprintln!("------> {:?}", path);

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
    ($x:expr) => (
        {
            // Load the logging enum variants locally.
            use $crate::log::LogSync::*;

            // Serialize body.
            let ron = ::ron::ser::to_string(&$x).unwrap();

            // Get value.
            let tx = $crate::log::SERVER_LOG_TX.lock().unwrap();
            let _ = tx.send(ron);
        }
    );
}
