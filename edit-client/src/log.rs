use super::client;
use extern::{
    crossbeam_channel::Sender,
    edit_common::commands::*,
    std::cell::RefCell,
};

thread_local! {
    pub static CLIENT_LOG_ID: RefCell<Option<String>> = RefCell::new(None);
    pub static CLIENT_LOG_SENDER: RefCell<Option<Sender<UserToSyncCommand>>> = RefCell::new(None);
}

pub fn log_init(tx: Sender<UserToSyncCommand>) -> Option<Sender<UserToSyncCommand>> {
    CLIENT_LOG_SENDER.with(|sender| sender.replace(Some(tx)))
}

pub fn log_send(data: &str) {
    CLIENT_LOG_SENDER.with(|sender| {
        if let Some(ref sender) = *sender.borrow() {
            let _ = sender.send(UserToSyncCommand::Log(data.to_string()));
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

    SendClient(UserToFrontendCommand),
    SendSync(UserToSyncCommand),
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

        let req = ::edit_common::commands::UserToFrontendCommand::UserToSyncCommand(
            ::edit_common::commands::UserToSyncCommand::Log(data.to_string()),
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
