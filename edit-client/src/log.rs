extern crate crossbeam_channel;

use super::client;
use self::crossbeam_channel::Sender;
use edit_common::commands::*;
use std::cell::RefCell;

thread_local! {
    pub static CLIENT_LOG_ID: RefCell<Option<String>> = RefCell::new(None);
    pub static CLIENT_LOG_SENDER: RefCell<Option<Sender<ServerCommand>>> = RefCell::new(None);
}

pub fn log_init(tx: Sender<ServerCommand>) -> Option<Sender<ServerCommand>> {
    CLIENT_LOG_SENDER.with(|sender| sender.replace(Some(tx)))
}

pub fn log_send(data: &str) {
    CLIENT_LOG_SENDER.with(|sender| {
        if let Some(ref sender) = *sender.borrow() {
            let _ = sender.send(ServerCommand::Log(data.to_string()));
        } else {
            eprintln!(
                "(~) error: logging without a logger: {}",
                &data.chars().take(256).collect::<String>()
            );
        }
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogWasm {
    Setup(String),
    Task(String, client::Task),
    SyncNew(String),

    SendClient(FrontendCommand),
    SendSync(ServerCommand),
    Debug(String),
}

// TODO switch on a debug flag/feature or something
#[macro_export]
#[cfg(target_arch = "wasm32")]
macro_rules! log_wasm {
    ($x:expr) => {{
        // Load the logging enum variants locally.
        use $crate::log::LogWasm::*;

        // Serialize body.
        let data = ::ron::ser::to_string(&$x).unwrap();

        // console_log!("[WASM_LOG] {}", ron);

        let req = ::edit_common::commands::FrontendCommand::ServerCommand(
            ::edit_common::commands::ServerCommand::Log(data.to_string()),
        );
        let data = ::serde_json::to_string(&req).unwrap();
        use $crate::wasm::sendCommandToJS;
        let _ = sendCommandToJS(&data);
    }};
}

#[macro_export]
#[cfg(not(target_arch = "wasm32"))]
macro_rules! log_wasm {
    ($x:expr) => {{
        // Load the logging enum variants locally.
        use $crate::log::log_send;
        use $crate::log::LogWasm::*;

        // Serialize body.
        let data = ::ron::ser::to_string(&$x).unwrap();

        log_send(&data);
    }};
}
