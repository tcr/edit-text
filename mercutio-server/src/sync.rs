use bus::{Bus, BusReader};
use crate::{
    SyncClientCommand,
    SyncServerCommand,
    db::*,
    util::*,
};
use crossbeam_channel::{
    Receiver as CCReceiver,
    Sender as CCSender,
    unbounded,
};
use diesel::{
    sqlite::SqliteConnection,
};
use failure::Error;
use oatie::{
    OT,
    doc::*,
    schema::RtfSchema,
    validate::validate_doc,
};
use rand::{thread_rng, Rng};
use ron;
use serde_json;
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};
use url::Url;
use ws;

const PAGE_TITLE_LEN: usize = 100;

pub fn default_new_doc(id: &str) -> Doc {
    Doc(doc_span![
        DocGroup({"tag": "h1"}, [
            DocChars(id),
        ])
    ])
}

/// Transform an operation incrementally against each interim document operation.
pub fn update_operation(
    mut op: Op,
    history: &HashMap<usize, Op>,
    target_version: usize,
    mut input_version: usize,
) -> Op {
    // Transform against each interim operation.
    // TODO upgrade_operation_to_current or something
    while input_version < target_version {
        // If the version exists (it should) transform against it.
        if let Some(ref version_op) = history.get(&input_version) {
            let (updated_op, _) = Op::transform::<RtfSchema>(version_op, &op);
            op = updated_op;
        }

        input_version += 1;
    }
    op
}

pub fn valid_page_id(input: &str) -> bool {
    if input.is_empty() || input.len() > PAGE_TITLE_LEN {
        return false;
    }
    input
        .chars()
        .all(|x| x.is_digit(10) || x.is_ascii_alphabetic() || x == '_')
}

/// Fanout messages from a bus to a websocket sender.
fn spawn_bus_to_client(
    out: ws::Sender,
    mut rx: BusReader<SyncClientCommand>,
) -> JoinHandle<Result<(), Error>> {
    thread::spawn(move || -> Result<(), Error> {
        while let Ok(command) = rx.recv() {
            out.send(serde_json::to_string(&command).unwrap())?;
        }
        Ok(())
    })
}

pub struct SyncState {
    ops: VecDeque<(String, usize, Op)>,
    version: usize,
    history: HashMap<usize, Op>,
    doc: Doc,
    client_bus: Bus<SyncClientCommand>,
}

struct ClientHandler {
    client_id: String,
    page_id: Option<String>,
    sync_state_mutex: Option<SharedSyncState>,
    out: Option<ws::Sender>,
    page_map: SharedPageMap,
    tx_db: CCSender<DbMessage>,
}

impl ws::Handler for ClientHandler {
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        let url = Url::parse("http://localhost/")
            .unwrap()
            .join(shake.request.resource())
            .unwrap();

        let mut path = url.path().to_owned();

        if path.starts_with("/$/ws/") {
            path = path["/$/ws".len()..].to_string();
        }

        if valid_page_id(&path[1..]) {
            self.page_id = Some(path[1..].to_string());
        } else {
            // TODO actually bail out
            self.page_id = Some("home".to_string());
        }

        println!(
            "(!) Client {:?} connected to {:?}",
            self.client_id, self.page_id
        );

        // HERE we want to synchronize state to something
        // on first startup

        let (tx_client, rx_client) = unbounded();
        self.tx_db.send(DbMessage::Initialize {
            id: self.page_id.clone().unwrap(),
            client: self.client_id.clone(),
            receiver: tx_client,
        });

        if let Ok(sync_state_mutex) = rx_client.recv() {
            self.sync_state_mutex = Some(sync_state_mutex);
        } else {
            panic!("Expected a sync state bundle.");
        }

        // We will consume the out map.
        let out = self.out.take().unwrap();

        // Forcibly set this new client's initial document state.
        {
            let sync_state_mutex = self.sync_state_mutex.clone().unwrap();
            let mut sync_state = sync_state_mutex.lock().unwrap();

            let command = SyncClientCommand::Init(
                self.client_id.clone(),
                sync_state.doc.0.clone(),
                sync_state.version,
            );
            out.send(serde_json::to_string(&command).unwrap())?;

            // Forward packets from sync bus to all clients.
            let rx = { sync_state.client_bus.add_rx() };
            spawn_bus_to_client(out, rx);
        }

        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        let req_parse: Result<SyncServerCommand, _> = serde_json::from_slice(&msg.into_data());
        match req_parse {
            Ok(value) => {
                log_sync!(ClientPacket(value.clone()));

                match value {
                    SyncServerCommand::Keepalive => {
                        // noop
                    }
                    SyncServerCommand::Commit(client_id, op, version) => {
                        let mut sync_state_mutex = self.sync_state_mutex.clone().unwrap();
                        let mut sync_state = sync_state_mutex.lock().unwrap();
                        sync_state.ops.push_back((client_id, version, op));
                    }
                }
            }
            Err(err) => {
                println!("Packet error: {:?}", err);
            }
        }

        Ok(())
    }
}

/// Creates a new page entry in the page map and spawns a sync
/// thread to manage it.
fn allocate_page(
    db: &SqliteConnection,
    page_map_mutex: &SharedPageMap,
    page_id: &str,
    tx_db: CCSender<DbMessage>,
) -> SharedSyncState {
    {
        let mut page_map = page_map_mutex.lock().unwrap();

        if page_map.get(page_id).is_none() {
            println!("(%) writing new page for {:?}", page_id);
            let inner_doc = get_single_page(&db, page_id)
                .map(|x| Doc(::ron::de::from_str::<DocSpan>(&x.body).unwrap()))
                .unwrap_or_else(|| default_new_doc(page_id));

            page_map.insert(
                page_id.to_string(),
                Arc::new(Mutex::new(SyncState {
                    ops: VecDeque::new(),
                    version: 100,
                    history: hashmap![],
                    doc: inner_doc,
                    client_bus: Bus::new(255),
                })),
            );
        } else {
            println!("(%) launching {:?}", page_id);
        }
    }

    // TODO pass in a real _period value
    spawn_sync_server(page_map_mutex, page_id, 100, tx_db)
        .expect("Failed to spawn sync server");

    {
        let page_map = page_map_mutex.lock().unwrap();
        page_map.get(page_id).clone().unwrap().clone()
    }
}

/// Run a sync server thread for a given page ID.
fn spawn_sync_server(
    page_map: &SharedPageMap,
    page_id: &str,
    period: u64,
    tx_db: CCSender<DbMessage>,
) -> Result<JoinHandle<Result<(), Error>>, ::std::io::Error> {
    let page_map = page_map.clone();
    let page_id = page_id.to_string();
    thread::Builder::new()
        .name("sync_thread_processor".into())
        .spawn(move || -> Result<(), Error> {
            let sync_state_mutex = page_map
                .lock()
                .unwrap()
                .get(&page_id)
                .clone()
                .unwrap()
                .clone();
                
            // Handle incoming packets.
            loop {
                // Wait a set duration between transforms.
                thread::sleep(Duration::from_millis(period as u64));

                // let now = Instant::now();

                let mut sync_state = sync_state_mutex.lock().unwrap();

                // Go through the deque and update our operations.
                while let Some((client_id, input_version, op)) = sync_state.ops.pop_front() {
                    let target_version = sync_state.version;

                    // log_sync!(Debug(format!("client {:?} sent {:?}", client_id, op)));

                    // let res = action_sync(&doc, new_op, op_group).unwrap();

                    // Update the operation so we can apply it to the document.
                    let op =
                        update_operation(op, &sync_state.history, target_version, input_version);
                    sync_state.history.insert(target_version, op.clone());

                    // log_sync!(Debug(format!("updated op to {:?}", op)));

                    // Update the document with this operation.
                    sync_state.doc = Op::apply(&sync_state.doc, &op);
                    sync_state.version = target_version + 1;

                    validate_doc(&sync_state.doc).expect("Validation error");

                    // log_sync!(Debug(format!("doc is now {:?}", sync_state.doc)));

                    // if let Ok(md) = doc_to_markdown(&sync_state.doc.0) {
                    if let Ok(serialized) =
                        remove_carets(&sync_state.doc).and_then(|x| Ok(::ron::ser::to_string(&x.0)?))
                    {
                        tx_db.try_send(DbMessage::Update {
                            id: page_id.to_string(),
                            body: serialized,
                        })?;
                    }
                    // }

                    // Broadcast to all connected websockets.
                    let command = SyncClientCommand::Update(
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

type SharedSyncState = Arc<Mutex<SyncState>>;
type SharedPageMap = Arc<Mutex<HashMap<String, SharedSyncState>>>;

enum DbMessage {
    // Updated the document
    Update {
        id: String,
        body: String,
    },

    // Intiialize a client.
    Initialize {
        id: String,
        client: String,
        receiver: CCSender<SharedSyncState>,
    },
}

fn spawn_update_db(
    conn: SqliteConnection,
    page_map_mutex: SharedPageMap,
    tx_db: CCSender<DbMessage>,
    rx_db: CCReceiver<DbMessage>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while let Ok(message) = rx_db.recv() {
            // println!("(@) writing {:?}", id);
            match message {
                DbMessage::Update { id, body } => {
                    create_post(&conn, &id, &body);
                }
                DbMessage::Initialize { id, client, receiver } => {
                    let page_map = page_map_mutex.lock().unwrap().get(&id).map(|x| x.clone());

                    let shared_sync_state =
                        if let Some(value) = page_map {
                            println!("(%) reloading {:?}", id);
                            value.clone()
                        } else {
                            allocate_page(
                                &conn,
                                &page_map_mutex.clone(),
                                &id,
                                tx_db.clone(),
                            )
                        };

                    receiver.send(shared_sync_state);
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

    let db = db_connection();
    let original_pages = all_posts(&db);

    // When a page is started, we create a sync state mutex...
    let page_map: SharedPageMap = Arc::new(Mutex::new(HashMap::new()));

    let (tx_db, rx_db) = unbounded::<DbMessage>();
    spawn_update_db(db, page_map.clone(), tx_db.clone(), rx_db);

    // Listen to incoming clients.
    let _ = ws::listen(url, {
        take!(=page_map, =tx_db);
        move |out| {
            log_sync!(ClientConnect);

            println!("Client connected.");

            // TODO how to select from unused client IDs?
            let new_client_id: String = thread_rng()
                .gen_ascii_chars()
                .take(6)
                .collect();

            // Listen to commands from the clients and submit to sync server.
            ClientHandler {
                client_id: new_client_id,
                page_id: None,
                sync_state_mutex: None, //sync_state_mutex.clone(),
                out: Some(out),
                page_map: page_map.clone(),
                tx_db: tx_db.clone(),
            }
        }
    });
}
