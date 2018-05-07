#![allow(deprecated)]

use crate::{
    db::*,
    util::*,
    graphql::sync_graphql_server,
    state::*,
};

use extern::{
    bus::{Bus, BusReader},
    crossbeam_channel::{
        Receiver as CCReceiver,
        Sender as CCSender,
        unbounded,
    },
    diesel::{
        sqlite::SqliteConnection,
    },
    failure::Error,
    mercutio_common::{
        SyncToUserCommand,
        UserToSyncCommand,
    },
    oatie::{
        doc::*,
    },
    simple_ws::*,
    rand::{thread_rng, Rng},
    r2d2,
    r2d2_diesel::ConnectionManager,
    ron,
    serde_json,
    std::{
        collections::{HashMap, VecDeque},
        sync::{Arc, Mutex},
        thread::{self, JoinHandle},
        time::Duration,
    },
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

/// Websocket handler for an individual user.
struct ClientSocket {
    client_id: String,
    sync_state_mutex: SharedSyncState,
}

impl ClientSocket {
    fn new(
        client_id: &str,
        sync_state_mutex: SharedSyncState,
        out: Arc<Mutex<ws::Sender>>,
    ) -> Result<ClientSocket, Error> {
        let socket = ClientSocket {
            client_id: client_id.to_string(),
            sync_state_mutex,
        };

        {
            let mut sync_state = socket.sync_state_mutex.lock().unwrap();
            let version = sync_state.version;

            // Initialize client state on outgoing websocket.
            let command = SyncToUserCommand::Init(
                client_id.to_string(),
                sync_state.doc.0.clone(),
                version,
            );
            out.lock().unwrap()
                .send(serde_json::to_string(&command).unwrap())?;

            // Register with clients list.
            sync_state.clients.insert(client_id.to_string(), version);

            // Forward packets from sync bus to all clients.
            let rx = { sync_state.client_bus.add_rx() };
            ClientSocket::spawn_bus_to_client(out, rx);
        }

        Ok(socket)
    }

    /// Fanout messages from a bus to a websocket sender.
    // #[spawn]
    fn spawn_bus_to_client(
        out: Arc<Mutex<ws::Sender>>,
        mut rx: BusReader<SyncToUserCommand>,
    ) -> JoinHandle<Result<(), Error>> {
        thread::spawn(move || -> Result<(), Error> {
            while let Ok(command) = rx.recv() {
                out.lock().unwrap().send(serde_json::to_string(&command).unwrap())?;
            }
            Ok(())
        })
    }

    fn handle_sync_command(
        &self,
        command: UserToSyncCommand,
    ) -> Result<(), Error> {
        log_sync!(ClientPacket(command.clone()));

        match command {
            UserToSyncCommand::Commit(client_id, op, version) => {
                let mut sync_state = self.sync_state_mutex.lock().unwrap();
                sync_state.ops.push_back((client_id.clone(), version, op.clone()));
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
        let mut sync_state = self.sync_state_mutex.lock().unwrap();

        let op = remove_carets_op(&sync_state.doc, vec![self.client_id.clone()])?;
        let version = sync_state.version;
        sync_state.ops.push_back((self.client_id.clone(), version, op));

        sync_state.clients.remove(&self.client_id);

        Ok(())
    }
}

/// Websocket implementation.
impl SimpleSocket for ClientSocket {
    type Args = (String, CCSender<Initialize>);

    fn initialize(
        (client_id, tx_db): Self::Args,
        url: &str,
        out: Arc<Mutex<ws::Sender>>,
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

        // Fetch the sync state mutex for this client.
        let sync_state_mutex = {
            let (tx_client, rx_client) = unbounded();
            let _ = tx_db.send(Initialize {
                id: page_id.clone(),
                receiver: tx_client,
            });
            rx_client.recv()
                .expect("Expected a sync state bundle.")
        };

        // Initialize this client.
        Ok(ClientSocket::new(&client_id, sync_state_mutex, out)?)
    }

    fn handle_message(&mut self, data: &[u8]) -> Result<(), Error> {
        self.handle_sync_command(serde_json::from_slice(&data)?)
    }

    fn cleanup(&mut self) -> Result<(), Error> {
        self.cleanup_user()
    }
}


type SharedSyncState = Arc<Mutex<SyncState>>;

struct Initialize {
    id: String,
    receiver: CCSender<SharedSyncState>,
}

struct PageMap {
    db_pool: DbPool,
    pages: HashMap<String, SharedSyncState>,
}

impl PageMap {
    fn new(
        db_pool: DbPool,
    ) -> PageMap {
        PageMap {
            db_pool,
            pages: HashMap::new(),
        }
    }

    fn acquire_page(
        &mut self,
        page_id: &str
    ) -> SharedSyncState {
        match self.pages.get(page_id) {
            Some(state) => {
                eprintln!("(%) reloading {:?}", page_id);
                state.clone()
            }
            None => self.allocate_page(page_id)
        }
    }

    /// Creates a new page entry in the page map and spawns a sync
    /// thread to manage it.
    fn allocate_page(
        &mut self,
        page_id: &str,
    ) -> SharedSyncState {
        let page_map = &mut self.pages; // TODO just inline this reference

        // See if this page exists, or if we have to start it.
        //TODO is the else { ... } path ever even taken?
        if page_map.get(page_id).is_none() {
            println!("(%) writing new page for {:?}", page_id);
            
            // Retrieve from database, or use a default generic document.
            let conn = self.db_pool.get().unwrap();
            let inner_doc = get_single_page(&conn, page_id)
                .unwrap_or_else(|| default_new_doc(page_id));

            page_map.insert(
                page_id.to_string(),
                Arc::new(Mutex::new(SyncState {
                    ops: VecDeque::new(),
                    version: 100,
                    clients: hashmap![],
                    history: hashmap![],
                    doc: inner_doc,
                    client_bus: Bus::new(255),
                })),
            );
        } else {
            println!("(%) launching {:?}", page_id);
        }

        let shared_sync_state = page_map.get(page_id).map(|x| x.clone()).unwrap();

        let _ = spawn_sync_thread(
            shared_sync_state.clone(),
            page_id,
            100, // TODO pass in a real _period value
            self.db_pool.clone(),
        );

        shared_sync_state
    }
}

/// Run a sync server thread for a given page ID.
// #[spawn]
fn spawn_sync_thread(
    sync_state_mutex: SharedSyncState,
    page_id: &str,
    period: u64,
    db_pool: DbPool,
) -> JoinHandle<Result<(), Error>> {
    let page_id = page_id.to_string();
    thread::Builder::new()
        .name("spawn_sync_thread".into())
        .spawn(move || -> Result<(), Error> {
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
                    let op = sync_state.commit(
                        &client_id,
                        op,
                        input_version,
                    ).expect("Could not commit client operation.");

                    // Updates the database with the new document version.
                    if let Ok(doc) =
                        remove_carets(&sync_state.doc)
                    {
                        let conn = db_pool.get().unwrap();
                        create_page(&conn, &page_id, &doc);
                    }

                    // Broadcast this operation to all connected websockets.
                    let command = SyncToUserCommand::Update(
                        sync_state.doc.0.clone(),
                        sync_state.version,
                        client_id,
                        op,
                    );
                    sync_state.client_bus.broadcast(command);
                }

                // let elapsed = now.elapsed();
                // println!("sync duration: {}s, {}us", elapsed.as_secs(), elapsed.subsec_nanos()/1_000);
            }
        })
        .expect("could not spawn thread")
}

// #[spawn]
// TODO make this coordinate properly with 
fn spawn_master_thread(
    db_pool: DbPool,
    rx_db: CCReceiver<Initialize>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut page_map = PageMap::new(db_pool);

        while let Ok(Initialize { id, receiver }) = rx_db.recv() {
            let shared_sync_state = page_map.acquire_page(&id);
            let _ = receiver.send(shared_sync_state);
        }
    })
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
    let (tx_db, rx_db) = unbounded::<Initialize>();
    spawn_master_thread(db_pool.clone(), rx_db);

    // Websocket URL.
    let url = format!("0.0.0.0:{}", port);
    eprintln!("sync_socket_server is listening for ws connections on {}", url);

    // Start the WebSocket listener.
    let _ = ws::listen(url, {
        take!(=tx_db);
        move |out| {
            log_sync!(ClientConnect);

            eprintln!("Client connected.");

            // Listen to commands from the clients and submit to sync server.            
            SocketHandler::<ClientSocket>::new(
                (
                    generate_random_page_id(), // TODO can we select from unused client IDs?
                    tx_db.clone(),
                ),
                out,
            )
        }
    });
}
