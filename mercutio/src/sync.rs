use failure::Error;
use oatie::{Operation, OT};
use oatie::debug_pretty;
use oatie::doc::*;
use oatie::schema::{validate_doc_span, ValidateContext};
use oatie::transform::transform;
use rocket_contrib::Json;
use rocket::response::NamedFile;
use rocket::State;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use bus::Bus;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use wasm::start_websocket_server;
use ws;
use std::{panic, process};

pub fn default_doc() -> Doc {
    Doc(doc_span![
        DocGroup({"tag": "h1"}, [
            // DocGroup({"tag": "caret", "client": "left"}, []),
            // DocGroup({"tag": "caret", "client": "right"}, []),
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

pub fn action_sync(doc: &Doc, ops_a: Vec<Op>, ops_b: Vec<Op>) -> Result<(Doc, Op), Error> {
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
    let mut validate_ctx = ValidateContext::new();
    validate_doc_span(&mut validate_ctx, &new_doc.0).expect("Validation error");

    Ok((new_doc, Operation::compose(&op_a, &a_)))
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SyncServerCommand {
    // Connect(String),
    Commit(String, Op, usize),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SyncClientCommand {
    Update(DocSpan, usize),
}

pub struct SyncState {
    ops: HashMap<String, Vec<Op>>,
    version: usize,
}

pub fn sync_socket_server(state: MoteState) {
    thread::spawn(move || {
        let url = "127.0.0.1:3010";

        let sync_state_mutex = Arc::new(Mutex::new(SyncState {
            ops: hashmap![],
            version: 100,
        }));

        let bus = Arc::new(Mutex::new(Bus::new(255)));

        let sync_state_mutex_capture = sync_state_mutex.clone();
        let state_capture = state.clone();
        let bus_capture = bus.clone();
        thread::spawn(move || {
            if let Err(value) = panic::catch_unwind(|| {
                loop {
                    // wait 1s
                    thread::sleep(Duration::from_millis(100));

                    // Attempt to extract client map
                    let mut sync_state = sync_state_mutex_capture.lock().unwrap();
                    let left_ops = sync_state.ops.remove("left").unwrap_or(vec![]);
                    let middle_ops = sync_state.ops.remove("middle").unwrap_or(vec![]);
                    let right_ops = sync_state.ops.remove("right").unwrap_or(vec![]);
                    if left_ops.is_empty() && middle_ops.is_empty() && right_ops.is_empty() {
                        continue;
                    }

                    // TODO generally extract client ops, then merge in
                    // a generalized action op
                    
                    // Do transform
                    let mut doc = state_capture.body.lock().unwrap();
                    let (_, new_op) = action_sync(&doc, left_ops, right_ops).unwrap();
                    let (new_doc, _) = action_sync(&doc, vec![new_op], middle_ops).unwrap();
                    // let (new_doc, _) = action_sync(&doc, left_ops, right_ops).unwrap();
                    *doc = new_doc;

                    // Increase version
                    sync_state.version += 1;

                    bus_capture.lock().unwrap().broadcast((doc.0.clone(), sync_state.version));
                }
            }) {
                println!("Error: {:?}", value);
                process::exit(1);
            }
        });

        let state_capture = state.clone();
        let bus_capture = bus.clone();
        ws::listen(url, move |out| {
            // Initial document state.
            {
                let doc = state.body.lock().unwrap();
                let command = SyncClientCommand::Update(doc.0.clone(), 100);
                out.send(serde_json::to_string(&command).unwrap());
            }

            let mut rx = {
                bus_capture.lock().unwrap().add_rx()
            };
            thread::spawn(move || {
                while let Ok((doc, version)) = rx.recv() {
                    let command = SyncClientCommand::Update(doc, version);
                    out.send(serde_json::to_string(&command).unwrap());
                }
            });

            let state = state.clone();
            let sync_state_mutex_capture = sync_state_mutex.clone();
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
                            // SyncServerCommand::Connect(client_id) => {
                            //     let mut state = state_mutex.lock().unwrap();
                            //     *state..get_mut(&client_id).unwrap() = vec![];
                            // }
                            SyncServerCommand::Commit(client_id, op, version) => {
                                let mut sync_state = sync_state_mutex_capture.lock().unwrap();
                                // TODO remove hack version == 0 which lets us add carets from all parties
                                if version == 0 || version == sync_state.version {
                                    sync_state.ops.entry(client_id).or_insert(vec![]).push(op);
                                }
                            }
                            // SyncServerCommand::Sync(ops_a, ops_b) => {
                            //     let mut doc = state.body.lock().unwrap();
                            //     if let Ok(new_doc) = action_sync(&*doc, ops_a, ops_b) {
                            //         *doc = new_doc.clone();

                            //         let command = SyncClientCommand::Update(Some(new_doc.0));
                            //         out.send(serde_json::to_string(&command).unwrap());
                            //     } else {
                            //         let command = SyncClientCommand::Update(None);
                            //         out.send(serde_json::to_string(&command).unwrap());
                            //     }
                            // }
                        }
                    }
                }

                Ok(())
            }
        })
    });
}
