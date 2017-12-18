use rand;
use oatie::doc::*;
use oatie::{OT};
use serde_json;
use ws;
use failure::Error;
use oatie::stepper::*;
use oatie::writer::*;
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
    WrapGroup(String, CurSpan),
    Load(DocSpan),
    Target(CurSpan),
    Monkey(bool),
}


#[derive(Debug)]
struct Walker {
    original_doc: Doc,
    doc: DocStepper,
    caret_pos: isize,
}

impl Walker {
    fn to_cursor(doc: &Doc) -> Walker {
        use oatie::schema::*;

        // Walk the doc until the thing
        let mut walker = Walker {
            original_doc: doc.clone(),
            doc: DocStepper::new(&doc.0),
            caret_pos: -1,
        };

        let mut matched = false;
        loop {
            match walker.doc.head() {
                Some(DocChars(text)) => {
                    walker.caret_pos += 1;
                    walker.doc.skip(1);
                },
                Some(DocGroup(attrs, _)) => {
                    if attrs["tag"] == "cursor" {
                        matched = true;
                        break;
                    }

                    if Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
                        walker.caret_pos += 1;
                    }

                    walker.doc.enter();
                }
                None => {
                    if walker.doc.is_done() {
                        break;
                    } else {
                        walker.doc.exit();
                    }
                }
            }
        }
        if !matched {
            panic!("Didn't find a cursor.");
        }

        walker
    }

    fn back_chars(&mut self, count: usize) -> &mut Walker {
        use oatie::schema::*;

        let mut matched = false;
        self.doc.unskip(1);
        loop {
            match self.doc.head() {
                Some(DocChars(text)) => {
                    self.caret_pos -= 1;
                    matched = true;
                    break;
                },
                Some(DocGroup(attrs, _)) => {
                    self.doc.unenter();
                }
                None => {
                    // TODO check backwards is_done()!!!
                    if self.doc.is_done() {
                        break;
                    } else {
                        self.doc.exit();
                        match self.doc.head() {
                            Some(DocGroup(attrs, _)) => {
                                self.doc.prev();
                                if Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
                                    self.caret_pos -= 1;
                                    break;
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }
        if !matched {
            panic!("Didn't find a cursor.");
        }
        self
    }

    fn to_writers(&self) -> (DelWriter, AddWriter) {
        let mut del = DelWriter::new();
        let mut add = AddWriter::new();

        // Walk the doc until the thing
        let mut doc_stepper = DocStepper::new(&self.original_doc.0);

        while self.doc != doc_stepper {
            match doc_stepper.head() {
                Some(DocChars(text)) => {
                    del.skip(1);
                    add.skip(1);
                    doc_stepper.skip(1);
                },
                Some(DocGroup(attrs, _)) => {
                    del.begin();
                    add.begin();
                    doc_stepper.enter();
                }
                None => {
                    del.exit();
                    add.exit();
                    if doc_stepper.is_done() {
                        break;
                    } else {
                        doc_stepper.exit();
                    }
                }
            }
        }

        (del, add)
    }
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

fn rename_group(client: &Client, tag: &str, input: &CurSpan) -> Result<(), Error> {
    fn rename_group_inner(
        tag: &str,
        input: &mut CurStepper,
        doc: &mut DocStepper,
        del: &mut DelWriter,
        add: &mut AddWriter,
    ) {
        while !input.is_done() && input.head.is_some() {
            match input.get_head() {
                CurSkip(value) => {
                    doc.skip(value);
                    input.next();
                    del.skip(value);
                    add.skip(value);
                }
                CurWithGroup(..) => {
                    input.enter();
                    doc.enter();
                    del.begin();
                    add.begin();

                    rename_group_inner(tag, input, doc, del, add);

                    input.exit();
                    doc.exit();
                    del.exit();
                    add.exit();
                }
                CurGroup => {
                    // Get doc inner span length
                    let len = if let Some(DocElement::DocGroup(_, span)) = doc.head() {
                        span.skip_len()
                    } else {
                        panic!("unreachable");
                    };

                    del.begin();
                    add.begin();

                    del.skip(len);
                    add.skip(len);

                    del.close();
                    add.close(hashmap! { "tag".to_string() => tag.to_string() });
                    doc.next();
                    input.next();
                }
                CurChar => {
                    del.skip(1);
                    add.skip(1);
                    doc.skip(1);
                    input.next();
                }
            }
        }
    }

    let mut doc = client.doc.lock().unwrap();

    let mut cur_stepper = CurStepper::new(input);
    let mut doc_stepper = DocStepper::new(&doc.0);
    let mut del_writer = DelWriter::new();
    let mut add_writer = AddWriter::new();
    rename_group_inner(
        tag,
        &mut cur_stepper,
        &mut doc_stepper,
        &mut del_writer,
        &mut add_writer,
    );

    // Apply operation.
    let op = (del_writer.result(), add_writer.result());
    let new_doc = OT::apply(&*doc, &op);

    // Store update
    *doc = new_doc;

    // Send update.
    let res = ClientCommand::Update(doc.0.clone(), op);
    client.send(&res)?;

    Ok(())
}

fn wrap_group(client: &Client, tag: &str, input: &CurSpan) -> Result<(), Error> {
    fn wrap_group_inner(
        tag: &str,
        input: &mut CurStepper,
        doc: &mut DocStepper,
        del: &mut DelWriter,
        add: &mut AddWriter,
    ) {
        while !input.is_done() && input.head.is_some() {
            match input.get_head() {
                CurSkip(value) => {
                    doc.skip(value);
                    input.next();
                    del.skip(value);
                    add.skip(value);
                }
                CurWithGroup(..) => {
                    input.enter();
                    doc.enter();
                    del.begin();
                    add.begin();

                    wrap_group_inner(tag, input, doc, del, add);

                    input.exit();
                    doc.exit();
                    del.exit();
                    add.exit();
                }
                CurGroup => {
                    del.skip(1);

                    add.begin();
                    add.skip(1);
                    add.close(hashmap! { "tag".to_string() => tag.to_string() });

                    doc.next();
                    input.next();
                }
                CurChar => {
                    del.skip(1);
                    add.skip(1);
                    doc.skip(1);
                    input.next();
                }
            }
        }
    }

    let mut doc = client.doc.lock().unwrap();

    let mut cur_stepper = CurStepper::new(input);
    let mut doc_stepper = DocStepper::new(&doc.0);
    let mut del_writer = DelWriter::new();
    let mut add_writer = AddWriter::new();
    wrap_group_inner(
        tag,
        &mut cur_stepper,
        &mut doc_stepper,
        &mut del_writer,
        &mut add_writer,
    );

    // Apply operation.
    let op = (del_writer.result(), add_writer.result());
    let new_doc = OT::apply(&*doc, &op);

    // Store update
    *doc = new_doc;

    // Send update.
    let res = ClientCommand::Update(doc.0.clone(), op);
    client.send(&res)?;

    Ok(())
}


fn delete_char(doc: &Doc) -> Result<Op, Error> {
    let mut walker = Walker::to_cursor(&*doc);
    walker.back_chars(1);

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
    let (mut del_writer, mut add_writer) = Walker::to_cursor(&*doc)
        .to_writers();

    del_writer.exit_all();

    let c: char = from_u32(key).unwrap_or('?');
    add_writer.chars(&format!("{}", c));
    add_writer.exit_all();

    Ok((del_writer.result(), add_writer.result()))
}

fn cursor_parent(client: &Client, replace_with: &Attrs) -> Result<(), Error> {
    let mut doc = client.doc.lock().unwrap();

    let mut walker = CursorParentGroup::new(&replace_with);
    walker.walk(&*doc);
    //assert!(walker.terminated);

    // Apply operation.
    let op = (walker.del.result(), walker.add.result());
    let new_doc = OT::apply(&*doc, &op);

    // Store update
    *doc = new_doc;

    // Send update.
    let res = ClientCommand::Update(doc.0.clone(), op);
    client.send(&res)?;

    Ok(())
}

fn caret_move(client: &Client, increase: bool) -> Result<(), Error> {
    let mut doc = client.doc.lock().unwrap();

    let mut w = CaretPosition::new();
    w.walk(&*doc);
    println!("COUNT {:?}", w.pos());

    // TODO need a literate way to express these commands
    // walker.to_cursor().back_chars(1).delete_char()
    // 

    // let mut walker = CursorParentGroup::new(&replace_with);
    // walker.walk(&*doc);
    let mut walker = CaretSet::new(if increase { w.pos() + 1 } else { w.pos() - 1 });
    walker.walk(&*doc);
    //assert!(walker.terminated);

    // Apply operation.
    let op = (walker.del.result(), walker.add.result());
    let new_doc = OT::apply(&*doc, &op);

    // Store update
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

        // command + ,
        (188, true, false, Box::new(|client: &Client| {
            println!("renaming a group.");
            let cur = client.target.lock().unwrap();

            let future = NativeCommand::WrapGroup("null".into(), cur.clone().unwrap());
            let prompt = ClientCommand::PromptString("Name of new outer tag:".into(), "p".into(), future);
            client.send(&prompt)?;
            Ok(())
        })),

        // backspace
        (8, false, false, Box::new(|client: &Client| {
            println!("backspace");

            let mut doc = client.doc.lock().unwrap();

            let op = delete_char(&*doc)?;

            // Apply new operation.
            let new_doc = OT::apply(&*doc, &op);
            *doc = new_doc;

            // Send update.
            let res = ClientCommand::Update(doc.0.clone(), op);
            client.send(&res)?;

            Ok(())
        })),

        // left
        (37, false, false, Box::new(|client: &Client| {
            caret_move(client, false)?;
            Ok(())
        })),
        // right
        (39, false, false, Box::new(|client: &Client| {
            caret_move(client, true)?;
            Ok(())
        })),
    ]
}

fn button_handlers() -> Vec<(&'static str, Box<Fn(&Client) -> Result<(), Error>>)> {
    vec![
        ("Heading 1", Box::new(|client: &Client| {
            cursor_parent(client, &hashmap! { "tag".to_string() => "h1".to_string() })?;
            Ok(())
        })),
        ("Heading 2", Box::new(|client: &Client| {
            cursor_parent(client, &hashmap! { "tag".to_string() => "h2".to_string() })?;
            Ok(())
        })),
        ("Heading 3", Box::new(|client: &Client| {
            cursor_parent(client, &hashmap! { "tag".to_string() => "h3".to_string() })?;
            Ok(())
        })),
        ("Paragraph", Box::new(|client: &Client| {
            cursor_parent(client, &hashmap! { "tag".to_string() => "p".to_string() })?;
            Ok(())
        })),
        ("Code", Box::new(|client: &Client| {
            cursor_parent(client, &hashmap! { "tag".to_string() => "pre".to_string() })?;
            Ok(())
        })),
    ]
}

fn native_command(client: &Client, req: NativeCommand) -> Result<(), Error> {
    match req {
        NativeCommand::RenameGroup(tag, cur) => {
            rename_group(client, &tag, &cur)?;
        }
        NativeCommand::WrapGroup(tag, cur) => {
            wrap_group(client, &tag, &cur)?;
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
            let mut doc = client.doc.lock().unwrap();

            let op = add_char(&*doc, char_code)?;

            // Apply new operation.
            let new_doc = OT::apply(&*doc, &op);
            *doc = new_doc;

            // Send update.
            let res = ClientCommand::Update(doc.0.clone(), op);
            client.send(&res)?;
        }
        NativeCommand::Target(cur) => {
            cur_to_caret(client, &cur);
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

fn cur_to_caret(client: &Client, cur: &CurSpan) -> Result<(), Error> {
    let mut doc = client.doc.lock().unwrap();

    let mut walker = CursorToCaretPosition::new(CurStepper::new(cur));
    walker.walk(&*doc);

    println!("target: {:?}", walker.pos());

    // Iterate until cursor is reached
    // If on a char, or a span, use it.
    // If at end of parent which is a block, use it.
    // Otherwise (ascending), whichever is begin next block is it.
    // Otherwise, previous block is it (fallback).
    // Otherwise, no cursor, abort.

    // let mut walker = CursorParentGroup::new(&replace_with);
    // walker.walk(&*doc);
    let mut walker = CaretSet::new(walker.pos());
    walker.walk(&*doc);
    //assert!(walker.terminated);

    // Apply operation.
    let op = (walker.del.result(), walker.add.result());
    let new_doc = OT::apply(&*doc, &op);

    // Store update
    *doc = new_doc;

    // Send update.
    let res = ClientCommand::Update(doc.0.clone(), op);
    client.send(&res)?;

    Ok(())
}

struct Client {
    out: ws::Sender,
    doc: Mutex<Doc>,
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
