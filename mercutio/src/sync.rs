use std::sync::{Arc, Mutex};
use oatie::doc::*;
use oatie::{Operation, OT};
use rocket_contrib::Json;
use rocket::State;
use rocket::response::NamedFile;
use failure::Error;
use serde_json::Value;
use std::path::{Path, PathBuf};
use oatie::transform::transform;
use oatie::debug_pretty;
use wasm::start_websocket_server;
use std::thread;
use oatie::schema::{validate_doc_span, ValidateContext};
use ws;
use serde_json;

pub fn default_doc() -> Doc {
    Doc(doc_span![
        DocGroup({"tag": "h1"}, [
            DocGroup({"tag": "caret", "client": "left"}, []),
            DocGroup({"tag": "caret", "client": "right"}, []),
            DocChars("Hello world!"),
        ]),
        DocGroup({"tag": "p"}, [
            // DocChars("What's "),
            // DocGroup({"tag": "span", "class": "bold"}, [DocChars("new and great")]),
            // DocChars(" with you?"),
            DocChars("What's up with you?"),
        ]),
        // DocGroup({"tag": "ul"}, [
        //     DocGroup({"tag": "li"}, [
        //         DocGroup({"tag": "p"}, [
        //             DocChars("Three adjectives strong."),
        //         ]),
        //         DocGroup({"tag": "p"}, [
        //             DocChars("World!"),
        //         ]),
        //     ]),
        // ])
    ])
}

#[derive(Clone)]
pub struct MoteState {
    pub body: Arc<Mutex<Doc>>,
}

pub fn action_sync(doc: &Doc, ops_a: Vec<Op>, ops_b: Vec<Op>) -> Result<Doc, Error> {
    println!(" ---> input ops_a");
    println!("{:?}", ops_a);
    println!();

    // Flatten client A operations.
    let mut op_a = op_span!([], []);
    for op in &ops_a {
        op_a = Operation::compose(&op_a, op);
    }

    println!(" ---> input ops_b");
    println!("{:?}", ops_b);
    println!();

    // Flatten client B operations.
    let mut op_b = op_span!([], []);
    for op in &ops_b {
        op_b = Operation::compose(&op_b, op);
    }

    println!("OP A {:?}", op_a);
    println!("OP B {:?}", op_b);

    println!();
    println!("<test>");
    println!("doc:   {}", debug_pretty(&doc.0));
    println!();
    println!("a_del: {}", debug_pretty(&op_a.0));
    println!("a_add: {}", debug_pretty(&op_a.1));
    println!();
    println!("b_del: {}", debug_pretty(&op_b.0));
    println!("b_add: {}", debug_pretty(&op_b.1));
    println!("</test>");
    println!();

    println!("(!) recreating initial client state...");
    println!();

    // TODO remove this validation code if we're performing the check client-side

    // let mut check_op_a = op_span!([], []);
    // for (i, op) in ops_a.iter().enumerate() {
    //     println!("  A: applying {:?}/{:?}", i + 1, ops_a.len());
    //     check_op_a = Operation::compose(&check_op_a, &op);
    //     println!(" op: {}", debug_pretty(&check_op_a));
    //     let _ = OT::apply(&doc.clone(), &check_op_a);
    // }

    // println!();

    // let mut check_op_b = op_span!([], []);
    // for (i, op) in ops_b.iter().enumerate() {
    //     println!("  B: applying {:?}/{:?}", i + 1, ops_b.len());
    //     check_op_b = Operation::compose(&check_op_b, &op);
    //     println!(" op: {}", debug_pretty(&check_op_b));
    //     let _ = OT::apply(&doc.clone(), &check_op_b);
    // }

    let doc_a = OT::apply(&doc.clone(), &op_a);
    let doc_b = OT::apply(&doc.clone(), &op_b);

    println!("ok");
    println!();

    println!("(!) applying transformed operations...");

    // Tranform
    let (a_, b_) = transform(&op_a, &op_b);

    println!("");
    println!("DOC A {:?}", doc_a);
    println!("OP A' {:?}", a_);
    let a_res = OT::apply(&doc_a, &a_);

    println!("");
    println!("DOC B {:?}", doc_b);
    println!("OP B' {:?}", b_);
    let b_res = OT::apply(&doc_b, &b_);

    println!("");
    println!("a res {:?}", a_res);
    println!("b res {:?}", b_res);

    println!("equal? {:?}", a_res == b_res);

    let success = if a_res != b_res { false } else { true };

    // TODO return error when success is false

    let new_doc = Doc(a_res.0);
    validate_doc_span(ValidateContext::new(), &new_doc.0).expect("Validation error");

    Ok(new_doc)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SyncServerCommand {
    Sync(Vec<Op>, Vec<Op>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SyncClientCommand {
    Update(Option<DocSpan>),
}

pub fn sync_socket_server(state: MoteState) {
    thread::spawn(move || {
        let url = "127.0.0.1:3010";
        ws::listen(url, move |out| {
            // DO the thing

            // Reset
            // Sync
            // Update
            // TODO how do you get the state of the other thing in here?

            // Initial
            {
                let doc = state.body.lock().unwrap();
                let command = SyncClientCommand::Update(Some(doc.0.clone()));
                out.send(
                    serde_json::to_string(&command).unwrap(),
                );
            }

            let state = state.clone();
            move |msg: ws::Message| {
                let req_parse: Result<SyncServerCommand, _> =
                    serde_json::from_slice(&msg.into_data());
                match req_parse {
                    Err(err) => {
                        println!("Packet error: {:?}", err);
                    }
                    Ok(value) => {
                        // native_command(&self.client, value).expect("Native command error");
                        println!("lmao {:?}", value);

                        match value {
                            SyncServerCommand::Sync(ops_a, ops_b) => {
                                let mut doc = state.body.lock().unwrap();
                                if let Ok(new_doc) = action_sync(&*doc, ops_a, ops_b) {
                                    *doc = new_doc.clone();

                                    let command = SyncClientCommand::Update(Some(new_doc.0));
                                    out.send(serde_json::to_string(&command).unwrap());
                                } else {
                                    let command = SyncClientCommand::Update(None);
                                    out.send(serde_json::to_string(&command).unwrap());
                                }
                            }
                        }
                    }
                }

                Ok(())
            }
        })
    });
}
