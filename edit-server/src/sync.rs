//! Synchronization server. Threads for websockets and graphql.

use crate::{
    db::*,
    carets::*,
    graphql::sync_graphql_server,
    state::*,
};

use extern::{
    crossbeam_channel::{
        Receiver as CCReceiver,
        Sender as CCSender,
        unbounded,
    },
    diesel::{
        sqlite::SqliteConnection,
    },
    failure::Error,
    edit_common::commands::*,
    oatie::{
        doc::*,
    },
    simple_ws::*,
    rand::{thread_rng, Rng},
    r2d2,
    r2d2_diesel::ConnectionManager,
    ron,
    serde_json,
    simple_ws,
    std::{
        collections::{HashMap, VecDeque},
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    },
    thread_spawn::thread_spawn,
    url::Url,
    ws,
};

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

const PAGE_TITLE_LEN: usize = 100;

pub fn default_new_doc(id: &str) -> Doc {
    Doc(doc_span![
        DocGroup({"tag": "h1"}, [
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
        .all(|x| x.is_digit(10) || x.is_ascii_alphabetic() || x == '_')
}

fn generate_random_page_id() -> String {
    thread_rng()
        .gen_ascii_chars()
        .take(6)
        .collect()
}

// Target Page ID, ClientNotification
struct ClientNotify(String, ClientNotification);

enum ClientNotification {
    Connect {
        client_id: String,
        out: simple_ws::Sender,
    },
    Commit {
        client_id: String,
        op: Op,
        version: usize,
    },
    Disconnect {
        client_id: String,
    },
}

/// Websocket handler for an individual user.
struct ClientSocket {
    page_id: String,
    client_id: String,
    tx: CCSender<ClientNotify>,
}

impl ClientSocket {
    fn new(
        client_id: &str,
        page_id: &str,
        tx: CCSender<ClientNotify>,
        out: simple_ws::Sender,
    ) -> Result<ClientSocket, Error> {
        // Notify sync thread of our connection.
        let _ = tx.send(ClientNotify(
            page_id.to_string(),
            ClientNotification::Connect {
                client_id: client_id.to_string(),
                out: out,
            }
        ));

        let socket = ClientSocket {
            page_id: page_id.to_string(),
            client_id: client_id.to_string(),
            tx,
        };

        Ok(socket)
    }


    fn handle_sync_command(
        &self,
        command: UserToSyncCommand,
    ) -> Result<(), Error> {
        log_sync!(ClientPacket(command.clone()));

        match command {
            UserToSyncCommand::Commit(client_id, op, version) => {
                let _ = self.tx.send(ClientNotify(
                    self.page_id.to_string(),
                    ClientNotification::Commit {
                        client_id,
                        op,
                        version,
                    },
                ));
                // let mut sync_state = self.sync_state_mutex.lock().unwrap();
                // sync_state.ops.push_back((client_id.clone(), version, op.clone()));
            }
            UserToSyncCommand::TerminateProxy => {
                // NOTE we ignore this, it's only used for user proxy
            }
        }

        Ok(())
    }

    fn cleanup_user(
        &self,
    ) -> Result<(), Error> {
        let _ = self.tx.send(ClientNotify(
            self.page_id.to_owned(),
            ClientNotification::Disconnect {
                client_id: self.client_id.to_owned(),
            }
        ));
        Ok(())
    }
}

/// Websocket implementation.
impl SimpleSocket for ClientSocket {
    type Args = (String, CCSender<ClientNotify>);

    fn initialize(
        (client_id, tx_master): Self::Args,
        url: &str,
        out: simple_ws::Sender,
    ) -> Result<ClientSocket, Error> {
        let url = Url::parse("http://localhost/")
            .unwrap()
            .join(url)
            .unwrap();
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

        println!(
            "(!) Client {:?} connected to {:?}",
            client_id, page_id
        );

        // Initialize this client.
        Ok(ClientSocket::new(&client_id, &page_id, tx_master, out)?)
    }

    fn handle_message(&mut self, data: &[u8]) -> Result<(), Error> {
        self.handle_sync_command(serde_json::from_slice(&data)?)
    }

    fn cleanup(&mut self) -> Result<(), Error> {
        self.cleanup_user()
    }
}

/// Run a sync server thread for a given page ID.
#[thread_spawn]
fn spawn_sync_thread(
    sync_state_mutex: SharedSyncThreadState,
    page_id: String,
    period: u64,
    db_pool: DbPool,
) -> Result<(), Error> {
    // Handle incoming packets.
    loop {
        // Wait a set duration between transforms.
        // Note that this is for artifically forcing a client-side queue of operations.
        // It's not needed for operation.
        thread::sleep(Duration::from_millis(period as u64));

        // let now = Instant::now();

        let mut sync_state = sync_state_mutex.lock().unwrap();

        // Go through the deque and update our operations.
        while let Some((client_id, input_version, op)) = sync_state.ops.pop_front() {
            // TODO we should evict the client if this fails.
            let op = sync_state.state.commit(
                &client_id,
                op,
                input_version,
            ).expect("Could not commit client operation.");

            // Updates the database with the new document version.
            if let Ok(doc) =
                remove_carets(&sync_state.state.doc)
            {
                let conn = db_pool.get().unwrap();
                create_page(&conn, &page_id, &doc);
            }

            // Broadcast this operation to all connected websockets.
            let command = SyncToUserCommand::Update(
                sync_state.state.doc.0.clone(),
                sync_state.state.version,
                client_id,
                op,
            );
            let json = serde_json::to_string(&command).unwrap();
            for item in &sync_state.client_bus_vec {
                let _ = item.lock().unwrap().send(json.clone());
            }
        }

        // let elapsed = now.elapsed();
        // println!("sync duration: {}s, {}us", elapsed.as_secs(), elapsed.subsec_nanos()/1_000);
    }
}

pub struct SyncThreadState {
    pub state: SyncState,
    pub ops: VecDeque<(String, usize, Op)>,
    pub client_bus_vec: Vec<simple_ws::Sender>,
}

// TODO unshared
type SharedSyncThreadState = Arc<Mutex<SyncThreadState>>;

struct PageMap {
    db_pool: DbPool,
    pages: HashMap<String, SharedSyncThreadState>,
}

impl PageMap {
    fn new(
        db_pool: DbPool,
    ) -> PageMap {
        PageMap {
            db_pool,
            pages: hashmap![],
        }
    }

    /// Creates a new page entry in the page map and spawns a sync
    /// thread to manage it.
    fn acquire_page(
        &mut self,
        page_id: &str,
    ) -> SharedSyncThreadState {
        // If this page doesn't exist, let's allocate a new thread for it.
        if self.pages.get(page_id).is_none() {
            println!("(%) loading new page for {:?}", page_id);
            
            // Retrieve from database, or use a default generic document.
            let conn = self.db_pool.get().unwrap();
            let inner_doc = get_single_page(&conn, page_id)
                .unwrap_or_else(|| default_new_doc(page_id));

            self.pages.insert(
                page_id.to_string(),
                Arc::new(Mutex::new(SyncThreadState {
                    state: SyncState::new(inner_doc, 100), // Arbitrarily select version 100
                    ops: VecDeque::new(),
                    client_bus_vec: vec![],
                })),
            );
        } else {
            eprintln!("(%) reloading {:?}", page_id);
        }

        let shared_sync_state = self.pages.get(page_id)
            .map(|x| x.clone()).unwrap();

        let _ = spawn_sync_thread(
            shared_sync_state.clone(),
            page_id.to_owned(),
            100, // TODO pass in a real _period value from command line arguments
            self.db_pool.clone(),
        );

        shared_sync_state
    }
}

// TODO make this coordinate properly with 
#[thread_spawn]
fn spawn_master_thread(
    db_pool: DbPool,
    rx_master: CCReceiver<ClientNotify>,
) {
    let mut page_map = PageMap::new(db_pool);

    while let Ok(ClientNotify(page_id, notification)) = rx_master.recv() {
        let shared_sync_state = page_map.acquire_page(&page_id);

        match notification {
            ClientNotification::Connect {
                client_id, out
            } => {
                let mut sync_state = shared_sync_state.lock().unwrap();
                let version = sync_state.state.version;

                // Initialize client state on outgoing websocket.
                let command = SyncToUserCommand::Init(
                    client_id.to_string(),
                    sync_state.state.doc.0.clone(),
                    version,
                );
                out.lock().unwrap()
                    .send(serde_json::to_string(&command).unwrap()).unwrap();

                // Register with clients list.
                sync_state.state.clients.insert(client_id.to_string(), version);
                
                sync_state.client_bus_vec.push(out);
            },
            ClientNotification::Commit {
                client_id,
                op,
                version,
            } => {
                let mut sync_state = shared_sync_state.lock().unwrap();
                sync_state.ops.push_back((client_id.clone(), version, op));
            },
            ClientNotification::Disconnect {
                client_id
            } => {
                let mut sync_state = shared_sync_state.lock().unwrap();

                // Todo 
                let op = remove_carets_op(&sync_state.state.doc, vec![client_id.clone()]).unwrap();
                let version = sync_state.state.version;
                sync_state.ops.push_back((client_id.clone(), version, op));

                sync_state.state.clients.remove(&client_id);
            },
        }
    }
}

// TODO use _period
pub fn sync_socket_server(port: u16, _period: usize) {
    log_sync!(Spawn);

    let db_pool = db_pool_create();

    // Start the GraphQL server.
    ::std::thread::spawn({
        take!(=db_pool);
        move || {
            sync_graphql_server(db_pool);
        }
    });

    // Spawn master coordination thread.
    let (tx_master, rx_master) = unbounded::<ClientNotify>();
    spawn_master_thread(db_pool.clone(), rx_master);

    // Websocket URL.
    let url = format!("0.0.0.0:{}", port);
    eprintln!("sync_socket_server is listening for ws connections on {}", url);

    // Start the WebSocket listener.
    let _ = ws::listen(url, {
        take!(=tx_master);
        move |out| {
            log_sync!(ClientConnect);

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
