extern crate iron;
extern crate staticfile;
extern crate mount;
extern crate router;
extern crate oatie;
extern crate rustc_serialize;
extern crate bodyparser;
#[macro_use] extern crate literator;

// This example serves the docs from target/doc/staticfile at /doc/
//
// Run `cargo doc && cargo test && ./target/doc_server`, then
// point your browser to http://127.0.0.1:3000/doc/

use std::path::Path;

use iron::mime::Mime;
use iron::status;
use iron::prelude::*;

use mount::Mount;
use router::Router;
use staticfile::Static;
use rustc_serialize::json;

use oatie::doc::*;
use oatie::compose::compose;
use oatie::transform::transform_insertions;
use oatie::apply_operation;

fn default_doc() -> DocElement {
    DocGroup(container! { ("tag".into(), "ul".into()) }, vec![
        DocGroup(container! { ("tag".into(), "li".into()) }, vec![
            DocGroup(container! { ("tag".into(), "h1".into()) }, vec![
                DocChars("Hello!".into()),
            ]),
            DocGroup(container! { ("tag".into(), "p".into()) }, vec![
                DocChars("World!".into()),
            ]),
        ]),
    ])
}

fn say_hello(req: &mut Request) -> IronResult<Response> {
    println!("Running send_hello handler, URL path: {}", req.url.path.join("/"));

    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((content_type, status::Ok, json::encode(&default_doc()).unwrap())))
}

type TestAlias = (Vec<Op>, Vec<DocElement>);

fn test_thing(req: &mut Request) -> IronResult<Response> {
    println!("Running test_thing handler, URL path: {}", req.url.path.join("/"));

    let struct_body = req.get::<bodyparser::Struct<TestAlias>>();
    let success = match struct_body {
        Ok(Some(struct_body)) => {
            let (mut ops, doc) = struct_body;
            let start = vec![default_doc()];

            let res = if ops.len() > 0 {
                let mut op = ops.remove(0);
                for i in ops.into_iter() {
                    op = compose(&op, &i);
                }

                println!("start obj {:?}", start);
                println!("apply op {:?}", op);

                apply_operation(&start, &op)
            } else {
                start
            };

            println!("COMPARE {:?}", res);
            println!("EXPECTD {:?}", doc);
            println!("success? {:?}", res == doc);

            res == doc
        }
        Ok(None) => {
            println!("No body");
            false
        },
        Err(err) => {
            println!("Error: {:?}", err);
            false
        }
    };

    let content_type = "application/json".parse::<Mime>().unwrap();
    if success {
        Ok(Response::with((content_type, status::Ok, "{\"ok\": true}")))
    } else {
        Ok(Response::with((content_type, status::BadRequest, "{\"ok\": false}")))
    }
}

type SyncAlias = (Vec<Op>, Vec<Op>);

fn sync_thing(req: &mut Request) -> IronResult<Response> {
    println!("Running sync thing handler, URL path: {}", req.url.path.join("/"));

    let struct_body = req.get::<bodyparser::Struct<SyncAlias>>();
    let success = match struct_body {
        Ok(Some(struct_body)) => {
            let (mut ops_a, mut ops_b) = struct_body;

            let op_a = if ops_a.len() == 0 {
                (vec![], vec![])
            } else {
                let mut op = ops_a.remove(0);
                for i in ops_a.into_iter() {
                    op = compose(&op, &i);
                }
                op
            };

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

            let (_, ins_a) = op_a.clone();
            let (_, ins_b) = op_b.clone();

            let (a_, b_) = transform_insertions(&ins_a, &ins_b);

            println!("OP A' {:?}", a_);
            println!("OP B' {:?}", b_);

            println!("testing...");

            let doc_a = apply_operation(&vec![default_doc()], &op_a);
            let doc_b = apply_operation(&vec![default_doc()], &op_b);
            println!("DOC A {:?}", doc_a);
            println!("DOC B {:?}", doc_b);

            let a_res = apply_operation(&doc_a, &a_);
            let b_res = apply_operation(&doc_b, &b_);

            println!("a res {:?}", a_res);
            println!("b res {:?}", b_res);

            println!("equal? {:?}", a_res == b_res);

            a_res == b_res
        }
        Ok(None) => {
            println!("No body");
            false
        },
        Err(err) => {
            println!("Error: {:?}", err);
            false
        }
    };

    let content_type = "application/json".parse::<Mime>().unwrap();
    if success {
        Ok(Response::with((content_type, status::Ok, "{\"ok\": true}")))
    } else {
        Ok(Response::with((content_type, status::BadRequest, "{\"ok\": false}")))
    }
}


fn main() {
    // let a: TestAlias = (vec![], vec![DocGroup(container! { }, vec![DocChars("hi".into())])]);
    // println!("lets see it: {:?}", json::encode(&a).unwrap());

    let mut router = Router::new();
    router
        .get("/hello", say_hello)
        .post("/confirm", test_thing)
        .post("/sync", sync_thing);

    let mut mount = Mount::new();

    mount.mount("/", Static::new(Path::new("src/static/")));
    mount.mount("/api", router);

    println!("Doc server running on http://localhost:3000/");

    Iron::new(mount).http("127.0.0.1:3000").unwrap();
}
