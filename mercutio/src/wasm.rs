use rand;
use oatie::doc::*;
use oatie::{OT, Operation};
use serde_json;
use ws;
use failure::Error;
use std::char::from_u32;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use rand::Rng;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use super::walkers::*;

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

fn replace_block(doc: &Doc, tag: &str) -> Result<Op, Error> {
    let mut walker = Walker::to_caret(&*doc);
    walker.back_block();

    let len = if let Some(DocGroup(_, ref span)) = walker.doc.head() {
        span.skip_len()
    } else {
        println!("uhg {:?}", walker);
        unreachable!()
    };

    let (mut del_writer, mut add_writer) = walker.to_writers();

    del_writer.group(&del_span![DelSkip(len)]);

    add_writer.group(&hashmap! { "tag".to_string() => tag.to_string() }, &add_span![AddSkip(len)]);

    Ok((del_writer.result(), add_writer.result()))
}

fn delete_char(doc: &Doc) -> Result<Op, Error> {
    let mut walker = Walker::to_caret(&*doc);
    walker.back_char();

    if let Some(DocChars(..)) = walker.doc.head() {
        // fallthrough
    } else {
        return Ok(op_span!([], []));
    }

    let (mut del_writer, mut add_writer) = walker.to_writers();

    del_writer.chars(1);
    del_writer.exit_all();

    add_writer.exit_all();

    Ok((del_writer.result(), add_writer.result()))
}

fn add_char(doc: &Doc, key: u32) -> Result<Op, Error> {
    let (mut del_writer, mut add_writer) = Walker::to_caret(&*doc)
        .to_writers();

    del_writer.exit_all();

    let c: char = from_u32(key).unwrap_or('?');
    add_writer.chars(&format!("{}", c));
    add_writer.exit_all();

    Ok((del_writer.result(), add_writer.result()))
}

fn caret_move(doc: &Doc, increase: bool) -> Result<Op, Error> {
    let mut writer = Walker::to_caret(&*doc);

    if increase {
        writer.next_char();
    } else {
        writer.back_char();
    }

    let (mut del_writer, mut add_writer) = writer.to_writers();

    del_writer.exit_all();
    add_writer.exit_all();

    Ok((del_writer.result(), add_writer.result()))
}

fn cur_to_caret(doc: &Doc, cur: &CurSpan) -> Result<Op, Error> {
    let (mut del_writer, mut add_writer) = Walker::to_caret(&*doc).to_writers();

    del_writer.begin();
    del_writer.close();
    del_writer.exit_all();

    add_writer.exit_all();

    let op_1 = (del_writer.result(), add_writer.result());

    let mut doc_2: Doc = doc.clone();
    OT::apply(&doc_2, &op_1);

    let mut writer = Walker::to_cursor(&doc_2, cur);
    writer.snap_char();
    


    let (mut del_writer, mut add_writer) = writer.to_writers();

    del_writer.exit_all();

    add_writer.begin();
    add_writer.close(hashmap! { "tag".to_string() => "cursor".to_string() });
    add_writer.exit_all();

    let op_2 = (del_writer.result(), add_writer.result());

    Ok(Operation::compose(&op_1, &op_2))
}

fn client_op<C: Fn(&Doc) -> Result<Op, Error>>(client: &Client, callback: C) -> Result<(), Error> {
    let mut doc = client.doc.lock().unwrap();

    let op = callback(&*doc)?;

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
        (190, true, false, Box::new(|client: &Client| {
            println!("renaming a group.");
            let cur = client.target.lock().unwrap();

            // Unwrap into real error
            let future = NativeCommand::RenameGroup("null".into(), cur.clone().unwrap());
            let prompt = ClientCommand::PromptString("Rename tag group:".into(), "p".into(), future);
            client.send(&prompt)?;
            Ok(())
        })),

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
        (8, false, false, Box::new(|client: &Client| {
            println!("backspace");
            client_op(client, |doc| delete_char(doc))
        })),

        // left
        (37, false, false, Box::new(|client: &Client| {
            client_op(client, |doc| caret_move(doc, false))
        })),
        // right
        (39, false, false, Box::new(|client: &Client| {
            client_op(client, |doc| caret_move(doc, true))
        })),
    ]
}

fn button_handlers() -> Vec<(&'static str, Box<Fn(&Client) -> Result<(), Error>>)> {
    vec![
        ("Heading 1", Box::new(|client: &Client| {
            client_op(client, |doc| replace_block(doc, "h1"))
        })),
        ("Heading 2", Box::new(|client: &Client| {
            client_op(client, |doc| replace_block(doc, "h2"))
        })),
        ("Heading 3", Box::new(|client: &Client| {
            client_op(client, |doc| replace_block(doc, "h3"))
        })),
        ("Paragraph", Box::new(|client: &Client| {
            client_op(client, |doc| replace_block(doc, "p"))
        })),
        ("Code", Box::new(|client: &Client| {
            client_op(client, |doc| replace_block(doc, "pre"))
        })),
    ]
}

fn native_command(client: &Client, req: NativeCommand) -> Result<(), Error> {
    match req {
        NativeCommand::RenameGroup(tag, cur) => {
            client_op(client, |doc| replace_block(doc, &tag))?
        }
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
        NativeCommand::Character(char_code) => {
            client_op(client, |doc| add_char(doc, char_code))?
        }
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
}

impl Client {
    fn send(&self, req: &ClientCommand) -> Result<(), Error> {
        let json = serde_json::to_string(&req)?;
        self.out.send(json)?;
        Ok(())
    }
}

pub fn start_websocket_server() {
    ws::listen("127.0.0.1:3012", |out| {
        let client = Arc::new(Client {
            out,
            doc: Mutex::new(Doc(vec![])),
            target: Mutex::new(None),
            monkey: AtomicBool::new(false),
        });

        client.send(&ClientCommand::Setup {
            keys: key_handlers().into_iter().map(|x| (x.0, x.1, x.2)).collect(),
            buttons: button_handlers().into_iter().enumerate().map(|(i, x)| (i, x.0.to_string())).collect(),
        }).expect("Could not send initial state");

        // Button monkey.
        let thread_client: Arc<_> = client.clone();
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            loop {
                thread::sleep(Duration::from_millis(rng.gen_range(0, 2000) + 500));
                if thread_client.monkey.load(Ordering::Relaxed) {
                    rand::thread_rng()
                        .choose(&button_handlers())
                        .map(|button| {
                            button.1(&*thread_client);
                        });
                }
            }
        });

        // Letter monkey.
        let thread_client: Arc<_> = client.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(0, 200) + 100));
                if thread_client.monkey.load(Ordering::Relaxed) {
                    native_command(&*thread_client, NativeCommand::Character(
                        *rand::thread_rng().choose(&vec![
                            rand::thread_rng().gen_range(b'A', b'Z'),
                            rand::thread_rng().gen_range(b'a', b'z'),
                            rand::thread_rng().gen_range(b'0', b'9'),
                            b' ',
                        ]).unwrap() as _));
                }
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
