use bus::{Bus, BusReader};
use crate::{
    SyncClientCommand,
    SyncServerCommand,
    markdown::markdown_to_doc,
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

pub fn default_doc() -> Doc {
    const INPUT: &'static str = r#"

# Hello world!

This is edit-text, a web-based rich text editor.

* This is a very early preview.

* Supports collaborative editing.

* Written in Rust in the backend, cross-compiled to WebAssembly on the frontend.

* Supports Markdown export.

This app might be easy to break! That's okay though. We'll notice and fix it, and it'll break less in the future.

Type github.com/tcr/edit-text into your search bar for more information.

"#;

    // Should be no errors
    let doc = Doc(markdown_to_doc(&INPUT).unwrap());
    validate_doc(&doc).expect("Initial Markdown document was malformed");
    doc
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
    sync_state_mutex: Option<Arc<Mutex<SyncState>>>,
    out: Option<ws::Sender>,
    page_map: SharedPageMap,
    tx_db: CCSender<(String, String)>,
}

impl ws::Handler for ClientHandler {
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        let url = Url::parse("http://localhost/")
            .unwrap()
            .join(shake.request.resource())
            .unwrap();

        let query = url.query();
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

        self.sync_state_mutex = Some(allocate_page(
            &self.page_map,
            self.page_id.as_ref().unwrap(),
            query == Some("helloworld"),
            self.tx_db.clone(),
        ));

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

/// Creates a new page netry in the page map and spawns a sync
/// thread to manage it.
fn allocate_page(
    page_map_mutex: &SharedPageMap,
    page_id: &str,
    helloworld: bool,
    tx_db: CCSender<(String, String)>,
) -> Arc<Mutex<SyncState>> {
    {
        let mut page_map = page_map_mutex.lock().unwrap();

        if page_map.get(page_id).is_none() {
            page_map.insert(
                page_id.to_string(),
                Arc::new(Mutex::new(SyncState {
                    ops: VecDeque::new(),
                    version: 100,
                    history: hashmap![],
                    doc: if helloworld {
                        default_doc()
                    } else {
                        default_new_doc(page_id)
                    }, //default_doc(),
                    client_bus: Bus::new(255),
                })),
            );
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
    tx_db: CCSender<(String, String)>,
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
                        tx_db.try_send((page_id.to_string(), serialized))?;
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

type SharedPageMap = Arc<Mutex<HashMap<String, Arc<Mutex<SyncState>>>>>;

fn spawn_update_db(conn: SqliteConnection, rx_db: CCReceiver<(String, String)>) -> JoinHandle<()> {
    thread::spawn(move || {
        while let Ok((id, body)) = rx_db.recv() {
            // println!("(@) writing {:?}", id);
            create_post(&conn, &id, &body);
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

    let (tx_db, rx_db) = unbounded::<(String, String)>();
    spawn_update_db(db, rx_db);

    // When a page is started, we create a sync state mutex...
    let page_map: SharedPageMap = Arc::new(Mutex::new({
        let mut hash = HashMap::new();
        for (page_id, md) in original_pages {
            println!("(@) Restoring {:?}", page_id);
            if let Ok(doc) = ::ron::de::from_str(&md) {
                hash.insert(
                    page_id.to_string(),
                    Arc::new(Mutex::new(SyncState {
                        ops: VecDeque::new(),
                        version: 100,
                        history: hashmap![],
                        doc: Doc(doc),
                        client_bus: Bus::new(255),
                    })),
                );
            }
        }
        hash
    }));

    // Listen to incoming clients.
    let _ = ws::listen(url, {
        take!(=page_map, =tx_db);
        move |out| {
            log_sync!(ClientConnect);

            println!("Client connected.");

            // TODO how to select from unused client IDs?
            let new_client_id = thread_rng().gen_ascii_chars().take(6).collect::<String>();

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
