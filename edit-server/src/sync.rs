//! Synchronization server. Threads for websockets and graphql.

use crate::{
    carets::*,
    db::*,
    graphql::sync_graphql_server,
    log::log_sync_init,
    state::*,
};

use crossbeam_channel::{
    unbounded,
    Receiver as CCReceiver,
    Sender as CCSender,
};
use edit_common::commands::*;
use edit_common::simple_ws;
use edit_common::simple_ws::*;
use failure::Error;
use oatie::doc::*;
use oatie::rtf::*;
use rand::{
    thread_rng,
    Rng,
};
use serde_json;
use std::env;
use std::{
    collections::HashMap,
    thread,
    time::Duration,
};
use url::Url;
use ws;

fn debug_sync_delay() -> Option<u64> {
    env::var("EDIT_DEBUG_SYNC_DELAY")
        .ok()
        .and_then(|x| x.parse::<u64>().ok())
}

const INITIAL_SYNC_VERSION: usize = 100; // Arbitrarily select version 100
const PAGE_TITLE_LEN: usize = 100; // 100 chars is the limit

pub fn default_new_doc(id: &str) -> Doc<RtfSchema> {
    Doc(doc_span![
        DocGroup(Attrs::Header(1), [
            DocChars(id),
        ])
    ])
}

pub fn valid_page_id(input: &str) -> bool {
    if input.is_empty() || input.len() > PAGE_TITLE_LEN {
        return false;
    }
    input
        .chars()
        .all(|x| x.is_digit(10) || x.is_ascii_alphabetic() || x == '_' || x == '-')
}

fn generate_random_page_id() -> String {
    thread_rng().gen_ascii_chars().take(6).collect()
}

// Target Page ID, ClientUpdate
pub struct ClientNotify(pub String, pub ClientUpdate);

// TODO rename this PageUpdate
pub enum ClientUpdate {
    Connect {
        client_id: String,
        out: simple_ws::Sender,
    },
    Commit {
        client_id: String,
        op: Op<RtfSchema>,
        version: usize,
    },
    Disconnect {
        client_id: String,
    },
    Overwrite {
        doc: Doc<RtfSchema>,
    },
}

/// Websocket handler for an individual user.
struct ClientSocket {
    page_id: String,
    client_id: String,
    tx_master: CCSender<ClientNotify>,
}

/// Websocket implementation.
impl SimpleSocket for ClientSocket {
    type Args = (String, CCSender<ClientNotify>);

    fn initialize(
        (client_id, tx_master): Self::Args,
        url: &str,
        out: simple_ws::Sender,
    ) -> Result<ClientSocket, Error> {
        let url = Url::parse("http://localhost/").unwrap().join(url).unwrap();
        let mut path = url.path().to_owned();

        if path.starts_with("/$/ws/") {
            path = path["/$/ws".len()..].to_string();
        }

        let page_id = if valid_page_id(&path[1..]) {
            path[1..].to_string()
        } else {
            // TODO actually bail out, how?
            "home".to_string()
        };

        eprintln!("(!) Client {:?} connected to {:?}", client_id, page_id);

        // Notify sync thread of our having connected.
        let _ = tx_master.send(ClientNotify(
            page_id.to_string(),
            ClientUpdate::Connect {
                client_id: client_id.to_string(),
                out: out,
            },
        ));

        // Store client state in a ClientSocket.
        Ok(ClientSocket {
            page_id: page_id.to_string(),
            client_id: client_id.to_string(),
            tx_master,
        })
    }

    fn handle_message(&mut self, data: &[u8]) -> Result<(), Error> {
        let command: ServerCommand = serde_json::from_slice(&data)?;

        // TODO don't log client Log(...)
        // log_sync!("SERVER", ClientPacket(command.clone()));
        // println!("-----> {:?}", command);

        match command {
            ServerCommand::Commit(client_id, op, version) => {
                let _ = self.tx_master.send(ClientNotify(
                    self.page_id.to_string(),
                    ClientUpdate::Commit {
                        client_id,
                        op,
                        version,
                    },
                ));
                // let mut sync_state = self.sync_state_mutex.lock().unwrap();
                // sync_state.ops.push_back((client_id.clone(), version, op.clone()));
            }
            ServerCommand::TerminateProxy => {
                // NOTE we ignore this, it's only used for user proxy
            }
            ServerCommand::Log(log) => {
                log_raw!(self.client_id, log);
            }
        }

        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), Error> {
        self.tx_master.send(ClientNotify(
            self.page_id.to_owned(),
            ClientUpdate::Disconnect {
                client_id: self.client_id.to_owned(),
            },
        ));

        Ok(())
    }
}

pub struct PageController {
    page_id: String,
    db_pool: DbPool,
    state: SyncState,
    clients: HashMap<String, simple_ws::Sender>,
}

#[allow(unused)]
impl PageController {
    // This is just a commit across all operations, and forwarding it to
    // all listening clients. It also is the commit point for all new
    // operations.
    fn sync_commit(&mut self, client_id: &str, op: Op<RtfSchema>, input_version: usize) {
        // TODO we should evict the client if this fails.
        let op = self
            .state
            .commit(&client_id, op, input_version)
            .expect("Could not commit client operation.");

        // Updates the database with the new document version.
        if let Ok(doc) = remove_carets(&self.state.doc) {
            let conn = self.db_pool.get().unwrap();
            // TODO why is this "create" page
            create_page(&conn, &self.page_id, &doc);
        }

        // Broadcast this operation to all connected websockets.
        let command = ClientCommand::Update(self.state.version, client_id.to_owned(), op);
        self.broadcast_client_command(&command);
    }

    /// Forward command to everyone in our client set.
    fn broadcast_client_command(&self, command: &ClientCommand) {
        let json = serde_json::to_string(&command).unwrap();
        for (_, client) in &self.clients {
            let _ = client.lock().unwrap().send(json.clone());
        }
    }

    fn send_client_command(
        &self,
        client: &simple_ws::Sender,
        command: &ClientCommand,
    ) -> Result<(), Error> {
        let json = serde_json::to_string(&command).unwrap();
        Ok(client.lock().unwrap().send(json.clone())?)
    }

    fn send_client_restart(&self, client_id: &str) -> Result<(), Error> {
        let code = ws::CloseCode::Restart;
        let reason = "Server received an updated version of the document.";

        // TODO abort if client doesn't exist, or move the client_id referencing
        // to its own function
        self.clients.get(client_id).map(|client| {
            let _ = client.lock().unwrap().close_with_reason(code, reason);
        });
        Ok(())
    }

    /// Forward restart code to everyone in our client set.
    fn broadcast_restart(&self) -> Result<(), Error> {
        let code = ws::CloseCode::Restart;
        let reason = "Server received an updated version of the document.";
        for (_, client) in &self.clients {
            let _ = client.lock().unwrap().close_with_reason(code, reason);
        }
        Ok(())
    }

    // Handle a client's update.
    fn handle(&mut self, notification: ClientUpdate) {
        match notification {
            ClientUpdate::Connect { client_id, out } => {
                let version = self.state.version;

                // Initialize client state on outgoing websocket.
                let command =
                    ClientCommand::Init(client_id.to_string(), self.state.doc.0.clone(), version);
                let _ = self.send_client_command(&out, &command);

                // Register with clients list.
                self.state.clients.insert(client_id.to_string(), version);

                // Forward to all in our client set.
                self.clients.insert(client_id.to_string(), out);
            }

            ClientUpdate::Disconnect { client_id } => {
                // Remove our caret from document.
                let op = remove_carets_op(&self.state.doc, vec![client_id.clone()]).unwrap();
                let version = self.state.version;
                self.sync_commit(&client_id, op, version);

                // Remove from our client set.
                self.state.clients.remove(&client_id);
                self.clients.remove(&client_id);
            }

            ClientUpdate::Commit {
                client_id,
                op,
                version,
            } => {
                // Debug setting to wait a set duration between successive notifications.
                // This is helpful for artifically forcing a client-side queue of operations.
                // It's not needed for operation though.
                if let Some(delay) = debug_sync_delay() {
                    thread::sleep(Duration::from_millis(delay));
                }

                // Commit the operation.
                // TODO remove this AssertUnwindSafe, since it's probably not safe.
                let sync = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    self.sync_commit(&client_id, op, version);
                }));

                if let Err(err) = sync {
                    eprintln!(
                        "received invalid packet from client: {:?} - {:?}",
                        client_id, err
                    );
                    // let _ = self.send_client_restart(&client_id);
                }
            }

            ClientUpdate::Overwrite { doc } => {
                let _ = self.broadcast_restart();

                // Rewrite our state.
                self.state = SyncState::new(doc, INITIAL_SYNC_VERSION);
                self.clients = HashMap::new();
            }
        }
    }
}

/// Run a sync server thread for a given page ID.
pub fn spawn_sync_thread(
    page_id: String,
    rx_notify: CCReceiver<ClientUpdate>,
    inner_doc: Doc<RtfSchema>,
    db_pool: DbPool,
) -> Result<(), Error> {
    thread::spawn(move || {
        // This page ID's state.
        // TODO make this a ::new(...) statement
        let mut sync = PageController {
            page_id,
            db_pool,
            state: SyncState::new(inner_doc, INITIAL_SYNC_VERSION),
            clients: HashMap::new(),
        };

        while let Some(notification) = rx_notify.recv() {
            // let now = Instant::now()

            // TODO with need to listen for errors and break the loop if erorrs occurr
            // (killin the sync thread).
            sync.handle(notification);

            // let elapsed = now.elapsed();
            // println!("sync duration: {}s, {}us", elapsed.as_secs(), elapsed.subsec_nanos()/1_000);
        }
    });
    Ok(())
}

struct PageMaster {
    db_pool: DbPool,
    pages: HashMap<String, CCSender<ClientUpdate>>,
}

impl PageMaster {
    fn new(db_pool: DbPool) -> PageMaster {
        PageMaster {
            db_pool,
            pages: hashmap![],
        }
    }

    /// Creates a new page entry in the page map and spawns a sync
    /// thread to manage it.
    fn acquire_page(&mut self, page_id: &str) -> CCSender<ClientUpdate> {
        // If this page doesn't exist, let's allocate a new thread for it.
        if self.pages.get(page_id).is_none() {
            println!("(%) loading new page for {:?}", page_id);

            // Retrieve from database, or use a default generic document.
            let conn = self.db_pool.get().unwrap();
            let inner_doc =
                get_single_page(&conn, page_id).unwrap_or_else(|| default_new_doc(page_id));

            let (tx_notify, rx_notify) = unbounded();
            self.pages.insert(page_id.to_string(), tx_notify.clone());

            // We ignore all errors from the sync thread, and thus the whole thread.
            let _ = spawn_sync_thread(
                page_id.to_owned(),
                rx_notify,
                inner_doc,
                self.db_pool.clone(),
            );
            tx_notify
        } else {
            self.pages.get(page_id).map(|x| x.clone()).unwrap()
        }
    }
}

// TODO make this coordinate properly with
fn spawn_page_master(db_pool: DbPool, rx_master: CCReceiver<ClientNotify>) {
    thread::spawn(move || {
        let mut page_map = PageMaster::new(db_pool);

        while let Some(ClientNotify(page_id, notification)) = rx_master.recv() {
            let _ = page_map.acquire_page(&page_id).send(notification);
        }
    });
}

// TODO use _period
pub fn sync_socket_server(port: u16) {
    let db_pool = db_pool_create();

    // Start recorder.
    log_sync_init(db_pool.clone());

    log_sync!("SERVER", Spawn);

    // Spawn master coordination thread.
    let (tx_master, rx_master) = unbounded::<ClientNotify>();
    spawn_page_master(db_pool.clone(), rx_master);

    // Start the GraphQL server.
    ::std::thread::spawn({
        take!(=db_pool, =tx_master);
        move || {
            sync_graphql_server(db_pool, tx_master);
        }
    });

    // Websocket URL.
    let url = format!("0.0.0.0:{}", port);
    eprintln!(
        "  Sync server is listening for WebSocket connections on port {}",
        port
    );

    // Start the WebSocket listener.
    let _ = ws::listen(url, {
        take!(=tx_master);
        move |out| {
            log_sync!("SERVER", ClientConnect);

            eprintln!("Client connected.");

            // Listen to commands from the clients and submit to sync server.
            SocketHandler::<ClientSocket>::new(
                (
                    generate_random_page_id(), // TODO can we select from unused client IDs?
                    tx_master.clone(),
                ),
                out,
            )
        }
    });
}
