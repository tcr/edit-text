//! mercutio-server standalone binary for web deployment.

#![feature(proc_macro)]

extern crate bus;
extern crate crossbeam_channel;
extern crate failure;
extern crate include_dir_macro;
extern crate maplit;
extern crate mercutio;
extern crate mercutio_server;
extern crate oatie;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate structopt;
extern crate structopt_derive;
extern crate take_mut;
extern crate tiny_http;
extern crate url;
extern crate ws;

use include_dir_macro::include_dir;
use mercutio_server::sync::*;
use rand::thread_rng;
use std::collections::HashMap;
use std::panic;
use std::path::Path;
use std::process;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use structopt::StructOpt;
use tiny_http::{Header, Response};
use url::Url;

fn spawn_server_thread(
    server: Arc<tiny_http::Server>,
    client_proxy: bool,
    dist_dir: HashMap<&'static Path, &'static [u8]>,
    template_dir: HashMap<&'static Path, &'static [u8]>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let req = server.recv().unwrap();

            // Extract just the path segment from this URL.
            // The `url` crate needs an absolute base to create a Url.
            let path = Url::parse("http://localhost/")
                .unwrap()
                .join(req.url())
                .unwrap()
                .path()
                .to_owned();

            let update_config_var = |data: &[u8]| -> Vec<u8> {
                let input = String::from_utf8_lossy(data);
                let output = input.replace(
                    "CONFIG = {}",
                    &format!(
                        "CONFIG = {{configured: true, wasm: {}}}",
                        if client_proxy { "false" } else { "true" }
                    ),
                );
                output.into_bytes()
            };

            match path.as_ref() {
                "/" | "/index.html" => {
                    // Redirect as random client
                    let mut rng = thread_rng();
                    let new_page_id = ::rand::seq::sample_iter(&mut rng, 0..26u8, 8)
                        .unwrap()
                        .into_iter()
                        .map(|x| (b'a' + x) as char)
                        .collect::<String>();
                    let mut res = Response::empty(302);
                    let mut h = Header::from_bytes(
                        b"Location".to_vec(),
                        format!("/{}#helloworld", new_page_id).as_bytes(),
                    ).unwrap();
                    res.add_header(h);
                    let _ = req.respond(res);
                }

                "/favicon.png" => {
                    let data = template_dir.get(Path::new("favicon.png")).unwrap();
                    let _ = req.respond(
                        Response::from_data(*data).with_header(
                            Header::from_bytes("content-type".as_bytes(), "image/png".as_bytes())
                                .unwrap(),
                        ),
                    );
                }

                "/$/multi" | "/$/multi/" => {
                    let data = template_dir.get(Path::new("multi.html")).unwrap();
                    let _ = req.respond(
                        Response::from_data(update_config_var(data)).with_header(
                            Header::from_bytes("content-type".as_bytes(), "text/html".as_bytes())
                                .unwrap(),
                        ),
                    );
                }

                path => {
                    // Skip the initial "/$/"
                    if path.starts_with("/$/") {
                        let path = path.chars().skip(3).collect::<String>();
                        if let Some(target) = dist_dir.get(Path::new(&path)) {
                            let _ = req.respond(
                                Response::from_data(*target).with_header(
                                    Header::from_bytes(
                                        "content-type".as_bytes(),
                                        "text/html".as_bytes(),
                                    ).unwrap(),
                                ),
                            );
                        } else {
                            let _ = req.respond(Response::from_string("404".to_owned()));
                        }
                    } else {
                        // Possibly a page?
                        let path_segs = (path[1..]).split('/').collect::<Vec<_>>();
                        if valid_page_id(&path_segs[0]) && path_segs.len() == 2
                            && path_segs[1] == "presentation"
                        {
                            let data = template_dir.get(Path::new("presentation.html")).unwrap();
                            let _ = req.respond(
                                Response::from_data(update_config_var(data)).with_header(
                                    Header::from_bytes(
                                        "content-type".as_bytes(),
                                        "text/html".as_bytes(),
                                    ).unwrap(),
                                ),
                            );
                        } else if valid_page_id(&path_segs[0]) && path_segs.len() == 1 {
                            let data = template_dir.get(Path::new("client.html")).unwrap();
                            let _ = req.respond(
                                Response::from_data(update_config_var(data)).with_header(
                                    Header::from_bytes(
                                        "content-type".as_bytes(),
                                        "text/html".as_bytes(),
                                    ).unwrap(),
                                ),
                            );
                        } else {
                            // TODO real 404 error code
                            let _ = req.respond(Response::from_string("404".to_owned()));
                        }
                    }
                }
            }
        }
    })
}

fn run_http_server(port: u16, client_proxy: bool) {
    let server = tiny_http::Server::http(&format!("0.0.0.0:{}", port)).unwrap();

    let server = Arc::new(server);
    let mut guards = Vec::with_capacity(4);

    let dist_dir = include_dir!("mercutio-frontend/dist");
    let template_dir = include_dir!("mercutio-frontend/templates");
    // Necessary files
    assert!(template_dir.contains_key(Path::new("multi.html")));
    assert!(template_dir.contains_key(Path::new("client.html")));
    assert!(template_dir.contains_key(Path::new("presentation.html")));
    assert!(template_dir.contains_key(Path::new("favicon.png")));

    for _ in 0..4 {
        let server = server.clone();

        let guard = spawn_server_thread(
            server.clone(),
            client_proxy,
            dist_dir.clone(),
            template_dir.clone(),
        );

        guards.push(guard);
    }

    println!("Listening on http://localhost:{}/", port);

    for guard in guards {
        let _ = guard.join();
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "mercutio", about = "Sync server.")]
struct Opt {
    #[structopt(long = "port", help = "Port", default_value = "8000")]
    port: u16,

    #[structopt(long = "period", help = "Sync period", default_value = "100")]
    period: usize,

    #[structopt(help = "Enable client proxy", long = "client-proxy", short = "c")]
    client_proxy: bool,
}

fn spawn_sync_socket_server() -> JoinHandle<()> {
    // port + 1
    thread::spawn(|| {
        let opt = Opt::from_args();
        sync_socket_server(opt.port + 1, opt.period);
    })
}

fn main() {
    // Set aborting process handler.
    let orig_handler = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        orig_handler(panic_info);
        process::exit(1);
    }));

    let opt = Opt::from_args();

    println!("client proxy: {:?}", opt.client_proxy);

    let _ = spawn_sync_socket_server();

    run_http_server(opt.port, opt.client_proxy)
}
