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

struct ClientSocket {
    client_id: String,
    sync_state_mutex: SharedSyncState,
}

impl ClientSocket {
    fn initialize(&self, out: Arc<Mutex<ws::Sender>>) -> Result<(), Error> {
        let mut sync_state = self.sync_state_mutex.lock().unwrap();
        let version = sync_state.version;

        // Initialize client state on outgoing websocket.
        let command = SyncToUserCommand::Init(
            self.client_id.clone(),
            sync_state.doc.0.clone(),
            version,
        );
        out.lock().unwrap()
            .send(serde_json::to_string(&command).unwrap())?;

        // Register with clients list.
        sync_state.clients.insert(self.client_id.clone(), version);

        // Forward packets from sync bus to all clients.
        let rx = { sync_state.client_bus.add_rx() };
        ClientSocket::spawn_bus_to_client(out, rx);

        Ok(())
    }

    /// Fanout messages from a bus to a websocket sender.
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
}

impl SimpleSocket for ClientSocket {
    type Args = (String, CCSender<DbMessage>);

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

        // Create the client sockets.
        let (tx_client, rx_client) = unbounded();

        // Ask the db to initialize us with the sync state bundle.
        let _ = tx_db.send(DbMessage::Initialize {
            id: page_id.clone(),
            receiver: tx_client,
        });
        let sync_state_mutex = rx_client.recv().expect("Expected a sync state bundle.");

        // Initialize this client.
        let client = ClientSocket {
            client_id,
            sync_state_mutex,
        };
        client.initialize(out)?;

        Ok(client)
    }

    fn handle_message(&mut self, data: &[u8]) -> Result<(), Error> {
        let msg: UserToSyncCommand = serde_json::from_slice(&data)?;

        log_sync!(ClientPacket(msg.clone()));

        match msg {
            UserToSyncCommand::Commit(client_id, op, version) => {
                let mut sync_state = self.sync_state_mutex.lock().unwrap();
                sync_state.ops.push_back((client_id.clone(), version, op.clone()));
            }
            UserToSyncCommand::TerminateProxy => {
                // ignore this, only for proxy
            }
        }

        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), Error> {
        let mut sync_state = self.sync_state_mutex.lock().unwrap();

        let op = remove_carets_op(&sync_state.doc, vec![self.client_id.clone()])?;
        let version = sync_state.version;
        sync_state.ops.push_back((self.client_id.clone(), version, op));

        sync_state.clients.remove(&self.client_id);

        Ok(())
    }
}


type SharedSyncState = Arc<Mutex<SyncState>>;
type PageMap = HashMap<String, SharedSyncState>;

enum DbMessage {
    // Updated the document
    Update {
        id: String,
        body: Doc,
    },

    // Intiialize a client with a page ID.
    Initialize {
        id: String,
        receiver: CCSender<SharedSyncState>,
    },
}

/// Creates a new page entry in the page map and spawns a sync
/// thread to manage it.
fn allocate_page(
    db: &SqliteConnection,
    page_map: &mut PageMap,
    page_id: &str,
    tx_db: CCSender<DbMessage>,
) -> SharedSyncState {
    if page_map.get(page_id).is_none() {
        println!("(%) writing new page for {:?}", page_id);
        
        // Retrieve from database, or use a default generic document.
        let inner_doc = get_single_page(&db, page_id)
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

    let shared_sync_state = page_map.get(page_id).clone().unwrap().clone();

    // TODO pass in a real _period value
    spawn_sync_server(shared_sync_state.clone(), page_id, 100, tx_db)
        .expect("Failed to spawn sync server");

    shared_sync_state
}

/// Run a sync server thread for a given page ID.
fn spawn_sync_server(
    sync_state_mutex: SharedSyncState,
    page_id: &str,
    period: u64,
    tx_db: CCSender<DbMessage>,
) -> Result<JoinHandle<Result<(), Error>>, ::std::io::Error> {
    let page_id = page_id.to_string();
    thread::Builder::new()
        .name("sync_thread_processor".into())
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

                    if let Ok(doc) =
                        remove_carets(&sync_state.doc)
                    {
                        tx_db.try_send(DbMessage::Update {
                            id: page_id.to_string(),
                            body: doc,
                        })?;
                    }

                    // Broadcast to all connected websockets.
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
}

fn spawn_update_db(
    db_pool: DbPool,
    tx_db: CCSender<DbMessage>,
    rx_db: CCReceiver<DbMessage>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut page_map: PageMap = HashMap::new();

        while let Ok(message) = rx_db.recv() {
            // println!("(@) writing {:?}", id);
            match message {
                DbMessage::Update { id, body } => {
                    let conn = db_pool.get().unwrap();
                    create_page(&conn, &id, &body);
                }
                DbMessage::Initialize { id, receiver } => {
                    let shared_sync_state =
                        if let Some(value) = page_map.get(&id) {
                            println!("(%) reloading {:?}", id);
                            value.clone()
                        } else {
                            let conn = db_pool.get().unwrap();
                            allocate_page(
                                &conn,
                                &mut page_map,
                                &id,
                                tx_db.clone(),
                            )
                        };

                    let _ = receiver.send(shared_sync_state);
                }
            }
        }
    })
}

// TODO use _period
pub fn sync_socket_server(port: u16, _period: usize) {
    log_sync!(Spawn);

    let url = format!("0.0.0.0:{}", port);

    println!("Listening sync_socket_server on 0.0.0.0:{}", port);

    let db_pool = db_pool_create();

    let (tx_db, rx_db) = unbounded::<DbMessage>();
    spawn_update_db(db_pool.clone(), tx_db.clone(), rx_db);

    // Start the sync GraphQL server.
    ::std::thread::spawn({
        take!(=db_pool);
        move || {
            sync_graphql_server(db_pool);
        }
    });

    // Start the sync WebSocket server.
    let _ = ws::listen(url, {
        take!(=tx_db);
        move |out| {
            log_sync!(ClientConnect);

            println!("Client connected.");

            // TODO how to select from unused client IDs?
            let new_client_id: String = thread_rng()
                .gen_ascii_chars()
                .take(6)
                .collect();

            // Listen to commands from the clients and submit to sync server.            
            SocketHandler::<ClientSocket>::new((new_client_id, tx_db.clone()), out)
        }
    });
}
