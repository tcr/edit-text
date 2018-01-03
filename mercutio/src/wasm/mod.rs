pub mod actions;
pub mod walkers;

use failure::Error;
use oatie::{OT, Operation, debug_pretty};
use oatie::doc::*;
use rand;
use rand::Rng;
use self::actions::*;
use super::sync::{SyncServerCommand, SyncClientCommand};
use serde_json;
use std::{panic, process};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use ws;
use crossbeam_channel::{unbounded, Sender};



// Commands to send back to native.
#[derive(Serialize, Deserialize, Debug)]
pub enum NativeCommand {
    // Connect(String),
    Keypress(u32, bool, bool),
    Button(u32),
    Character(u32),
    RenameGroup(String, CurSpan),
    // Load(DocSpan),
    Target(CurSpan),
    Monkey(bool),
}

// Commands to send to JavaScript.
#[derive(Serialize, Deserialize, Debug)]
pub enum ClientCommand {
    Setup {
        keys: Vec<(u32, bool, bool)>,
        buttons: Vec<(usize, String)>,
    },
    PromptString(String, String, NativeCommand),
    Update(DocSpan, Option<Op>, usize),
    Error(String),
}

fn client_op<C>(client: &Client, callback: C) -> Result<(), Error>
where
    C: Fn(ActionContext) -> Result<Op, Error>,
{
    let mut doc = client.doc.lock().unwrap();

    let client_id = client.name.lock().unwrap().clone().unwrap().to_string();
    let op = callback(ActionContext {
        doc: doc.clone(),
        client_id,
    })?;

    // Apply new operation.
    let new_doc = OT::apply(&*doc, &op);

    let original_doc = client.original_doc.lock().unwrap();
    let mut original_ops = client.original_ops.lock().unwrap();
    original_ops.push(op.clone());

    // println!("ORIGINAL: {:?}", *original_doc);
    let mut check_op_a = op_span!([], []);
    for (i, op) in original_ops.iter().enumerate() {
        // println!("  {}: applying {:?}/{:?}", client.name, i + 1, original_ops.len());
        // println!("  {} 1️⃣: let op_left = op_span!{:?};", client.name, check_op_a);
        // println!("  {} 1️⃣: let op_right = op_span!{:?};", client.name, op);
        check_op_a = Operation::compose(&check_op_a, &op);
        // println!("  {} 1️⃣: let res = op_span!{:?};", client.name, check_op_a);
        // println!("  {} 1️⃣: let original = doc_span!{:?};", client.name, *original_doc);
        // println!("  {} 1️⃣: let latest_doc = doc_span!{:?};", client.name, *doc);
        let _ = OT::apply(&*original_doc, &check_op_a);
    }

    *doc = new_doc;
    assert_eq!(OT::apply(&*original_doc, &check_op_a), *doc);

    // Send update.
    let version = client.version.load(Ordering::Relaxed);
    let res = ClientCommand::Update(doc.0.clone(), Some(op.clone()), version);
    client.send(&res)?;

    // Send operation to sync server.
    client.tx.send((op, version));

    Ok(())
}

fn key_handlers() -> Vec<(u32, bool, bool, Box<Fn(&Client) -> Result<(), Error>>)> {
    vec![
        // backspace
        (
            8,
            false,
            false,
            Box::new(|client: &Client| {
                println!("backspace");
                client_op(client, |doc| delete_char(doc))
            }),
        ),
        // left
        (
            37,
            false,
            false,
            Box::new(|client: &Client| client_op(client, |doc| caret_move(doc, false))),
        ),
        // right
        (
            39,
            false,
            false,
            Box::new(|client: &Client| client_op(client, |doc| caret_move(doc, true))),
        ),
        // up
        (
            38,
            false,
            false,
            Box::new(|client: &Client| client_op(client, |doc| caret_block_move(doc, false))),
        ),
        // down
        (
            40,
            false,
            false,
            Box::new(|client: &Client| client_op(client, |doc| caret_block_move(doc, true))),
        ),
        // enter
        (
            13,
            false,
            false,
            Box::new(|client: &Client| client_op(client, |doc| split_block(doc))),
        ),
    ]
}

fn button_handlers() -> Vec<(&'static str, Box<Fn(&Client) -> Result<(), Error>>)> {
    vec![
        (
            "Heading 1",
            Box::new(|client: &Client| client_op(client, |doc| replace_block(doc, "h1"))),
        ),
        (
            "Heading 2",
            Box::new(|client: &Client| client_op(client, |doc| replace_block(doc, "h2"))),
        ),
        (
            "Heading 3",
            Box::new(|client: &Client| client_op(client, |doc| replace_block(doc, "h3"))),
        ),
        (
            "Paragraph",
            Box::new(|client: &Client| client_op(client, |doc| replace_block(doc, "p"))),
        ),
        (
            "Code",
            Box::new(|client: &Client| client_op(client, |doc| replace_block(doc, "pre"))),
        ),
        (
            "List",
            Box::new(|client: &Client| client_op(client, |doc| toggle_list(doc))),
        ),
    ]
}

fn native_command(client: &Client, req: NativeCommand) -> Result<(), Error> {
    match req {
        // NativeCommand::Connect(client_id) => {
        //     *client.name.lock().unwrap() = Some(client_id);
        // }
        NativeCommand::RenameGroup(tag, _) => client_op(client, |doc| replace_block(doc, &tag))?,
        NativeCommand::Button(index) => {
            // Find which button handler to respond to this command.
            button_handlers()
                .get(index as usize)
                .map(|handler| handler.1(client));
        }
        NativeCommand::Keypress(key_code, meta_key, shift_key) => {
            println!("key: {:?} {:?} {:?}", key_code, meta_key, shift_key);

            // Find which key handler to process this command.
            for command in key_handlers() {
                if command.0 == key_code && command.1 == meta_key && command.2 == shift_key {
                    command.3(client)?;
                    break;
                }
            }
        }
        NativeCommand::Character(char_code) => client_op(client, |doc| add_char(doc, char_code))?,
        NativeCommand::Target(cur) => {
            client_op(client, |doc| cur_to_caret(doc, &cur))?;
            *client.target.lock().unwrap() = Some(cur);
        }
        // NativeCommand::Load(doc) => {
        //     let mut client_doc = client.doc.lock().unwrap();
        //     *client_doc = Doc(doc.clone());

        //     *client.original_doc.lock().unwrap() = Doc(doc.clone());
        //     *client.original_ops.lock().unwrap() = vec![];

        //     let next_version = client.version.load(Ordering::Relaxed) + 1;
        //     client.version.store(next_version, Ordering::Relaxed);
        //     println!("Bumped version to {:?}", next_version);

        //     // Native drives client state.
        //     let res = ClientCommand::Update(doc.clone(), None, next_version);
        //     client.send(&res)?;

        //     // Drop mutex.
        //     // TODO this probably isn't necessary, but we shoudl version
        //     // doc and version in same mutex
        //     drop(client_doc);

        //     // Load the caret.
        //     if !client.first_load.load(Ordering::Relaxed) {
        //         client.first_load.store(true, Ordering::Relaxed);

        //         client_op(client, |doc| init_caret(doc))?;
        //     }
        // }
        NativeCommand::Monkey(setting) => {
            client.monkey.store(setting, Ordering::Relaxed);
        }
    }
    Ok(())
}

struct Client {
    out: ws::Sender,
    doc: Mutex<Doc>,
    original_doc: Mutex<Doc>,
    original_ops: Mutex<Vec<Op>>,
    first_load: AtomicBool,
    //TODO remove the target field? base only on carets instead
    target: Mutex<Option<CurSpan>>,
    monkey: AtomicBool,
    name: Mutex<Option<String>>,
    version: AtomicUsize,
    alive: AtomicBool,
    tx: Sender<(Op, usize)>,
}

impl Client {
    fn send(&self, req: &ClientCommand) -> Result<(), Error> {
        let json = serde_json::to_string(&req)?;
        self.out.send(json)?;
        Ok(())
    }
}

type MonkeyParam = (u64, u64, u64);

// "Human-like"
const MONKEY_BUTTON: MonkeyParam = (500, 0, 2000);
const MONKEY_LETTER: MonkeyParam = (50, 0, 200);
const MONKEY_ARROW: MonkeyParam = (0, 0, 500);
const MONKEY_BACKSPACE: MonkeyParam = (0, 0, 400);
const MONKEY_ENTER: MonkeyParam = (600, 0, 3_000);

// Race
// const MONKEY_BUTTON: MonkeyParam = (0, 0, 100);
// const MONKEY_LETTER: MonkeyParam = (0, 0, 100);
// const MONKEY_ARROW: MonkeyParam = (0, 0, 100);
// const MONKEY_BACKSPACE: MonkeyParam = (0, 0, 100);
// const MONKEY_ENTER: MonkeyParam = (0, 0, 1_000);

fn monkey_wait(input: MonkeyParam) {
    let mut rng = rand::thread_rng();
    thread::sleep(Duration::from_millis(input.0 + rng.gen_range(input.1, input.2)));
}

#[allow(unused)]
fn setup_monkey(client: Arc<Client>) {
    // Button monkey.
    let thread_client: Arc<_> = client.clone();
    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        while thread_client.alive.load(Ordering::Relaxed) {
            monkey_wait(MONKEY_BUTTON);
            if thread_client.monkey.load(Ordering::Relaxed) {
                rng.choose(&button_handlers()).map(|button| {
                    button.1(&*thread_client);
                });
            }
        }
    });

    // Letter monkey.
    let thread_client: Arc<_> = client.clone();
    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        while thread_client.alive.load(Ordering::Relaxed) {
            monkey_wait(MONKEY_LETTER);
            if thread_client.monkey.load(Ordering::Relaxed) {
                let char_list = vec![
                            rng.gen_range(b'A', b'Z'),
                            rng.gen_range(b'a', b'z'),
                            rng.gen_range(b'0', b'9'),
                            b' ',
                        ];
                native_command(
                    &*thread_client,
                    NativeCommand::Character(*rng
                        .choose(&char_list)
                        .unwrap() as _),
                );
            }
        }
    });

    // Arrow keys.
    let thread_client: Arc<_> = client.clone();
    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        while thread_client.alive.load(Ordering::Relaxed) {
            monkey_wait(MONKEY_ARROW);
            if thread_client.monkey.load(Ordering::Relaxed) {
                let key = *rng.choose(&[37, 39, 37, 39, 37, 39, 38, 40]).unwrap();
                native_command(
                    &*thread_client,
                    NativeCommand::Keypress(key, false, false),
                );
            }
        }
    });

    // Backspace monkey.
    let thread_client: Arc<_> = client.clone();
    thread::spawn(move || {
        while thread_client.alive.load(Ordering::Relaxed) {
            monkey_wait(MONKEY_BACKSPACE);
            if thread_client.monkey.load(Ordering::Relaxed) {
                native_command(
                    &*thread_client,
                    NativeCommand::Keypress(8, false, false),
                );
            }
        }
    });

    // Enter monkey.
    let thread_client: Arc<_> = client.clone();
    thread::spawn(move || loop {
        while thread_client.alive.load(Ordering::Relaxed) {
            monkey_wait(MONKEY_ENTER);
            if thread_client.monkey.load(Ordering::Relaxed) {
                native_command(&*thread_client, NativeCommand::Keypress(13, false, false));
            }
        }
    });
}

struct SocketHandler {
    client: Arc<Client>,
}

impl ws::Handler for SocketHandler {
    fn on_open(&mut self, shake: ws::Handshake) -> Result<(), ws::Error> {
        let client_id = shake.request.resource()[1..].to_string();
        *self.client.name.lock().unwrap() = Some(client_id);
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> Result<(), ws::Error> {
        // Handle messages received on this connection
        println!("wasm got a packet from client '{}'. ", msg);

        let req_parse: Result<NativeCommand, _> = serde_json::from_slice(&msg.into_data());
        match req_parse {
            Err(err) => {
                println!("Packet error: {:?}", err);
            }
            Ok(value) => {
                native_command(&self.client, value).expect("Native command error");
            }
        }

        Ok(())
    }

    fn on_error(&mut self, err: ws::Error) {
        println!("Killing after error");
        self.client.monkey.store(false, Ordering::Relaxed);
        self.client.alive.store(false, Ordering::Relaxed);
    }

    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        println!("Killing after close");
        self.client.monkey.store(false, Ordering::Relaxed);
        self.client.alive.store(false, Ordering::Relaxed);
    }
}

pub fn server(url: &str) {
    ws::listen(url, |out| {
        let (tx, rx) = unbounded();

        let mut client = Arc::new(Client {
            out,
            doc: Mutex::new(Doc(vec![])),
            
            original_doc: Mutex::new(Doc(vec![])),
            original_ops: Mutex::new(vec![]),

            first_load: AtomicBool::new(false),
            target: Mutex::new(None),
            monkey: AtomicBool::new(false),
            name: Mutex::new(None),
            version: AtomicUsize::new(100),
            alive: AtomicBool::new(true),

            tx,
        });

        // Send initial setup packet.
        client
            .send(&ClientCommand::Setup {
                keys: key_handlers()
                    .into_iter()
                    .map(|x| (x.0, x.1, x.2))
                    .collect(),
                buttons: button_handlers()
                    .into_iter()
                    .enumerate()
                    .map(|(i, x)| (i, x.0.to_string()))
                    .collect(),
            })
            .expect("Could not send initial state");

        // Setup monkey tasks.
        setup_monkey(client.clone());

        let client_capture = client.clone();
        thread::spawn(move || {
            ws::connect("ws://127.0.0.1:3010", |out| {
                // out.send(serde_json::to_string(&SyncServerCommand::Connect("left".to_string())).unwrap()).unwrap();

                // Send over operations
                let rx_capture = rx.clone();
                let client_capture_capture = client_capture.clone();
                thread::spawn(move || {
                    while let Ok((packet, version)) = rx_capture.recv() {
                        let name = client_capture_capture.name.lock().unwrap().clone().unwrap();
                        // let cur_version = client_capture_capture.version.load(Ordering::Relaxed);
                        out.send(serde_json::to_string(&SyncServerCommand::Commit(
                            name,
                            packet,
                            version
                        )).unwrap()).unwrap();
                    }
                });

                let client_capture_capture = client_capture.clone();
                move |msg: ws::Message| {
                    // Handle messages received on this connection
                    println!("wasm got a packet from sync '{}'. ", msg);

                    let req_parse: Result<SyncClientCommand, _> = serde_json::from_slice(&msg.into_data());
                    match req_parse {
                        Err(err) => {
                            println!("Packet error: {:?}", err);
                        }
                        Ok(value) => {
                            match value {
                                SyncClientCommand::Update(doc, version) => {
                                    let mut client_doc = client_capture_capture.doc.lock().unwrap();
                                    *client_doc = Doc(doc.clone());

                                    *client_capture_capture.original_doc.lock().unwrap() = Doc(doc.clone());
                                    *client_capture_capture.original_ops.lock().unwrap() = vec![];

                                    client_capture_capture.version.store(version, Ordering::Relaxed);
                                    println!("new version is {:?}", version);

                                    // Native drives client state.
                                    let res = ClientCommand::Update(doc.clone(), None, version);
                                    client_capture_capture.send(&res).unwrap();

                                    // Drop mutex.
                                    // TODO this probably isn't necessary, but we shoudl version
                                    // doc and version in same mutex
                                    drop(client_doc);

                                    // Load the caret.
                                    if !client_capture_capture.first_load.load(Ordering::Relaxed) {
                                        client_capture_capture.first_load.store(true, Ordering::Relaxed);

                                        let v = client_capture_capture.version.load(Ordering::Relaxed);
                                        client_capture_capture.version.store(0, Ordering::Relaxed);
                                        client_op(&*client_capture_capture, |doc| init_caret(doc)).unwrap();
                                        client_capture_capture.version.store(v, Ordering::Relaxed);
                                    }
                                }
                            }
                            // native_command(&client, value).expect("Native command error");
                        }
                    }

                    Ok(())
                }
            }).unwrap();
        });


        // Websocket message handler.
        SocketHandler {
            client,
        }
    }).unwrap();
}

pub fn start_websocket_server() {
    thread::spawn(|| {
        if let Err(value) = panic::catch_unwind(|| {
            server("127.0.0.1:3012");
        }) {
            println!("Error: {:?}", value);
            process::exit(1);
        }
    });
}
