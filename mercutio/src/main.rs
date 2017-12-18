#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate oatie;
extern crate rocket;
extern crate rocket_contrib;
extern crate ws;
#[macro_use]
extern crate maplit;
extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate rand;

pub mod wasm;
pub mod walkers;

use std::sync::{Arc, Mutex};
use oatie::doc::*;
use oatie::{OT, Operation};
use rocket_contrib::Json;
use rocket::State;
use rocket::response::NamedFile;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::thread;
use oatie::transform::transform;
use wasm::start_websocket_server;

fn default_doc() -> Doc {
    Doc(doc_span![
        DocGroup({"tag": "h1"}, [
            DocChars("Hello! "),
            DocGroup({"tag": "span", "class": "bold"}, [DocChars("what's")]),
            DocChars(" up?"),
        ]),
        DocGroup({"tag": "ul"}, [
            DocGroup({"tag": "li"}, [
                DocGroup({"tag": "p"}, [
                    DocGroup({"tag": "cursor"}, []),
                    DocChars("Three adjectives strong."),
                ]),
                DocGroup({"tag": "p"}, [
                    DocChars("World!"),
                ]),
            ]),
        ])
    ])
}

#[derive(Serialize)]
struct ConfirmResponse {
    ok: bool,
}

type ConfirmInput = (Vec<Op>, Vec<DocElement>);

// TODO should return HTTP error code on failure?
#[post("/api/confirm", data = "<struct_body>")]
fn api_confirm(struct_body: Json<ConfirmInput>, mote: State<MoteState>) -> Json<ConfirmResponse> {
    let (ops, compare_span) = struct_body.0;
    let compare_doc = Doc(compare_span);

    let doc = mote.body.lock().unwrap();
    let start = doc.clone();

    println!("");
    for op in &ops {
        println!("input: op_span!(");
        println!("  {:?},", op.0);
        println!("  {:?},", op.1);
        println!(")");
    }
    println!("");

    let mut op = op_span!([], []);

    let res = if ops.len() > 0 {
        let mut res = start.clone();
        for i in ops.into_iter() {
            println!("combining: op_span!(");
            println!("  {:?},", i.0);
            println!("  {:?},", i.1);
            println!(")");

            op = Operation::compose(&op, &i);

            println!("combined: op_span!(");
            println!("  {:?},", op.0);
            println!("  {:?},", op.1);
            println!(")");

            println!("CMP add {:?}", op.0);
            println!("    del {:?}", op.1);

            println!("start obj {:?}", start);
            println!("apply op {:?}", op);

            res = OT::apply(&start, &op)
        }
        res
    } else {
        start
    };

    println!("COMPARE {:?}", res);
    println!("EXPECTD {:?}", compare_doc);
    println!("success? {:?}", res == compare_doc);

    Json(ConfirmResponse { ok: res == compare_doc })
}

#[derive(Serialize)]
struct RandomResponse {
    ok: bool,
    op: Op,
    doc: DocSpan,
}

type RandomInput = (Vec<Op>, Vec<DocElement>);

// TODO should return HTTP error code on failure?
#[post("/api/random", data = "<struct_body>")]
fn api_random(struct_body: Json<RandomInput>, mote: State<MoteState>) -> Json<RandomResponse> {
    let (ops, compare_span) = struct_body.0;
    let compare_doc = Doc(compare_span);

    let doc = mote.body.lock().unwrap();
    let start = doc.clone();

    println!("");
    for op in &ops {
        println!("input: op_span!(");
        println!("  {:?},", op.0);
        println!("  {:?},", op.1);
        println!(")");
    }
    println!("");

    let mut op = op_span!([], []);

    let res = if ops.len() > 0 {
        let mut res = start.clone();
        for i in ops.into_iter() {
            println!("combining: op_span!(");
            println!("  {:?},", i.0);
            println!("  {:?},", i.1);
            println!(")");

            op = Operation::compose(&op, &i);

            println!("combined: op_span!(");
            println!("  {:?},", op.0);
            println!("  {:?},", op.1);
            println!(")");

            println!("CMP add {:?}", op.0);
            println!("    del {:?}", op.1);

            println!("start obj {:?}", start);
            println!("apply op {:?}", op);

            res = OT::apply(&start, &op)
        }
        res
    } else {
        start
    };

    println!("COMPARE {:?}", res);
    println!("EXPECTD {:?}", compare_doc);
    println!("success? {:?}", res == compare_doc);

    // TODO add op from random generator
    let new_op = op_span!(
        [],
        [AddGroup({"tag": "div"}, [AddSkip(1)])],
    );

    let compare_doc = OT::apply(&compare_doc, &new_op);

    Json(RandomResponse {
        ok: res == compare_doc,
        op: new_op,
        doc: compare_doc.0,
    })
}

type SyncInput = (Vec<Op>, Vec<Op>);

// TODO should return HTTP error code on failure?
#[post("/api/sync", data = "<struct_body>")]
fn api_sync(struct_body: Json<SyncInput>, mote: State<MoteState>) -> Json<ConfirmResponse> {
    let (mut ops_a, mut ops_b) = struct_body.0;

    let mut doc = mote.body.lock().unwrap();

    // Flatten client A operations.
    let op_a = if ops_a.len() == 0 {
        (vec![], vec![])
    } else {
        let mut op = ops_a.remove(0);
        for i in ops_a.into_iter() {
            op = Operation::compose(&op, &i);
        }
        op
    };

    // Flatten client B operations.
    let op_b = if ops_b.len() == 0 {
        (vec![], vec![])
    } else {
        let mut op = ops_b.remove(0);
        for i in ops_b.into_iter() {
            op = Operation::compose(&op, &i);
        }
        op
    };

    println!("OP A {:?}", op_a);
    println!("OP B {:?}", op_b);

    println!("");
    println!("<test>");
    println!("{:?}", op_a.0);
    println!("{:?}", op_a.1);
    println!("");
    println!("{:?}", op_b.0);
    println!("{:?}", op_b.1);
    println!("</test>");
    println!("");

    // Tranform
    let (a_, b_) = transform(&op_a, &op_b);

    println!("testing...");
    println!("");

    let doc_a = OT::apply(&doc.clone(), &op_a);
    let doc_b = OT::apply(&doc.clone(), &op_b);

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

    let success = if a_res != b_res {
        false
    } else {
        *doc = a_res.clone();
        true
    };

    Json(ConfirmResponse { ok: success })
}

#[get("/api/hello")]
fn api_hello(mote: State<MoteState>) -> Json<DocSpan> {
    let doc = mote.body.lock().unwrap();
    Json(doc.clone().0)
}

#[post("/api/reset")]
fn api_reset(mote: State<MoteState>) -> Json<Value> {
    let mut doc = mote.body.lock().unwrap();
    *doc = default_doc();
    Json(json!({
        "ok": true,
    }))
}

#[get("/")]
fn root() -> Option<NamedFile> {
    Path::new(file!())
        .parent()
        .map(|x| x.join("templates/").join("index.html"))
        .and_then(|x| NamedFile::open(x).ok())
}

#[get("/client")]
fn client() -> Option<NamedFile> {
    Path::new(file!())
        .parent()
        .map(|x| x.join("templates/").join("client.html"))
        .and_then(|x| NamedFile::open(x).ok())
}

#[get("/<file..>", rank = 2)]
fn files(file: PathBuf) -> Option<NamedFile> {
    Path::new(file!())
        .parent()
        .and_then(|x| x.parent())
        .map(|x| x.join("frontend/dist/").join(file))
        .and_then(|x| NamedFile::open(x).ok())
}

struct MoteState {
    body: Arc<Mutex<Doc>>,
}

fn main() {
    thread::spawn(|| {
        start_websocket_server();
    });

    rocket::ignite()
        .manage(MoteState { body: Arc::new(Mutex::new(default_doc())) })
        .mount(
            "/",
            routes![
                api_hello,
                api_confirm,
                api_sync,
                api_reset,
                api_random,
                root,
                client,
                files,
            ],
        )
        .launch();
}

// fn main_2() {
//     println!("yeah");

//     let doc = doc_span![DocGroup({"tag": "ul"}, [DocGroup({"tag": "li"}, [DocGroup({"tag": "h1"}, [DocChars("Hello! "), DocGroup({"tag": "b"}, [DocChars("what\'s")]), DocChars(" up?")]), DocGroup({"tag": "p"}, [DocChars("World!")])])])];

//     let mut ops: Vec<Op> = vec![
//         op_span!(
//             [DelWithGroup([DelWithGroup([DelWithGroup([DelSkip(6), DelChars(1)])])])],
//             [],
//         ),
//         op_span!(
//             [DelWithGroup([DelWithGroup([DelGroup([DelSkip(11)])])])],
//             [AddWithGroup([AddWithGroup([AddGroup({"tag": "h1"}, [AddSkip(6)]), AddGroup({"tag": "p"}, [AddSkip(5)])])])],
//         ),
//         op_span!(
//             [],
//             [AddWithGroup([AddWithGroup([AddSkip(1), AddWithGroup([AddWithGroup([AddSkip(1), AddChars("W")])])])])],
//         ),
//         op_span!(
//             [DelWithGroup([DelWithGroup([DelSkip(1), DelWithGroup([DelWithGroup([DelChars(1)])])])])],
//             [],
//         ),
//     ];

//     let mut op = op_span!([], []);
//     for i in ops.into_iter() {

//         println!("compose: op_span!(");
//         println!("  {:?},", i.0);
//         println!("  {:?},", i.1);
//         println!(")");

//         op = compose(&op, &i);

//         println!("applying: op_span!(");
//         println!("  {:?},", op.0);
//         println!("  {:?},", op.1);
//         println!(")");
//         let out = apply_operation(&doc, &op);

//         println!("doc: {:?}", out);
//     }

// // CMP add [DelWithGroup([DelWithGroup([DelGroup([DelWithGroup([DelChars(1), DelSkip(1)]), DelSkip(5), DelChars(1), DelSkip(5)])])])]
// //     del [AddWithGroup([AddWithGroup([AddGroup({"tag": "h1"}, [AddSkip(6)]), AddGroup({"tag": "p"}, [AddWithGroup([AddChars("W")]), AddSkip(4)])])])]
// // start obj [DocGroup({"tag": "ul"}, [DocGroup({"tag": "li"}, [DocGroup({"tag": "h1"}, [DocChars("Hello! "), DocGroup({"tag": "b"}, [DocChars("what\'s")]), DocChars(" up?")]), DocGroup({"tag": "p"}, [DocChars("World!")])])])]
// }
