#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate failure;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate oatie;
extern crate rand;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate ws;
extern crate take_mut;

pub mod wasm;

use std::sync::{Arc, Mutex};
use oatie::doc::*;
use oatie::{Operation, OT};
use rocket_contrib::Json;
use rocket::State;
use rocket::response::NamedFile;
use serde_json::Value;
use std::path::{Path, PathBuf};
use oatie::transform::transform;
use oatie::debug_pretty;
use wasm::start_websocket_server;

fn default_doc() -> Doc {
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

    Json(ConfirmResponse {
        ok: res == compare_doc,
    })
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

#[derive(Serialize)]
struct SyncResponse {
    ok: bool,
    doc: DocSpan,
}

type SyncInput = (Vec<Op>, Vec<Op>);

// TODO should return HTTP error code on failure?
#[post("/api/sync", data = "<struct_body>")]
fn api_sync(struct_body: Json<SyncInput>, mote: State<MoteState>) -> Json<SyncResponse> {
    let (ops_a, ops_b) = struct_body.0;

    let mut doc = mote.body.lock().unwrap();

    println!(" ---> input ops_a");
    println!("{:?}", ops_a);
    println!();

    // Flatten client A operations.
    let mut op_a = op_span!([], []);
    for op in &ops_a {
        op_a = Operation::compose(&op_a, op);
    };

    println!(" ---> input ops_b");
    println!("{:?}", ops_b);
    println!();

    // Flatten client B operations.
    let mut op_b = op_span!([], []);
    for op in &ops_b {
        op_b = Operation::compose(&op_b, op);
    };

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

    let success = if a_res != b_res {
        false
    } else {
        *doc = a_res.clone();
        true
    };

    Json(SyncResponse {
        ok: success,
        doc: a_res.0,
    })
}

/// Return the initial "hello world" document.
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
    Some(Path::new(".")
        .join("src/templates/")
        .join("index.html"))
        .and_then(|x| NamedFile::open(x).ok())
}

#[get("/client")]
fn client() -> Option<NamedFile> {
    Some(Path::new(".")
        .join("src/templates/")
        .join("client.html"))
        .and_then(|x| NamedFile::open(x).ok())
}

#[get("/favicon.png")]
fn favicon() -> Option<NamedFile> {
    Some(Path::new(".")
        .join("src/templates/")
        .join("favicon.png"))
        .and_then(|x| NamedFile::open(x).ok())
}

#[get("/<file..>", rank = 2)]
fn files(file: PathBuf) -> Option<NamedFile> {
    Some(Path::new(".")
        .join("frontend/dist/")
        .join(file))
        .and_then(|x| NamedFile::open(x).ok())
}

struct MoteState {
    body: Arc<Mutex<Doc>>,
}
fn main() {
    start_websocket_server();

    rocket::ignite()
        .manage(MoteState {
            body: Arc::new(Mutex::new(default_doc())),
        })
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
                favicon,
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
