#![allow(deprecated)]

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
use ws::util::{Token, Timeout};
use ws::{CloseCode, Frame};

const PAGE_TITLE_LEN: usize = 100;

pub fn default_new_doc(id: &str) -> Doc {
    Doc(doc_span![
        DocGroup({"tag": "h1"}, [
            DocChars(id),
        ])
    ])
}

/// Transform an operation incrementally against each interim document operation.
// TODO upgrade_operation_to_current or something
pub fn update_operation(
    client_id: &str,
    mut op: Op,
    history: &HashMap<usize, Op>,
    target_version: usize,
    mut input_version: usize,
) -> Op {
    // Transform against each interim operation.
    while input_version < target_version {
        // If the version exists (it should) transform against it.
        let version_op = history.get(&input_version)
            .expect("Version missing from history");
        let (updated_op, _) = Op::transform::<RtfSchema>(version_op, &op);
        op = updated_op;

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
    out: Arc<Mutex<ws::Sender>>,
    mut rx: BusReader<SyncClientCommand>,
) -> JoinHandle<Result<(), Error>> {
    thread::spawn(move || -> Result<(), Error> {
        while let Ok(command) = rx.recv() {
            out.lock().unwrap().send(serde_json::to_string(&command).unwrap())?;
        }
        Ok(())
    })
}

pub struct SyncState {
    ops: VecDeque<(String, usize, Op)>,
    version: usize,
    clients: HashMap<String, usize>, // client_id -> version
    history: HashMap<usize, Op>,
    doc: Doc,
    client_bus: Bus<SyncClientCommand>,
}

impl SyncState {
    fn prune_history(&mut self) {
        if let Some(min_version) = self.clients.iter().map(|(_, &v)| v).min() {
            for k in self.history.keys().cloned().collect::<Vec<usize>>() {
                if k < min_version {
                    // eprintln!("(^) evicted document version {}", k);
                    self.history.remove(&k);
                }
            }
        }
    }
}

struct ClientHandler {
    client_id: String,
    page_id: Option<String>,
    sync_state_mutex: Option<SharedSyncState>,
    out: Arc<Mutex<ws::Sender>>,
    tx_db: CCSender<DbMessage>,
    timeout: Option<Timeout>,
    closed: bool,
}

const PING: Token = Token(1);
const EXPIRE: Token = Token(2);

const PING_INTERVAL: u64 = 5_000;
const TIMEOUT_INTERVAL: u64 = 30_000;

impl Drop for ClientHandler {
    fn drop(&mut self) {
        assert!(self.closed);
    }
}

impl ClientHandler {
    fn initialize(&mut self) -> ws::Result<()> {
        println!(
            "(!) Client {:?} connected to {:?}",
            self.client_id, self.page_id
        );

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

        // Forcibly set this new client's initial document state.
        {
            let out = self.out.clone();
            let sync_state_mutex = self.sync_state_mutex.clone().unwrap();
            let mut sync_state = sync_state_mutex.lock().unwrap();
            let version = sync_state.version;

            // Initialize server with client
            let command = SyncClientCommand::Init(
                self.client_id.clone(),
                sync_state.doc.0.clone(),
                version,
            );
            out.lock().unwrap().send(serde_json::to_string(&command).unwrap())?;

            // Add to clients list.
            sync_state.clients.insert(self.client_id.clone(), version);

            // Forward packets from sync bus to all clients.
            let rx = { sync_state.client_bus.add_rx() };
            spawn_bus_to_client(out, rx);
        }

        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), Error> {
        assert!(!self.closed);

        let sync_state_mutex = self.sync_state_mutex.clone().unwrap();
        let mut sync_state = sync_state_mutex.lock().unwrap();

        let op = remove_carets_op(&sync_state.doc, vec![self.client_id.clone()])?;
        let version = sync_state.version;
        sync_state.ops.push_back((self.client_id.clone(), version, op));

        sync_state.clients.remove(&self.client_id);

        self.closed = true;

        Ok(())
    }

    fn handle_message(&mut self, message: SyncServerCommand) {
        log_sync!(ClientPacket(message.clone()));

        match message {
            SyncServerCommand::Commit(client_id, op, version) => {
                let mut sync_state_mutex = self.sync_state_mutex.clone().unwrap();
                let mut sync_state = sync_state_mutex.lock().unwrap();
                sync_state.ops.push_back((client_id.clone(), version, op.clone()));
            }
        }
    }
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
            // TODO actually bail out, how?
            self.page_id = Some("home".to_string());
        }

        {
            let out = self.out.lock().unwrap();

            // schedule a timeout to send a ping every 5 seconds
            try!(out.timeout(PING_INTERVAL, PING));
            // schedule a timeout to close the connection if there is no activity for 30 seconds
            try!(out.timeout(TIMEOUT_INTERVAL, EXPIRE));
        }

        self.initialize()
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        let req_parse: Result<SyncServerCommand, _> = serde_json::from_slice(&msg.into_data());
        match req_parse {
            Ok(value) => {
                self.handle_message(value);
            }
            Err(err) => {
                println!("Packet error: {:?}", err);
            }
        }

        Ok(())
    }

    fn on_shutdown(&mut self) {
        eprintln!("-----> SHUTDOWN");
        self.cleanup();
    }

    fn on_close(&mut self, _code: ws::CloseCode, reason: &str) {
        eprintln!("-----> CLOSE");
        self.cleanup();
    }

    fn on_error(&mut self, err: ws::Error) {
        eprintln!("-----> ERROR");
        self.cleanup();
    }

    fn on_timeout(&mut self, event: Token) -> ws::Result<()> {
        match event {
            PING => {
                let out = self.out.lock().unwrap();
                out.ping(vec![])?;
                out.timeout(PING_INTERVAL, PING)
            }
            EXPIRE => {
                println!("----> Socket Expired");
                self.out.lock().unwrap().close(CloseCode::Away)
            }
            _ => Err(ws::Error::new(ws::ErrorKind::Internal, "Invalid timeout token encountered!")),
        }
    }

    fn on_new_timeout(&mut self, event: Token, timeout: Timeout) -> ws::Result<()> {
        if event == EXPIRE {
            if let Some(t) = self.timeout.take() {
                try!(self.out.lock().unwrap().cancel(t))
            }
            self.timeout = Some(timeout)
        }
        Ok(())
    }

    fn on_frame(&mut self, frame: Frame) -> ws::Result<Option<Frame>> {
        // some activity has occurred, let's reset the expiration
        try!(self.out.lock().unwrap().timeout(TIMEOUT_INTERVAL, EXPIRE));
        Ok(Some(frame))
    }
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
            .map(|x| Doc(::ron::de::from_str::<DocSpan>(&x.body).unwrap()))
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
                thread::sleep(Duration::from_millis(period as u64));

                // let now = Instant::now();

                let mut sync_state = sync_state_mutex.lock().unwrap();

                // Go through the deque and update our operations.
                while let Some((client_id, input_version, op)) = sync_state.ops.pop_front() {
                    let target_version = sync_state.version;

                    // log_sync!(Debug(format!("client {:?} sent {:?}", client_id, op)));

                    // let res = action_sync(&doc, new_op, op_group).unwrap();

                    // Update the operation so we can apply it to the document.
                    let op = update_operation(
                        &client_id,
                        op,
                        &sync_state.history,
                        target_version,
                        input_version
                    );

                    if let Some(version) = sync_state.clients.get_mut(&client_id) {
                        *version = target_version;
                    } else {
                        // TODO what circumstances would it be missing? Client closed
                        // and removed itself from list but operation used later?
                    }

                    // Prune history entries.
                    sync_state.prune_history();
                    sync_state.history.insert(target_version, op.clone());

                    // log_sync!(Debug(format!("updated op to {:?}", op)));

                    // Update the document with this operation.
                    sync_state.doc = Op::apply(&sync_state.doc, &op);
                    sync_state.version = target_version + 1;

                    validate_doc(&sync_state.doc).expect("Validation error");

                    // log_sync!(Debug(format!("doc is now {:?}", sync_state.doc)));

                    // if let Ok(md) = doc_to_markdown(&sync_state.doc.0) {
                    if let Ok(serialized) =
                        remove_carets(&sync_state.doc)
                            .and_then(|x| Ok(::ron::ser::to_string(&x.0)?))
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
type PageMap = HashMap<String, SharedSyncState>;

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
    tx_db: CCSender<DbMessage>,
    rx_db: CCReceiver<DbMessage>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut page_map: PageMap = HashMap::new();

        while let Ok(message) = rx_db.recv() {
            // println!("(@) writing {:?}", id);
            match message {
                DbMessage::Update { id, body } => {
                    create_post(&conn, &id, &body);
                }
                DbMessage::Initialize { id, client, receiver } => {
                    let shared_sync_state =
                        if let Some(value) = page_map.get(&id) {
                            println!("(%) reloading {:?}", id);
                            value.clone()
                        } else {
                            allocate_page(
                                &conn,
                                &mut page_map,
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

    let (tx_db, rx_db) = unbounded::<DbMessage>();
    spawn_update_db(db, tx_db.clone(), rx_db);

    // Listen to incoming clients.
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
            ClientHandler {
                client_id: new_client_id,
                page_id: None,
                sync_state_mutex: None, //sync_state_mutex.clone(),
                out: Arc::new(Mutex::new(out)),
                tx_db: tx_db.clone(),
                timeout: None,
                closed: false,
            }
        }
    });
}
