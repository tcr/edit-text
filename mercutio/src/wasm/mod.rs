pub mod actions;
pub mod walkers;

use failure::Error;
use oatie::{debug_pretty, Operation, OT};
use oatie::doc::*;
use rand;
use rand::Rng;
use self::actions::*;
use super::sync::{SyncClientCommand, SyncServerCommand};
use serde_json;
use std::{panic, process};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use ws;
use crossbeam_channel::{unbounded, Receiver, Sender};

macro_rules! clone_all {
    ( $( $x:ident ),* ) => {
        $(let $x = $x.clone();)*
    };
}

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
    Update(String, Option<Op>),
    Error(String),
}

struct Client {
    name: Arc<Mutex<Option<String>>>,

    doc: Doc,
    version: usize,

    original_doc: Doc,
    original_ops: Vec<Op>,

    first_load: bool,
    monkey: Arc<AtomicBool>,
    alive: Arc<AtomicBool>,

    out: ws::Sender,
    tx: Sender<SyncServerCommand>,
}

impl Client {
    fn send(&self, req: &ClientCommand) -> Result<(), Error> {
        let json = serde_json::to_string(&req)?;
        self.out.send(json)?;
        Ok(())
    }
}



// // Creates an HTML tree from a document tree.
// function docToStrings(ret: Array<string>, vec: Array<any>) {
//   // TODO act like doc
//   // console.log(el);
//   // var h = newElem(el.DocGroup[0]);
//   for (var g = 0; g < vec.length; g++) {
//     const el = vec[g];
//     if (el.DocGroup) {
//       const attrs = el.DocGroup[0];
//       ret.push(`<div
//         data-tag=${JSON.stringify(String(attrs.tag))}
//         data-client=${JSON.stringify(String(attrs.client))}
//         class=${JSON.stringify(String(attrs.class || ''))}
//       >`);
//       docToStrings(ret, el.DocGroup[1]);
//       ret.push('</div>');
//     } else if (el.DocChars) {
//       for (var j = 0; j < el.DocChars.length; j++) {
//         ret.push('<span>');
//         ret.push(String(el.DocChars[j]));
//         ret.push('</span>');
//       }
//     } else {
//       throw new Error('unknown');
//     }
//   }
// }

fn doc_as_html(doc: &DocSpan) -> String {
    let mut out = String::new();
    for elem in doc {
        match elem {
            &DocGroup(ref attrs, ref span) => {
                out.push_str(&format!(r#"<div
                    data-tag={}
                    data-client={}
                    class={}
                >"#, 
                    serde_json::to_string(attrs.get("tag").unwrap_or(&"".to_string())).unwrap(),
                    serde_json::to_string(attrs.get("client").unwrap_or(&"".to_string())).unwrap(),
                    serde_json::to_string(attrs.get("class").unwrap_or(&"".to_string())).unwrap(),
                ));
                out.push_str(&doc_as_html(span));
                out.push_str(r"</div>");
            }
            &DocChars(ref text) => {
                for c in text.chars() {
                    out.push_str(r"<span>");
                    out.push(c);
                    out.push_str(r"</span>");
                }
            }
        }
    }
    out
}

fn client_op<C>(client: &mut Client, callback: C) -> Result<(), Error>
where
    C: Fn(ActionContext) -> Result<Op, Error>,
{
    let client_id = client.name.lock().unwrap().clone().unwrap().to_string();
    let op = callback(ActionContext {
        doc: client.doc.clone(),
        client_id,
    })?;

    // Apply new operation.
    let new_doc = OT::apply(&client.doc, &op);

    client.original_ops.push(op.clone());

    // TODO is this correct
    // println!("ORIGINAL: {:?}", client.original_doc);
    let mut check_op_a = op_span!([], []);
    let name = client.name.lock().unwrap().clone().unwrap();
    for (i, op) in client.original_ops.iter().enumerate() {
        // println!("  {}: applying {:?}/{:?}", name, i + 1, client.original_ops.len());
        // println!("  {} 1️⃣: let op_left = op_span!{:?};", name, check_op_a);
        // println!("  {} 1️⃣: let op_right = op_span!{:?};", name, op);
        check_op_a = Operation::compose(&check_op_a, &op);
        // println!("  {} 1️⃣: let res = op_span!{:?};", name, check_op_a);
        // println!("  {} 1️⃣: let original = doc_span!{:?};", name, client.original_doc);
        // println!("  {} 1️⃣: let latest_doc = doc_span!{:?};", name, client.doc);
        let _ = OT::apply(&client.original_doc, &check_op_a);
    }

    client.doc = new_doc;
    assert_eq!(OT::apply(&client.original_doc, &check_op_a), client.doc);

    // Send update.
    let res = ClientCommand::Update(doc_as_html(&client.doc.0), Some(op.clone()));
    client.send(&res)?;

    // Send operation to sync server.
    client.tx.send(SyncServerCommand::Commit(
        client.name.lock().unwrap().clone().unwrap(),
        op,
        client.version,
    ));

    Ok(())
}

fn key_handlers() -> Vec<(u32, bool, bool, Box<Fn(&mut Client) -> Result<(), Error>>)> {
    vec![
        // backspace
        (
            8,
            false,
            false,
            Box::new(|client: &mut Client| {
                client_op(client, |doc| delete_char(doc))
            }),
        ),
        // left
        (
            37,
            false,
            false,
            Box::new(|client: &mut Client| client_op(client, |doc| caret_move(doc, false))),
        ),
        // right
        (
            39,
            false,
            false,
            Box::new(|client: &mut Client| client_op(client, |doc| caret_move(doc, true))),
        ),
        // up
        (
            38,
            false,
            false,
            Box::new(|client: &mut Client| client_op(client, |doc| caret_block_move(doc, false))),
        ),
        // down
        (
            40,
            false,
            false,
            Box::new(|client: &mut Client| client_op(client, |doc| caret_block_move(doc, true))),
        ),
        // enter
        (
            13,
            false,
            false,
            Box::new(|client: &mut Client| client_op(client, |doc| split_block(doc))),
        ),
    ]
}

fn button_handlers() -> Vec<(&'static str, Box<Fn(&mut Client) -> Result<(), Error>>)> {
    vec![
        (
            "Heading 1",
            Box::new(|client: &mut Client| client_op(client, |doc| replace_block(doc, "h1"))),
        ),
        (
            "Heading 2",
            Box::new(|client: &mut Client| client_op(client, |doc| replace_block(doc, "h2"))),
        ),
        (
            "Heading 3",
            Box::new(|client: &mut Client| client_op(client, |doc| replace_block(doc, "h3"))),
        ),
        (
            "Paragraph",
            Box::new(|client: &mut Client| client_op(client, |doc| replace_block(doc, "p"))),
        ),
        (
            "Code",
            Box::new(|client: &mut Client| client_op(client, |doc| replace_block(doc, "pre"))),
        ),
        (
            "List",
            Box::new(|client: &mut Client| client_op(client, |doc| toggle_list(doc))),
        ),
    ]
}

fn native_command(client: &mut Client, req: NativeCommand) -> Result<(), Error> {
    match req {
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
        }
        NativeCommand::Monkey(setting) => {
            client.monkey.store(setting, Ordering::Relaxed);
        }
    }
    Ok(())
}

enum Task {
    ButtonMonkey,
    LetterMonkey,
    ArrowMonkey,
    BackspaceMonkey,
    EnterMonkey,
    SyncClientCommand(SyncClientCommand),
    NativeCommand(NativeCommand),
}

macro_rules! monkey_task {
    ( $alive:expr, $monkey:expr, $tx:expr, $wait_params:expr, $task:expr ) => {
        {
            let tx = $tx.clone();
            let alive = $alive.clone();
            let monkey = $monkey.clone();
            thread::spawn::<_, Result<(), Error>>(move || {
                let mut rng = rand::thread_rng();
                while alive.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(
                        $wait_params.0 + rng.gen_range($wait_params.1, $wait_params.2),
                    ));
                    if monkey.load(Ordering::Relaxed) {
                        tx.send($task)?;
                    }
                }
                Ok(())
            })
        }
    };
}

type MonkeyParam = (u64, u64, u64);

// "Human-like"
const MONKEY_BUTTON: MonkeyParam = (500, 0, 2000);
const MONKEY_LETTER: MonkeyParam = (50, 0, 200);
const MONKEY_ARROW: MonkeyParam = (0, 0, 500);
const MONKEY_BACKSPACE: MonkeyParam = (0, 0, 200);
const MONKEY_ENTER: MonkeyParam = (600, 0, 3_000);

// Race
// const MONKEY_BUTTON: MonkeyParam = (0, 0, 100);
// const MONKEY_LETTER: MonkeyParam = (0, 0, 100);
// const MONKEY_ARROW: MonkeyParam = (0, 0, 100);
// const MONKEY_BACKSPACE: MonkeyParam = (0, 0, 100);
// const MONKEY_ENTER: MonkeyParam = (0, 0, 1_000);

#[allow(unused)]
fn setup_monkey(alive: Arc<AtomicBool>, monkey: Arc<AtomicBool>, tx: Sender<Task>) {
    // Button monkey.
    monkey_task!(alive, monkey, tx, MONKEY_BUTTON, Task::ButtonMonkey);
    monkey_task!(alive, monkey, tx, MONKEY_LETTER, Task::LetterMonkey);
    monkey_task!(alive, monkey, tx, MONKEY_ARROW, Task::ArrowMonkey);
    monkey_task!(alive, monkey, tx, MONKEY_BACKSPACE, Task::BackspaceMonkey);
    monkey_task!(alive, monkey, tx, MONKEY_ENTER, Task::EnterMonkey);
}

struct SocketHandler {
    name: Arc<Mutex<Option<String>>>,
    monkey: Arc<AtomicBool>,
    alive: Arc<AtomicBool>,
    tx_task: Sender<Task>,
}

impl ws::Handler for SocketHandler {
    fn on_open(&mut self, shake: ws::Handshake) -> Result<(), ws::Error> {
        let client_id = shake.request.resource()[1..].to_string();
        *self.name.lock().unwrap() = Some(client_id);
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
                self.tx_task.send(Task::NativeCommand(value));
            }
        }

        Ok(())
    }

    fn on_error(&mut self, err: ws::Error) {
        println!("Killing after error");
        self.monkey.store(false, Ordering::Relaxed);
        self.alive.store(false, Ordering::Relaxed);
    }

    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        println!("Killing after close");
        self.monkey.store(false, Ordering::Relaxed);
        self.alive.store(false, Ordering::Relaxed);
    }
}

fn handle_task(value: Task, client: &mut Client) -> Result<(), Error> {
    let mut rng = rand::thread_rng();

    match value {
        Task::ButtonMonkey => {
            let index = rng.gen_range(0, button_handlers().len() as u32);
            let command = NativeCommand::Button(index);
            native_command(client, command)?;
        }
        Task::LetterMonkey => {
            let char_list = vec![
                rng.gen_range(b'A', b'Z'),
                rng.gen_range(b'a', b'z'),
                rng.gen_range(b'0', b'9'),
                b' ',
            ];
            let c = *rng.choose(&char_list).unwrap() as u32;
            let command = NativeCommand::Character(c);
            native_command(client, command)?;
        }
        Task::ArrowMonkey => {
            let key = *rng.choose(&[37, 39, 37, 39, 37, 39, 38, 40]).unwrap();
            let command = NativeCommand::Keypress(key, false, false);
            native_command(client, command)?;
        }
        Task::BackspaceMonkey => {
            let command = NativeCommand::Keypress(8, false, false);
            native_command(client, command)?;
        }
        Task::EnterMonkey => {
            let command = NativeCommand::Keypress(13, false, false);
            native_command(client, command)?;
        }

        // Handle commands from Native.
        Task::NativeCommand(command) => {
            native_command(client, command)?;
        }

        // Handle commands from Sync.
        Task::SyncClientCommand(SyncClientCommand::Update(doc, version)) => {
            client.original_doc = Doc(doc.clone());
            client.original_ops = vec![];

            client.doc = Doc(doc.clone());
            client.version = version;
            println!("new version is {:?}", version);

            // Native drives client state.
            let res = ClientCommand::Update(doc_as_html(&doc), None);
            client.send(&res).unwrap();

            // Load the caret.
            if !client.first_load {
                client.first_load = true;

                client.version = 0;
                client_op(client, |doc| init_caret(doc)).unwrap();
                client.version = version;
            }
        }
    }
    Ok(())
}

pub fn server(url: &str) {
    ws::listen(url, |out| {
        let (tx, rx) = unbounded();

        let name = Arc::new(Mutex::new(None));
        let monkey = Arc::new(AtomicBool::new(false));
        let alive = Arc::new(AtomicBool::new(true));

        let mut client = Client {
            name: name.clone(),

            doc: Doc(vec![]),
            version: 100,

            original_doc: Doc(vec![]),
            original_ops: vec![],

            first_load: false,
            monkey: monkey.clone(),
            alive: alive.clone(),

            out,
            tx,
        };

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

        let (tx_task, rx_task) = unbounded();
        setup_monkey(alive.clone(), monkey.clone(), tx_task.clone());

        // Setup monkey tasks.
        {
            thread::spawn::<_, Result<(), Error>>(move || {
                while let Ok(task) = rx_task.recv() {
                    handle_task(task, &mut client)?;
                }
                Ok(())
            });
        }

        // Connect to the sync server.
        {
            clone_all!(tx_task);
            thread::spawn(move || {
                ws::connect("ws://127.0.0.1:3010", move |out| {
                    // Send over operations
                    {
                        clone_all!(tx_task, rx);
                        thread::spawn(move || {
                            while let Ok(command) = rx.recv() {
                                out.send(serde_json::to_string(&command).unwrap()).unwrap();
                            }
                        });
                    }

                    {
                        clone_all!(tx_task);
                        move |msg: ws::Message| {
                            // Handle messages received on this connection
                            println!("wasm got a packet from sync '{}'. ", msg);

                            let req_parse: Result<SyncClientCommand, _> =
                                serde_json::from_slice(&msg.into_data());
                            match req_parse {
                                Err(err) => {
                                    println!("Packet error: {:?}", err);
                                }
                                Ok(value) => {
                                    tx_task.send(Task::SyncClientCommand(value));
                                }
                            }

                            Ok(())
                        }
                    }
                }).unwrap();
            });
        }

        // Websocket message handler.
        SocketHandler {
            name,
            monkey,
            alive,
            tx_task,
        }
    }).unwrap();
}

pub fn start_websocket_server() {
    server("127.0.0.1:3012");
}
