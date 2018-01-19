extern crate bus;
extern crate crossbeam_channel;
extern crate failure;
extern crate maplit;
extern crate mercutio;
extern crate oatie;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate take_mut;
extern crate tiny_http;
extern crate url;
extern crate ws;
extern crate uuid;

use mercutio::sync::*;
use std::env;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::thread;
use structopt::StructOpt;
use tiny_http::{Header, Response};
use url::Url;
use uuid::Uuid;

fn spawn_http_server(port: u16) {
    let server = tiny_http::Server::http(&format!("0.0.0.0:{}", port)).unwrap();

    let server = Arc::new(server);
    let mut guards = Vec::with_capacity(4);

    for _ in 0..4 {
        let server = server.clone();

        let guard = thread::spawn(move || {
            let root_path = env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_owned();
            let dist_path = root_path
                .join("mercutio/frontend/dist/")
                .canonicalize()
                .unwrap();
            let template_path = root_path
                .join("mercutio/frontend/templates/")
                .canonicalize()
                .unwrap();

            loop {
                let req = server.recv().unwrap();

                let path = Url::parse("http://localhost/")
                    .unwrap()
                    .join(req.url())
                    .unwrap()
                    .path()
                    .to_owned();

                match path.as_ref() {
                    "/" | "/index.html" => {
                        let my_uuid = Uuid::new_v4().to_string();
                        // Redirect as random client
                        let mut res = Response::empty(301);
                        let dest = format!("/client/?{}", &my_uuid[0..8]);
                        let mut h = Header::from_bytes(b"Location".to_vec(), dest.as_bytes()).unwrap();
                        res.add_header(h);
                        let _ = req.respond(res);
                    }
                    "/multi" | "/multi/" => {
                        let path = template_path.join("index.html");
                        let file = File::open(&path).unwrap();
                        let _ = req.respond(Response::from_file(file));
                    }
                    "/client" | "/client/" => {
                        let path = template_path.join("client.html");
                        let file = File::open(&path).unwrap();
                        let _ = req.respond(Response::from_file(file));
                    }
                    "/favicon.png" => {
                        let path = template_path.join("favicon.png");
                        let file = File::open(&path).unwrap();
                        let _ = req.respond(Response::from_file(file));
                    }
                    path => {
                        if let Some(target) = dist_path
                            .join(path.chars().skip(1).collect::<String>())
                            .canonicalize()
                            .ok()
                            .and_then(|x| {
                                if x.starts_with(&dist_path) {
                                    Some(x)
                                } else {
                                    None
                                }
                            }) {
                            println!("GET 200 {:?}", path);
                            let file = File::open(&target).unwrap();
                            let _ = req.respond(Response::from_file(file));
                        } else {
                            println!("GET 404 {:?}", path);
                            let _ = req.respond(Response::from_string("404".to_owned()));
                        }
                    }
                }

                // let file = File::open(&Path::new("image.png")).unwrap();
                // let response = tiny_http::Response::from_file(file);
                // let _ = request.respond(response);
            }
        });

        guards.push(guard);
    }

    println!("Listening on http://localhost:{}/", port);

    for guard in guards {
        let _ = guard.join();
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "mercutio-wasm", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(long = "port", help = "Port", default_value = "8000")] port: u16,

    #[structopt(long = "period", help = "Sync period", default_value = "50")] period: usize,
}

fn main() {
    let opt = Opt::from_args();

    let mercutio_state = MoteState {
        body: Arc::new(Mutex::new(default_doc())),
    };

    // port + 1
    sync_socket_server(opt.port + 1, opt.period, mercutio_state.clone());

    spawn_http_server(opt.port);

    // // Loop forever
    // loop {
    //     ::std::thread::sleep(::std::time::Duration::from_millis(1000));
    // }
}
