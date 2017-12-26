pub mod actions;
pub mod walkers;

use rand;
use oatie::doc::*;
use oatie::OT;
use serde_json;
use ws;
use failure::Error;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use rand::Rng;
use std::{panic, process};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use self::actions::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum NativeCommand {
    Keypress(u32, bool, bool),
    Button(u32),
    Character(u32),
    RenameGroup(String, CurSpan),
    Load(DocSpan),
    Target(CurSpan),
    Monkey(bool),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientCommand {
    Setup {
        keys: Vec<(u32, bool, bool)>,
        buttons: Vec<(usize, String)>,
    },
    PromptString(String, String, NativeCommand),
    Update(DocSpan, Op),
    Error(String),
}

fn client_op<C>(client: &Client, callback: C) -> Result<(), Error>
where
    C: Fn(ActionContext) -> Result<Op, Error>,
{
    let mut doc = client.doc.lock().unwrap();

    let op = callback(ActionContext {
        doc: doc.clone(),
        client_id: client.name.to_string(),
    })?;

    // Apply new operation.
    let new_doc = OT::apply(&*doc, &op);
    *doc = new_doc;

    // Send update.
    let res = ClientCommand::Update(doc.0.clone(), op);
    client.send(&res)?;

    Ok(())
}

fn key_handlers() -> Vec<(u32, bool, bool, Box<Fn(&Client) -> Result<(), Error>>)> {
    vec![
        // command + .
        // (
        //     190,
        //     true,
        //     false,
        //     Box::new(|client: &Client| {
        //         println!("renaming a group.");
        //         let cur = client.target.lock().unwrap();

        //         // Unwrap into real error
        //         let future = NativeCommand::RenameGroup("null".into(), cur.clone().unwrap());
        //         let prompt =
        //             ClientCommand::PromptString("Rename tag group:".into(), "p".into(), future);
        //         client.send(&prompt)?;
        //         Ok(())
        //     }),
        // ),
        // // command + ,
        // (188, true, false, Box::new(|client: &Client| {
        //     println!("renaming a group.");
        //     let cur = client.target.lock().unwrap();

        //     let future = NativeCommand::WrapGroup("null".into(), cur.clone().unwrap());
        //     let prompt = ClientCommand::PromptString("Name of new outer tag:".into(), "p".into(), future);
        //     client.send(&prompt)?;
        //     Ok(())
        // })),

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
    ]
}

fn native_command(client: &Client, req: NativeCommand) -> Result<(), Error> {
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
            *client.target.lock().unwrap() = Some(cur);
        }
        NativeCommand::Load(doc) => {
            *client.doc.lock().unwrap() = Doc(doc);
        }
        NativeCommand::Monkey(setting) => {
            client.monkey.store(setting, Ordering::Relaxed);
        }
    }
    Ok(())
}

struct Client {
    out: ws::Sender,
    doc: Mutex<Doc>,
    //TODO remove the target field? base only on carets instead
    target: Mutex<Option<CurSpan>>,
    monkey: AtomicBool,
    name: String,
}

impl Client {
    fn send(&self, req: &ClientCommand) -> Result<(), Error> {
        let json = serde_json::to_string(&req)?;
        self.out.send(json)?;
        Ok(())
    }
}

pub fn server(url: &str, name: &str) {
    ws::listen(url, |out| {
        let client = Arc::new(Client {
            out,
            doc: Mutex::new(Doc(vec![])),
            target: Mutex::new(None),
            monkey: AtomicBool::new(false),
            name: name.to_string(),
        });

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

        // Button monkey.
        let thread_client: Arc<_> = client.clone();
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            loop {
                thread::sleep(Duration::from_millis(rng.gen_range(0, 2000) + 500));
                if thread_client.monkey.load(Ordering::Relaxed) {
                    rand::thread_rng().choose(&button_handlers()).map(|button| {
                        button.1(&*thread_client);
                    });
                }
            }
        });

        // Letter monkey.
        let thread_client: Arc<_> = client.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(
                rand::thread_rng().gen_range(0, 2000) + 100,
            ));
            if thread_client.monkey.load(Ordering::Relaxed) {
                native_command(
                    &*thread_client,
                    NativeCommand::Character(*rand::thread_rng()
                        .choose(&vec![
                            rand::thread_rng().gen_range(b'A', b'Z'),
                            rand::thread_rng().gen_range(b'a', b'z'),
                            rand::thread_rng().gen_range(b'0', b'9'),
                            b' ',
                        ])
                        .unwrap() as _),
                );
            }
        });

        // Arrow keys.
        let thread_client: Arc<_> = client.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(
                rand::thread_rng().gen_range(0, 300) + 700,
            ));
            if thread_client.monkey.load(Ordering::Relaxed) {
                native_command(
                    &*thread_client,
                    NativeCommand::Keypress(*rand::thread_rng().choose(&[37, 39]).unwrap(), false, false),
                );
            }
        });

        // Enter monkey.
        let thread_client: Arc<_> = client.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(
                rand::thread_rng().gen_range(0, 3_000) + 1000,
            ));
            if thread_client.monkey.load(Ordering::Relaxed) {
                native_command(&*thread_client, NativeCommand::Keypress(13, false, false));
            }
        });

        // Arrow monkey
        // native_command(&*thread_client, NativeCommand::Keypress(39, false, false));

        move |msg: ws::Message| {
            // Handle messages received on this connection
            println!("Server got message '{}'. ", msg);

            let req_parse: Result<NativeCommand, _> = serde_json::from_slice(&msg.into_data());
            match req_parse {
                Err(err) => {
                    println!("Packet error: {:?}", err);
                }
                Ok(value) => {
                    native_command(&client, value).expect("Native command error");
                }
            }

            Ok(())
        }
    }).unwrap();
}

pub fn start_websocket_server() {
    thread::spawn(|| {
        if let Err(value) = panic::catch_unwind(|| {
            server("127.0.0.1:3012", "left");
        }) {
            println!("Error: {:?}", value);
            process::exit(1);
        }
    });

    thread::spawn(|| {
        if let Err(value) = panic::catch_unwind(|| {
            server("127.0.0.1:3013", "right");
        }) {
            println!("Error: {:?}", value);
            process::exit(1);
        }
    });
}
