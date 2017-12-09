#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate oatie;
extern crate rocket;
extern crate rocket_contrib;
extern crate ws;
#[macro_use]
extern crate maplit;

#[macro_use]
extern crate serde_derive;

extern crate serde;
#[macro_use] extern crate serde_json;

use std::sync::{Arc, Mutex};
use oatie::doc::*;
use oatie::{OT, Operation};
use rocket_contrib::Json;
use rocket::State;
use rocket::response::NamedFile;
use serde_json::{Value};
use std::path::{Path, PathBuf};
use std::thread;
use oatie::transform::transform;

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
    body: Arc<Mutex<Doc>>,
}


#[derive(Serialize, Deserialize, Debug)]
pub enum NativeCommand {
    Factorial(u32),
    RenameGroup(String, CurSpan),
    Error(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientCommand {
    Update(DocSpan),
    Error(String),
}

use oatie::stepper::*;
use oatie::writer::*;

fn rename_group_inner(tag: &str, input: &mut CurStepper, doc: &mut DocStepper, del: &mut DelWriter, add: &mut AddWriter) {
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

                // input.enter();
                // doc.enter();
                del.begin();
                add.begin();

                // rename_group_inner(input, doc, del, add);

                //TODO rename the groupppppp
                del.skip(len);
                add.skip(len);
                
                
                // input.exit();
                del.close();
                add.close(hashmap! { "tag".to_string() => tag.to_string() });

                doc.next();
                input.next();
            }
        }
    }
}

fn rename_group(client: &ws::Sender, tag: &str, input: &CurSpan) {
    let doc = default_doc();

    let mut cur_stepper = CurStepper::new(input);
    let mut doc_stepper = DocStepper::new(&doc.0);
    let mut del_writer = DelWriter::new();
    let mut add_writer = AddWriter::new();
    rename_group_inner(tag, &mut cur_stepper, &mut doc_stepper, &mut del_writer, &mut add_writer);
    
    // println!("del {:?}", del_writer.result());
    // println!("add {:?}", add_writer.result());

    let op = (del_writer.result(), add_writer.result());

    let doc = default_doc();
    let new_doc = OT::apply(&doc, &op);
    
    let res = ClientCommand::Update(new_doc.0);
    client.send(serde_json::to_string(&res).unwrap());
}

pub fn native_command(client: &ws::Sender, req: NativeCommand) {
    match req {
        NativeCommand::RenameGroup(tag, cur) => {
            rename_group(client, &tag, &cur);
            // NativeResponse::RenameGroup
        }
        _ => {
            println!("unhandled request: {:?}", req);
        }
    }
}

fn main() {
    thread::spawn(|| {
        ws::listen("127.0.0.1:3012", |out| {
            move |msg: ws::Message| {
                // Handle messages received on this connection
                println!("Server got message '{}'. ", msg);

                let req_parse: Result<NativeCommand, _> = serde_json::from_slice(&msg.into_data());
                native_command(&out, req_parse.unwrap());

                Ok(())
                // out.send(msg)
            }
        }).unwrap();
    });

    rocket::ignite()
        .manage(MoteState {
            body: Arc::new(Mutex::new(default_doc())),
        })
        .mount("/", routes![api_hello, api_confirm, api_sync, api_reset, api_random, root, files])
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
