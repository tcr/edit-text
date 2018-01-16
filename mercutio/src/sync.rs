use failure::Error;
use oatie::{Operation, OT};
use oatie::parse::debug_pretty;
use oatie::doc::*;
use oatie::validate::{validate_doc_span, ValidateContext};
use oatie::transform::transform;
use serde_json;
use std::collections::HashMap;
use bus::Bus;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
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

    let test = format!(
        r#"
doc:   {}

a_del: {}
a_add: {}

b_del: {}
b_add: {}
"#,
        debug_pretty(&doc.0),
        debug_pretty(&op_a.0),
        debug_pretty(&op_a.1),
        debug_pretty(&op_b.0),
        debug_pretty(&op_b.1)
    );

    // TODO dump to document
    {
        use std::io::prelude::*;
        let mut f = ::std::fs::File::create("test.txt").unwrap();
        f.write_all(&test.as_bytes()).unwrap();
        f.sync_all().unwrap();
    }

    println!();
    println!("<test>");
    print!("{}", test);
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

pub fn sync_socket_server(port: u16, period: usize, state: MoteState) {
    thread::spawn(move || {
        let url = format!("127.0.0.1:{}", port);

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
                    // Wait a set duration between transforms.
                    thread::sleep(Duration::from_millis(period as u64));

                    // Extract the client map and dump active client operations.
                    let mut sync_state = sync_state_mutex_capture.lock().unwrap();
                    let mut keys: Vec<_> = sync_state.ops.keys().cloned().collect();
                    keys.sort();
                    if keys.is_empty() {
                        continue;
                    }

                    // List the clients.
                    println!("evaluating: {:?}", keys);

                    // Perform the document operation transformation.
                    let mut doc = state_capture.body.lock().unwrap();
                    let mut new_doc = doc.clone();
                    let mut new_op = vec![op_span!([], [])];
                    for op_group in keys.iter().map(|x| sync_state.ops.remove(x).unwrap()) {
                        let res = action_sync(&doc, new_op, op_group).unwrap();
                        new_doc = res.0;
                        new_op = vec![res.1];
                    }
                    *doc = new_doc;

                    // let mut doc = state_capture.body.lock().unwrap();
                    // let (_, new_op) = action_sync(&doc, left_ops, middle_ops).unwrap();
                    // let (new_doc, _) = action_sync(&doc, vec![new_op], right_ops).unwrap();
                    // *doc = new_doc;

                    // Increase version
                    sync_state.version += 1;

                    bus_capture
                        .lock()
                        .unwrap()
                        .broadcast((doc.0.clone(), sync_state.version));
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

            let mut rx = { bus_capture.lock().unwrap().add_rx() };
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
                    Ok(value) => {
                        match value {
                            SyncServerCommand::Commit(client_id, op, version) => {
                                let mut sync_state = sync_state_mutex_capture.lock().unwrap();
                                // TODO remove hack version == 0 which lets us add carets from all parties
                                if version == 0 || version == sync_state.version {
                                    sync_state.ops.entry(client_id).or_insert(vec![]).push(op);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        println!("Packet error: {:?}", err);
                    }
                }

                Ok(())
            }
        })
    });
}
