#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate oatie;
extern crate rocket;
extern crate rocket_contrib;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use std::sync::{Arc, Mutex};
use oatie::doc::*;
use oatie::apply_operation;
use oatie::compose::compose;
use rocket_contrib::Json;
use rocket::State;
use rocket::response::NamedFile;
use std::path::{Path, PathBuf};
use oatie::transform::transform;

fn default_doc() -> DocElement {
    doc_span![DocGroup({"tag": "ul"}, [
        DocGroup({"tag": "li"}, [
            DocGroup({"tag": "h1"}, [
                DocChars("Hello! "),
                DocGroup({"tag": "span", "class": "bold"}, [DocChars("what's")]),
                DocChars(" up?"),
            ]),
            DocGroup({"tag": "p"}, [
                DocChars("World!"),
            ]),
        ]),
    ])].pop()
        .unwrap()
}

#[derive(Serialize)]
struct ConfirmResponse {
    ok: bool,
}

type ConfirmInput = (Vec<Op>, Vec<DocElement>);

// TODO should return HTTP error code on failure?
#[post("/api/confirm", data = "<struct_body>")]
fn api_confirm(struct_body: Json<ConfirmInput>, mote: State<MoteState>) -> Json<ConfirmResponse> {
    let (ops, compare_doc) = struct_body.0;

    let doc = mote.body.lock().unwrap();
    let start = vec![doc.clone()];

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

            op = compose(&op, &i);

            println!("combined: op_span!(");
            println!("  {:?},", op.0);
            println!("  {:?},", op.1);
            println!(")");

            println!("CMP add {:?}", op.0);
            println!("    del {:?}", op.1);

            println!("start obj {:?}", start);
            println!("apply op {:?}", op);

            res = apply_operation(&start, &op)
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
            op = compose(&op, &i);
        }
        op
    };

    // Flatten client B operations.
    let op_b = if ops_b.len() == 0 {
        (vec![], vec![])
    } else {
        let mut op = ops_b.remove(0);
        for i in ops_b.into_iter() {
            op = compose(&op, &i);
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

    let doc_a = apply_operation(&vec![doc.clone()], &op_a);
    let doc_b = apply_operation(&vec![doc.clone()], &op_b);

    println!("");
    println!("DOC A {:?}", doc_a);
    println!("OP A' {:?}", a_);
    let a_res = apply_operation(&doc_a, &a_);

    println!("");
    println!("DOC B {:?}", doc_b);
    println!("OP B' {:?}", b_);
    let b_res = apply_operation(&doc_b, &b_);

    println!("");
    println!("a res {:?}", a_res);
    println!("b res {:?}", b_res);

    println!("equal? {:?}", a_res == b_res);

    let success = if a_res != b_res {
        false
    } else {
        *doc = a_res[0].clone();
        true
    };

    Json(ConfirmResponse { ok: success })
}

#[get("/api/hello")]
fn api_hello(mote: State<MoteState>) -> Json<DocElement> {
    let doc = mote.body.lock().unwrap();
    Json(doc.clone())
}

#[get("/")]
fn root() -> Option<NamedFile> {
    Path::new(file!())
        .parent()
        .map(|x| x.join("static/").join("index.html"))
        .and_then(|x| NamedFile::open(x).ok())
}

#[get("/<file..>", rank = 2)]
fn files(file: PathBuf) -> Option<NamedFile> {
    Path::new(file!())
        .parent()
        .map(|x| x.join("static/").join(file))
        .and_then(|x| NamedFile::open(x).ok())
}

struct MoteState {
    body: Arc<Mutex<DocElement>>,
}

fn main() {
    rocket::ignite()
        .manage(MoteState {
            body: Arc::new(Mutex::new(default_doc())),
        })
        .mount("/", routes![api_hello, api_confirm, api_sync, root, files])
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
