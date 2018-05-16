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
    failure::Error,
    edit_common::commands::*,
    oatie::{
        doc::*,
    },
    simple_ws::*,
    rand::{thread_rng, Rng},
    ron,
    serde_json,
    simple_ws,
    std::{
        collections::HashMap,
        thread,
        time::Duration,
    },
    thread_spawn::thread_spawn,
    url::Url,
    ws,
};

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

// Target Page ID, ClientUpdate
pub struct ClientNotify(String, ClientUpdate);

pub enum ClientUpdate {
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

        eprintln!(
            "(!) Client {:?} connected to {:?}",
            client_id, page_id
        );

        // Notify sync thread of our connection.
        let _ = tx_master.send(ClientNotify(
            page_id.to_string(),
            ClientUpdate::Connect {
                client_id: client_id.to_string(),
                out: out,
            }
        ));

        // Preserve client state.
        Ok(ClientSocket {
            page_id: page_id.to_string(),
            client_id: client_id.to_string(),
            tx_master,
        })
    }

    fn handle_message(&mut self, data: &[u8]) -> Result<(), Error> {
        let command: UserToSyncCommand = serde_json::from_slice(&data)?;

        log_sync!(ClientPacket(command.clone()));

        match command {
            UserToSyncCommand::Commit(client_id, op, version) => {
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
            UserToSyncCommand::TerminateProxy => {
                // NOTE we ignore this, it's only used for user proxy
            }
        }

        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), Error> {
        self.tx_master.send(ClientNotify(
            self.page_id.to_owned(),
            ClientUpdate::Disconnect {
                client_id: self.client_id.to_owned(),
            }
        ))?;

        Ok(())
    }
}

pub struct PageController {
    page_id: String,
    db_pool: DbPool,
    state: SyncState,
    clients: Vec<simple_ws::Sender>,
}

impl PageController {
    fn sync_commit(
        &mut self,
        client_id: &str,
        op: Op,
        input_version: usize,
    ) {
        // TODO we should evict the client if this fails.
        let op = self.state.commit(
            &client_id,
            op,
            input_version,
        ).expect("Could not commit client operation.");

        // Updates the database with the new document version.
        if let Ok(doc) =
            remove_carets(&self.state.doc)
        {
            let conn = self.db_pool.get().unwrap();
            create_page(&conn, &self.page_id, &doc);
        }

        // Broadcast this operation to all connected websockets.
        let command = SyncToUserCommand::Update(
            self.state.doc.0.clone(),
            self.state.version,
            client_id.to_owned(),
            op,
        );
        let json = serde_json::to_string(&command).unwrap();
        for item in &self.clients {
            let _ = item.lock().unwrap().send(json.clone());
        }
    }

    // Handle a client's update.
    fn handle(
        &mut self,
        notification: ClientUpdate,
    ) {
        match notification {
            ClientUpdate::Connect {
                client_id, out
            } => {
                let version = self.state.version;

                // Initialize client state on outgoing websocket.
                let command = SyncToUserCommand::Init(
                    client_id.to_string(),
                    self.state.doc.0.clone(),
                    version,
                );
                out.lock().unwrap()
                    .send(serde_json::to_string(&command).unwrap()).unwrap();

                // Register with clients list.
                self.state.clients.insert(client_id.to_string(), version);
                
                self.clients.push(out);
            },

            ClientUpdate::Disconnect {
                client_id
            } => {
                // Todo 
                let op = remove_carets_op(&self.state.doc, vec![client_id.clone()]).unwrap();
                let version = self.state.version;
                self.sync_commit(&client_id, op, version);

                self.state.clients.remove(&client_id);
            },

            ClientUpdate::Commit {
                client_id,
                op,
                version,
            } => {
                // Commit the operation.
                self.sync_commit(&client_id, op, version);
            },
        }
    }
}

/// Run a sync server thread for a given page ID.
#[thread_spawn]
pub fn spawn_sync_thread(
    page_id: String,
    rx_notify: CCReceiver<ClientUpdate>,
    inner_doc: Doc,
    period: u64,
    db_pool: DbPool,
) -> Result<(), Error> {
    let mut sync = PageController {
        page_id,
        db_pool,
        state: SyncState::new(inner_doc, 100), // Arbitrarily select version 100
        clients: vec![],
    };

    while let Ok(notification) = rx_notify.recv() {
        // let now = Instant::now()

        // Wait a set duration between transforms.
        // Note that this is for artifically forcing a client-side queue of operations.
        // It's not needed for operation.
        thread::sleep(Duration::from_millis(period as u64));

        sync.handle(notification);

        // let elapsed = now.elapsed();
        // println!("sync duration: {}s, {}us", elapsed.as_secs(), elapsed.subsec_nanos()/1_000);
    }

    Ok(())
}

struct PageMaster {
    db_pool: DbPool,
    pages: HashMap<String, CCSender<ClientUpdate>>,
}

impl PageMaster {
    fn new(
        db_pool: DbPool,
    ) -> PageMaster {
        PageMaster {
            db_pool,
            pages: hashmap![],
        }
    }

    /// Creates a new page entry in the page map and spawns a sync
    /// thread to manage it.
    fn acquire_page(
        &mut self,
        page_id: &str,
    ) -> CCSender<ClientUpdate> {
        // If this page doesn't exist, let's allocate a new thread for it.
        if self.pages.get(page_id).is_none() {
            println!("(%) loading new page for {:?}", page_id);
            
            // Retrieve from database, or use a default generic document.
            let conn = self.db_pool.get().unwrap();
            let inner_doc = get_single_page(&conn, page_id)
                .unwrap_or_else(|| default_new_doc(page_id));

            let (tx_notify, rx_notify) = unbounded();
            self.pages.insert(
                page_id.to_string(),
                tx_notify.clone(),
            );

            spawn_sync_thread(
                page_id.to_owned(),
                rx_notify,
                inner_doc,
                100, // TODO pass in a real _period value from command line arguments
                self.db_pool.clone(),
            );
            tx_notify
        } else {
            self.pages.get(page_id).map(|x| x.clone()).unwrap()
        }
    }
}

// TODO make this coordinate properly with 
#[thread_spawn]
fn spawn_page_master(
    db_pool: DbPool,
    rx_master: CCReceiver<ClientNotify>,
) {
    let mut page_map = PageMaster::new(db_pool);

    while let Ok(ClientNotify(page_id, notification)) = rx_master.recv() {
        let _ = page_map.acquire_page(&page_id).send(notification);
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
    spawn_page_master(db_pool.clone(), rx_master);

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
