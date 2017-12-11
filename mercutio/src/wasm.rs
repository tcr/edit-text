use oatie::doc::*;
use oatie::{OT};
use serde_json;
use ws;
use failure::Error;
use oatie::stepper::*;
use oatie::writer::*;
use std::char::from_u32;

#[derive(Serialize, Deserialize, Debug)]
pub enum NativeCommand {
    Keypress(DocSpan, u32, u32, bool, bool, CurSpan),
    Character(DocSpan, u32, u32, bool, bool, CurSpan),
    RenameGroup(String, DocSpan, CurSpan),
    WrapGroup(String, DocSpan, CurSpan),
    Error(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientCommand {
    PromptString(String, String, NativeCommand),
    Update(DocSpan, Op),
    Error(String),
}

fn rename_group(client: &Client, doc: DocSpan, tag: &str, input: &CurSpan) -> Result<(), Error> {
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

    let mut cur_stepper = CurStepper::new(input);
    let mut doc_stepper = DocStepper::new(&doc);
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
    let new_doc = OT::apply(&Doc(doc), &op);

    // Send update.
    let res = ClientCommand::Update(new_doc.0, op);
    client.send(&res)?;

    Ok(())
}

fn wrap_group(client: &Client, doc: DocSpan, tag: &str, input: &CurSpan) -> Result<(), Error> {
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

    let mut cur_stepper = CurStepper::new(input);
    let mut doc_stepper = DocStepper::new(&doc);
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
    let new_doc = OT::apply(&Doc(doc), &op);

    // Send update.
    let res = ClientCommand::Update(new_doc.0, op);
    client.send(&res)?;

    Ok(())
}

fn delete_char(client: &Client, doc: DocSpan, input: &CurSpan) -> Result<(), Error> {
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

    let mut cur_stepper = CurStepper::new(input);
    let mut doc_stepper = DocStepper::new(&doc);
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
    let new_doc = OT::apply(&Doc(doc), &op);

    // Send update.
    let res = ClientCommand::Update(new_doc.0, op);
    client.send(&res)?;

    Ok(())
}

fn add_char(client: &Client, doc: DocSpan, key: u32, input: &CurSpan) -> Result<(), Error> {
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

    let mut cur_stepper = CurStepper::new(input);
    let mut doc_stepper = DocStepper::new(&doc);
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
    let new_doc = OT::apply(&Doc(doc), &op);

    // Send update.
    let res = ClientCommand::Update(new_doc.0, op);
    client.send(&res)?;

    Ok(())
}

// Load in key command.
fn key_command(
    client: &Client,
    doc: DocSpan,
    key_code: u32,
    char_code: u32,
    meta_key: bool,
    shift_key: bool,
    cur: CurSpan,
) -> Result<(), Error> {
    println!("key: {:?} {:?} {:?} {:?}", key_code, char_code, meta_key, shift_key);

    // command + .
    if key_code == 190 && meta_key && !shift_key {
        println!("renaming a group.");

        let future = NativeCommand::RenameGroup("null".into(), doc, cur);
        let prompt = ClientCommand::PromptString("Rename tag group:".into(), "p".into(), future);
        client.send(&prompt)?;
    }
    // command + ,
    else if key_code == 188 && meta_key && !shift_key {
        println!("wrapping a group.");

        let future = NativeCommand::WrapGroup("null".into(), doc, cur);
        let prompt = ClientCommand::PromptString("Name of new outer tag:".into(), "p".into(), future);
        client.send(&prompt)?;
    }
    // backspace
    else if key_code == 8 && !meta_key && !shift_key {
        println!("backspace");

        delete_char(client, doc, &cur)?;
    }

    Ok(())
}

fn native_command(client: &Client, req: NativeCommand) -> Result<(), Error> {
    match req {
        NativeCommand::RenameGroup(tag, doc, cur) => {
            rename_group(client, doc, &tag, &cur)?;
        }
        NativeCommand::WrapGroup(tag, doc, cur) => {
            wrap_group(client, doc, &tag, &cur)?;
        }
        NativeCommand::Keypress(doc, key_code, char_code, meta_key, shift_key, cur) => {
            key_command(client, doc, key_code, char_code, meta_key, shift_key, cur)?;
        }
        NativeCommand::Character(doc, key_code, char_code, meta_key, shift_key, cur) => {
            add_char(client, doc, char_code, &cur)?;
        }
        _ => {
            println!("unsupported request: {:?}", req);
        }
    }
    Ok(())
}

struct Client {
    out: ws::Sender,
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
            out
        };
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
            // out.send(msg)
        }
    }).unwrap();
}
