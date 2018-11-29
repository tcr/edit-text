use crate::db::{
    queries::*,
    DbPool,
};

use crossbeam_channel::{
    unbounded,
    Sender,
};
use edit_common::commands::*;
use std::mem;
use std::sync::{
    Arc,
    Mutex,
};

pub struct Logger {
    db_pool: Arc<Mutex<Option<DbPool>>>,
    sender: Sender<(String, String)>,
}

impl Logger {
    fn spawn() -> Logger {
        let db_pool = Arc::new(Mutex::new(None));

        let (tx, rx) = unbounded::<(String, String)>();
        let db_pool_inner = db_pool.clone();
        let _ = ::std::thread::spawn(move || {
            // Write all input to the log file.
            while let Some((source, log)) = rx.recv() {
                if let &mut Some(ref mut pool) = &mut *db_pool_inner.lock().unwrap() {
                    let conn = DbPool::get(pool).unwrap();
                    let _ = create_log(&conn, &source, &log);
                }
            }
        });

        Logger {
            db_pool,
            sender: tx,
        }
    }

    fn replace_db_pool(&self, db_pool: DbPool) -> Option<DbPool> {
        let db_pool_inner = &mut *self.db_pool.lock().unwrap();
        mem::replace(db_pool_inner, Some(db_pool))
    }

    pub fn log(&self, source: String, value: String) -> () {
        self.sender.send((source, value));
    }
}

lazy_static! {
    pub static ref SERVER_LOG_TX: Logger = Logger::spawn();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogSync {
    Launch,
    ServerSpawn,
    ClientConnect,
    ClientPacket(ServerCommand),
    Debug(String),
    Spawn,
}

#[macro_export]
macro_rules! log_sync {
    ($source:expr, $x:expr) => {{
        // Load the logging enum variants locally.
        use $crate::log::LogSync::*;

        // Serialize body.
        let ron = ::ron::ser::to_string(&$x).unwrap();

        // Send value.
        $crate::log::SERVER_LOG_TX.log(($source).to_string(), ron);
    }};
}

#[macro_export]
macro_rules! log_raw {
    ($source:expr, $x:expr) => {{
        $crate::log::SERVER_LOG_TX.log(($source).to_string(), ($x).to_string());
    }};
}

pub fn log_sync_init(pool: DbPool) -> Option<DbPool> {
    SERVER_LOG_TX.replace_db_pool(pool)
}
