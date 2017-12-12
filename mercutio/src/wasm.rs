use oatie::doc::*;
use oatie::{OT};
use serde_json;
use ws;
use failure::Error;
use oatie::stepper::*;
use oatie::writer::*;
use std::char::from_u32;
use std::cell::RefCell;

//TODO move this to being loaded from JS
fn default_doc() -> Doc {
    Doc(doc_span![
        DocGroup({"tag": "h1"}, [
            DocGroup({"tag": "cursor"}, []),
            DocChars("Hello! "),
            DocGroup({"tag": "span", "class": "bold"}, [DocChars("what's")]),
            DocChars(" up?"),
        ]),
        DocGroup({"tag": "ul"}, [
            DocGroup({"tag": "li"}, [
                DocGroup({"tag": "p"}, [
                    DocChars("Three adjectives strong."),
                ]),
                DocGroup({"tag": "p"}, [
                    DocChars("World!"),
                ]),
            ]),
        ])
    ])
}

#[derive(Serialize, Deserialize, Debug)]
pub enum NativeCommand {
    Keypress(u32, bool, bool),
    Button(u32),
    Character(u32, CurSpan),
    RenameGroup(String, CurSpan),
    WrapGroup(String, CurSpan),
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

    let mut doc = client.doc.borrow_mut();

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

    let mut doc = client.doc.borrow_mut();

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

    let mut doc = client.doc.borrow_mut();

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

fn add_char(client: &Client, key: u32, input: &CurSpan) -> Result<(), Error> {
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

    let mut doc = client.doc.borrow_mut();

    let mut cur_stepper = CurStepper::new(input);
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

    fn exit(&mut self, _attrs: &Attrs) {
        if self.cursor {
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

fn cursor_parent(client: &Client, replace_with: &Attrs) -> Result<(), Error> {
    let mut doc = client.doc.borrow_mut();

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

fn key_handlers() -> Vec<(u32, bool, bool, Box<Fn(&Client) -> Result<(), Error>>)> {
    vec![
        // command + .
        (190, true, false, Box::new(|client: &Client| {
            println!("renaming a group.");
            let cur = client.target.borrow();

            // Unwrap into real error
            let future = NativeCommand::RenameGroup("null".into(), cur.clone().unwrap());
            let prompt = ClientCommand::PromptString("Rename tag group:".into(), "p".into(), future);
            client.send(&prompt)?;
            Ok(())
        })),

        // command + ,
        (188, true, false, Box::new(|client: &Client| {
            println!("renaming a group.");
            let cur = client.target.borrow();

            let future = NativeCommand::WrapGroup("null".into(), cur.clone().unwrap());
            let prompt = ClientCommand::PromptString("Name of new outer tag:".into(), "p".into(), future);
            client.send(&prompt)?;
            Ok(())
        })),

        // backspace
        (8, false, false, Box::new(|client: &Client| {
            println!("backspace");
            let cur = client.target.borrow();

            // TODO unwrap into real error, not into panic
            delete_char(client, &cur.clone().unwrap())?;
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
            button_handlers()
                .get(index as usize)
                .map(|handler| handler.1(client));
        }
        NativeCommand::Keypress(key_code, meta_key, shift_key) => {
            println!("key: {:?} {:?} {:?}", key_code, meta_key, shift_key);

            for command in key_handlers() {
                if command.0 == key_code && command.1 == meta_key && command.2 == shift_key {
                    command.3(client)?;
                    break;
                }
            }
        }
        NativeCommand::Character(char_code, cur) => {
            add_char(client, char_code, &cur)?;
        }
        NativeCommand::Target(cur) => {
            *client.target.borrow_mut() = Some(cur);

        }
    }
    Ok(())
}

struct Client {
    out: ws::Sender,
    doc: RefCell<Doc>,
    target: RefCell<Option<CurSpan>>,
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
        let client = Client {
            out,
            doc: RefCell::new(default_doc()),
            target: RefCell::new(None),
        };

        client.send(&ClientCommand::Setup {
            keys: key_handlers().into_iter().map(|x| (x.0, x.1, x.2)).collect(),
            buttons: button_handlers().into_iter().enumerate().map(|(i, x)| (i, x.0.to_string())).collect(),
        }).expect("Could not send initial state");

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
