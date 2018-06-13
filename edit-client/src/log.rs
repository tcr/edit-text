use super::client;
use extern::{
    crossbeam_channel::{unbounded, Sender},
    edit_common::commands::*,
    std::fs::File,
    std::path::PathBuf,
    std::sync::{Arc, Mutex},
    std::io::prelude::*,
    std::cell::RefCell,
};

thread_local! {
    pub static CLIENT_LOG_SENDER: RefCell<Option<Sender<UserToSyncCommand>>> = RefCell::new(None);
}

pub fn log_init(tx: Sender<UserToSyncCommand>) -> Option<Sender<UserToSyncCommand>> {
    CLIENT_LOG_SENDER.with(|sender| {
        sender.replace(Some(tx))
    })
}

pub fn log_send(data: &str) {
    CLIENT_LOG_SENDER.with(|sender| {
        if let Some(ref sender) = *sender.borrow() {
            sender.send(UserToSyncCommand::Log(data.to_string()));
        } else {
            eprintln!("(~) error: logging without a logger: {}",
                &data.chars().take(256).collect::<String>());
        }
    })
}

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
pub enum LogWasm {
    Setup(String),
    Task(String, client::Task),
    SyncNew(String),

    SendClient(UserToFrontendCommand),
    SendSync(UserToSyncCommand),
    Debug(String),
}

// TODO switch on a debug flag/feature or something
#[macro_export]
#[cfg(target_arch = "wasm32")]
macro_rules! log_wasm {
    ($x:expr) => (
        {
            // Load the logging enum variants locally.
            // use $crate::log::LogWasm::*;

            // Serialize body.
            // let ron = ::ron::ser::to_string(&$x).unwrap();

            // console_log!("[WASM_LOG] {}", ron);
        }
    );
}

#[macro_export]
#[cfg(not(target_arch = "wasm32"))]
macro_rules! log_wasm {
    ($x:expr) => (
        {
            // Load the logging enum variants locally.
            use $crate::log::LogWasm::*;
            use $crate::log::log_send;

            // Serialize body.
            let ron = ::ron::ser::to_string(&$x).unwrap();

            log_send(&ron);
        }
    );
}
