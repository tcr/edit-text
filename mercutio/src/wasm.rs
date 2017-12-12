use rand;
use oatie::doc::*;
use oatie::{OT};
use serde_json;
use ws;
use failure::Error;
use oatie::stepper::*;
use oatie::writer::*;
use std::char::from_u32;
use std::cell::RefCell;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use rand::Rng;

#[derive(Serialize, Deserialize, Debug)]
pub enum NativeCommand {
    Keypress(u32, bool, bool),
    Button(u32),
    Character(u32),
    RenameGroup(String, CurSpan),
    WrapGroup(String, CurSpan),
    Load(DocSpan),
    Target(CurSpan),
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
                    let len = if let Some(DocElement::DocGroup(_, span)) = doc.head.clone() {
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

fn delete_char(client: &Client, input: &CurSpan) -> Result<(), Error> {
    fn delete_char_inner(
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

                    delete_char_inner(input, doc, del, add);

                    input.exit();
                    doc.exit();
                    del.exit();
                    add.exit();
                }
                CurGroup => {
                    del.skip(1);
                    add.skip(1);
                    doc.skip(1);
                    input.next();
                }
                CurChar => {
                    del.chars(1);
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
    delete_char_inner(
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

fn add_char(client: &Client, key: u32) -> Result<(), Error> {
    fn add_char_inner(
        key: char,
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

                    add_char_inner(key, input, doc, del, add);

                    input.exit();
                    doc.exit();
                    del.exit();
                    add.exit();
                }
                CurGroup => {
                    del.skip(1);
                    add.skip(1);
                    doc.skip(1);
                    input.next();
                }
                CurChar => {
                    del.skip(1);
                    add.skip(1);
                    add.chars(&format!("{}", key));
                    doc.skip(1);
                    input.next();
                }
            }
        }
    }

    let mut doc = client.doc.lock().unwrap();
    let input = client.target.lock().unwrap();

    if input.is_none() {
        return Ok(());
    }
    let input = input.clone().unwrap();

    let mut cur_stepper = CurStepper::new(&input);
    let mut doc_stepper = DocStepper::new(&doc.0);
    let mut del_writer = DelWriter::new();
    let mut add_writer = AddWriter::new();
    add_char_inner(
        //TODO >u8
        from_u32(key).unwrap_or('?'),
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

trait DocWalker {
    fn _walk(
        &mut self,
        doc: &mut DocStepper,
    ) {
        while !doc.is_done() && doc.head.is_some() {
            match doc.get_head() {
                DocChars(value) => {
                    self.chars(value);
                    doc.next();
                }
                DocGroup(attrs, span) => {
                    if self.enter(&attrs, &span) {
                        doc.enter();
                        self._walk(doc);
                        doc.exit();
                        self.exit(&attrs);
                    } else {
                        doc.skip(1);
                    }
                }
            }
        }
    }

    fn walk(&mut self, doc: &Doc) {
        let mut stepper = DocStepper::new(&doc.0);
        self._walk(&mut stepper);
    }

    fn chars(&mut self, _chars: String) {}
    fn enter(&mut self, _attrs: &Attrs, _span: &DocSpan) -> bool { true }
    fn exit(&mut self, _attrs: &Attrs) {}
}

#[derive(Debug)]
struct CursorParentGroup {
    del: DelWriter,
    add: AddWriter,

    cursor: bool,
    terminated: bool,
    new_attrs: Attrs,
}

impl CursorParentGroup {
    fn new(new_attrs: &Attrs) -> CursorParentGroup {
        CursorParentGroup {
            del: DelWriter::new(),
            add: AddWriter::new(),
            cursor: false,
            terminated: false,
            new_attrs: new_attrs.clone(),
        }
    }
}

impl DocWalker for CursorParentGroup {
    fn chars(&mut self, text: String) {
        self.del.skip(text.chars().count());
        self.add.skip(text.chars().count());
    }

    fn enter(&mut self, attrs: &Attrs, _span: &DocSpan) -> bool {
        if attrs["tag"] == "cursor" {
            self.cursor = true;
        }

        if self.cursor || self.terminated {
            self.del.skip(1);
            self.add.skip(1);
            return false;
        }

        self.del.begin();
        self.add.begin();
        true
    }

    fn exit(&mut self, attrs: &Attrs) {
        use oatie::schema::*;

        if self.cursor && Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
            self.del.close();
            self.add.close(self.new_attrs.clone());
            self.cursor = false;
            self.terminated = true;
        } else {
            self.del.exit();
            self.add.exit();
        }
    }
}



#[derive(Debug)]
struct CaretSet {
    del: DelWriter,
    add: AddWriter,

    destination: usize,
    pos: usize,
    terminated: bool,
}

impl CaretSet {
    fn new(destination: usize) -> CaretSet {
        CaretSet {
            del: DelWriter::new(),
            add: AddWriter::new(),
            destination,
            pos: 0,
            terminated: false,
        }
    }

    fn check_position(&mut self) {
        if self.pos == self.destination + 1 {
            self.add.begin();
            self.add.close(hashmap! { "tag".to_string() => "cursor".to_string() });
            self.terminated = true;
        }
    }
}

impl DocWalker for CaretSet {
    fn chars(&mut self, text: String) {
        for _ in 0..text.chars().count() {
            self.del.skip(1);
            self.add.skip(1);
            self.pos += 1;
            self.check_position();
        }
    }

    fn enter(&mut self, attrs: &Attrs, _span: &DocSpan) -> bool {
        use oatie::schema::*;

        // Skip over cursor.
        if attrs["tag"] == "cursor" {
            self.del.group_all();
            return false;
        }

        self.del.begin();
        self.add.begin();

        if Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
            self.pos += 1;
            self.check_position();
        }

        true
    }

    fn exit(&mut self, _attrs: &Attrs) {
        self.del.exit();
        self.add.exit();
    }
}



#[derive(Debug)]
struct CaretPosition {
    pos: usize,
    terminated: bool,
}

impl CaretPosition {
    fn new() -> CaretPosition {
        CaretPosition {
            pos: 0,
            terminated: false,
        }
    }

    fn pos(&self) -> usize {
        if self.pos > 0 { self.pos - 1 } else { 0 }
    }
}

impl DocWalker for CaretPosition {
    fn chars(&mut self, text: String) {
        if !self.terminated {
            self.pos += text.chars().count();
        }
    }

    fn enter(&mut self, attrs: &Attrs, _span: &DocSpan) -> bool {
        use oatie::schema::*;

        if attrs["tag"] == "cursor" {
            self.terminated = true;
        }

        if self.terminated {
            return false;
        }

        if Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
            self.pos += 1;
        }

        true
    }
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
            let cur = client.target.lock().unwrap();

            // TODO unwrap into real error, not into panic
            delete_char(client, &cur.clone().unwrap())?;
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
            add_char(client, char_code)?;
        }
        NativeCommand::Target(cur) => {
            *client.target.lock().unwrap() = Some(cur);
        }
        NativeCommand::Load(doc) => {
            *client.doc.lock().unwrap() = Doc(doc);
        }
    }
    Ok(())
}

fn cur_to_caret() {
    // Iterate until cursor is reached
    // If on a char, or a span, use it.
    // If at end of parent which is a block, use it.
    // Otherwise (ascending), whichever is begin next block is it.
    // Otherwise, previous block is it (fallback).
    // Otherwise, no cursor, abort.
}

struct Client {
    out: ws::Sender,
    doc: Mutex<Doc>,
    target: Mutex<Option<CurSpan>>,
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
        });

        client.send(&ClientCommand::Setup {
            keys: key_handlers().into_iter().map(|x| (x.0, x.1, x.2)).collect(),
            buttons: button_handlers().into_iter().enumerate().map(|(i, x)| (i, x.0.to_string())).collect(),
        }).expect("Could not send initial state");

        // Button monkey.
        let thread_client: Arc<_> = client.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(5000));
            let mut rng = rand::thread_rng();
            thread::sleep(Duration::from_millis(rng.gen_range(0, 2000) + 500));
            rand::thread_rng().choose(&button_handlers())
                .map(|button| {
                    button.1(&*thread_client);
                });
        });

        // Letter monkey.
        let thread_client: Arc<_> = client.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(5000));
            loop {
                thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(0, 200) + 100));
                native_command(&*thread_client, NativeCommand::Character(
                    *rand::thread_rng().choose(&vec![
                        rand::thread_rng().gen_range(b'A', b'Z'),
                        rand::thread_rng().gen_range(b'a', b'z'),
                        rand::thread_rng().gen_range(b'0', b'9'),
                        b' ',
                    ]).unwrap() as _));
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
