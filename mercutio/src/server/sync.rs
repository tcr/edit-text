use bus::Bus;
use failure::Error;
use oatie::OT;
use oatie::doc::*;
use oatie::parse::debug_pretty;
use oatie::schema::RtfSchema;
use oatie::transform::transform;
use oatie::validate::validate_doc;
use serde_json;
use std::{panic, process};
use std::collections::{HashSet, HashMap};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::{SyncClientCommand, SyncServerCommand};
use ws;
use std::collections::VecDeque;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::Instant;
use crate::markdown::markdown_to_doc;
use rand::{thread_rng, Rng};
use ron;

pub fn default_doc() -> Doc {
    const INPUT: &'static str = r#"

# Hello world!

This is edit-text, a web-based rich text editor.

* Version 0.1. This is a very early preview. :)

* Supports collaborative editing.

* Written in Rust and cross-compiled to WebAssembly.

* Supports Markdown export.

This app is very easy to break! That's okay though. We'll notice and fix it, and it'll break less in the future.

---

Type github.com/tcr/edit-text into your search bar for more information.

"#;

    // Should be no errors
    let doc = Doc(markdown_to_doc(&INPUT).unwrap());
    validate_doc(&doc).expect("Initial Markdown document was malformed");
    doc

    // Doc(doc_span![
    //     DocGroup({"tag": "h1"}, [
    //         // DocGroup({"tag": "caret", "client": "left"}, []),
    //         // DocGroup({"tag": "caret", "client": "right"}, []),
    //         DocChars("Hello world!"),
    //     ]),
    //     DocGroup({"tag": "p"}, [
    //         // DocChars("What's "),
    //         // DocGroup({"tag": "span", "class": "bold"}, [DocChars("new and great")]),
    //         // DocChars(" with you?"),
    //         DocChars("This is Mercutio, a rich text editor."),
    //     ]),
    //     // DocGroup({"tag": "ul"}, [
    //     //     DocGroup({"tag": "li"}, [
    //     //         DocGroup({"tag": "p"}, [
    //     //             DocChars("Three adjectives strong."),
    //     //         ]),
    //     //         DocGroup({"tag": "p"}, [
    //     //             DocChars("World!"),
    //     //         ]),
    //     //     ]),
    //     // ])
    // ])
}

#[derive(Clone)]
pub struct MoteState {
    pub body: Arc<Mutex<Doc>>,
}

// pub fn action_sync(doc: &Doc, ops_a: Vec<Op>, ops_b: Vec<Op>) -> Result<(Doc, Op), Error> {
//     println!(" ---> input ops_a");
//     println!("{:?}", ops_a);
//     println!();

//     // Flatten client A operations.
//     let mut op_a = op_span!([], []);
//     for op in &ops_a {
//         op_a = OT::compose(&op_a, op);
//     }

//     println!(" ---> input ops_b");
//     println!("{:?}", ops_b);
//     println!();

//     // Flatten client B operations.
//     let mut op_b = op_span!([], []);
//     for op in &ops_b {
//         op_b = OT::compose(&op_b, op);
//     }

//     println!("OP A {:?}", op_a);
//     println!("OP B {:?}", op_b);

//     let test = format!(
//         r#"
// doc:   {}

// a_del: {}
// a_add: {}

// b_del: {}
// b_add: {}
// "#,
//         debug_pretty(&doc.0),
//         debug_pretty(&op_a.0),
//         debug_pretty(&op_a.1),
//         debug_pretty(&op_b.0),
//         debug_pretty(&op_b.1)
//     );

//     // TODO dump to document
//     {
//         use std::io::prelude::*;
//         let mut f = ::std::fs::File::create("test.txt").unwrap();
//         f.write_all(&test.as_bytes()).unwrap();
//         f.sync_all().unwrap();
//     }

//     println!();
//     println!("<test>");
//     print!("{}", test);
//     println!("</test>");
//     println!();

//     println!("(!) recreating initial client state...");
//     println!();

//     // TODO remove this validation code if we're performing the check client-side

//     // let mut check_op_a = op_span!([], []);
//     // for (i, op) in ops_a.iter().enumerate() {
//     //     println!("  A: applying {:?}/{:?}", i + 1, ops_a.len());
//     //     check_op_a = OT::compose(&check_op_a, &op);
//     //     println!(" op: {}", debug_pretty(&check_op_a));
//     //     let _ = OT::apply(&doc.clone(), &check_op_a);
//     // }

//     // println!();

//     // let mut check_op_b = op_span!([], []);
//     // for (i, op) in ops_b.iter().enumerate() {
//     //     println!("  B: applying {:?}/{:?}", i + 1, ops_b.len());
//     //     check_op_b = OT::compose(&check_op_b, &op);
//     //     println!(" op: {}", debug_pretty(&check_op_b));
//     //     let _ = OT::apply(&doc.clone(), &check_op_b);
//     // }

//     let doc_a = OT::apply(&doc.clone(), &op_a);
//     let doc_b = OT::apply(&doc.clone(), &op_b);

//     println!("ok");
//     println!();

//     println!("(!) applying transformed operations...");

//     // Tranform
//     let (a_, b_) = transform::<RtfSchema>(&op_a, &op_b);

//     println!("");
//     println!("DOC A {:?}", doc_a);
//     println!("OP A' {:?}", a_);
//     let a_res = OT::apply(&doc_a, &a_);

//     println!("");
//     println!("DOC B {:?}", doc_b);
//     println!("OP B' {:?}", b_);
//     let b_res = OT::apply(&doc_b, &b_);

//     println!("");
//     println!("a res {:?}", a_res);
//     println!("b res {:?}", b_res);

//     println!("equal? {:?}", a_res == b_res);

//     let success = if a_res != b_res { false } else { true };

//     // TODO return error when success is false

//     let new_doc = Doc(a_res.0);
//     validate_doc(&new_doc).expect("Validation error");

//     Ok((new_doc, OT::compose(&op_a, &a_)))
// }

pub struct SyncState {
    ops: VecDeque<(String, usize, Op)>,
    version: usize,
    history: HashMap<usize, Op>,
}

/// Transform an operation incrementally against each interim document operation.
pub fn update_operation(mut op: Op, history: &HashMap<usize, Op>, target_version: usize, mut input_version: usize) -> Op {
    // Transform against each interim operation.
    // TODO upgrade_operation_to_current or something
    while input_version < target_version {
        // If the version exists (it should) transform against it.
        if let Some(ref version_op) = history.get(&input_version) {
            let (updated_op, _) = Op::transform::<RtfSchema>(version_op, &op);
            // let correction = correct_op(&updated_op).unwrap();
            // op = OT::compose(&updated_op, &correction);
            op = updated_op;
        }

        input_version += 1;
    }
    op  
}

pub fn sync_socket_server(port: u16, period: usize, state: MoteState) {
    log_sync!(Spawn);

    let url = format!("0.0.0.0:{}", port);

    println!("Listening sync_socket_server on 0.0.0.0:{}", port);

    let sync_state_mutex = Arc::new(Mutex::new(SyncState {
        ops: VecDeque::new(),
        version: 100,
        history: hashmap![],
    }));

    let bus = Arc::new(Mutex::new(Bus::new(255)));

    // Handle incoming packets.
    let join_process = thread::Builder::new().name("sync_thread_processor".into()).spawn({
        take!(=state, =bus, =sync_state_mutex);
        move || {
            loop {
                // Wait a set duration between transforms.
                thread::sleep(Duration::from_millis(period as u64));

                let now = Instant::now();

                let mut sync_state = sync_state_mutex.lock().unwrap();

                let mut doc = state.body.lock().unwrap();

                // Go through the deque and update our operations.
                while let Some((client_id, input_version, op)) = sync_state.ops.pop_front() {
                    let target_version = sync_state.version;

                    log_sync!(Debug(format!("client {:?} sent {:?}", client_id, op)));
                    
                    // let res = action_sync(&doc, new_op, op_group).unwrap();

                    // Update the operation so we can apply it to the document.
                    let op = update_operation(op, &sync_state.history, target_version, input_version);
                    sync_state.history.insert(target_version, op.clone());

                    log_sync!(Debug(format!("updated op to {:?}", op)));

                    // Update the document with this operation.
                    *doc = Op::apply(&doc, &op);
                    sync_state.version = target_version + 1;
                    
                    validate_doc(&doc).expect("Validation error");

                    log_sync!(Debug(format!("doc is now {:?}", *doc)));

                    // Broadcast to all connected websockets.
                    let command = SyncClientCommand::Update(doc.0.clone(), sync_state.version, client_id, op);
                    bus
                        .lock()
                        .unwrap()
                        .broadcast(command);
                }

                let elapsed = now.elapsed();
                // println!("sync duration: {}s, {}us", elapsed.as_secs(), elapsed.subsec_nanos()/1_000);
            }
        }
    });

    // Listen to incoming clients.
    ws::listen(url, {
        take!(=state, =bus);
        move |out| {
            log_sync!(ClientConnect);

            println!("Client connected.");

            // Forcibly set this new client's initial document state.
            {
                // TODO how to select from unused client IDs?
                let new_id = thread_rng().gen_ascii_chars().take(6).collect::<String>();

                let doc = state.body.lock().unwrap();
                let mut sync_state = sync_state_mutex.lock().unwrap();
                let command = SyncClientCommand::Init(new_id.clone(), doc.0.clone(), sync_state.version);
                out.send(serde_json::to_string(&command).unwrap());
            }

            // Forward packets from sync to all clients.
            let mut rx = { bus.lock().unwrap().add_rx() };
            thread::spawn(|| {
                take!(out, mut rx);
                while let Ok(command) = rx.recv() {
                    out.send(serde_json::to_string(&command).unwrap());
                }
            });

            // Listen to commands from the clients and submit to sync server.
            let state = state.clone();
            let sync_state_mutex_capture = sync_state_mutex.clone();
            move |msg: ws::Message| {
                let req_parse: Result<SyncServerCommand, _> = serde_json::from_slice(&msg.into_data());
                match req_parse {
                    Ok(value) => {
                        log_sync!(ClientPacket(value.clone()));

                        match value {
                            SyncServerCommand::Keepalive => {
                                // noop
                            }
                            SyncServerCommand::Commit(client_id, op, version) => {
                                let mut sync_state = sync_state_mutex_capture.lock().unwrap();
                                sync_state.ops.push_back((client_id, version, op));
                            }
                        }
                    }
                    Err(err) => {
                        println!("Packet error: {:?}", err);
                    }
                }

                Ok(())
            }
        }
    });

    // join_process.join();
}
