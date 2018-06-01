//! edit-server standalone binary for web deployment.

#![feature(extern_in_paths)]
#![feature(proc_macro)]
#![feature(proc_macro_non_items)]

extern crate crossbeam_channel;
extern crate edit_common;
extern crate edit_server;
extern crate include_dir_macro;
extern crate maplit;
#[macro_use]
extern crate oatie;
extern crate rand;
#[macro_use]
extern crate rouille;
extern crate serde;
extern crate structopt;
extern crate structopt_derive;
extern crate take_mut;
extern crate url;
#[macro_use]
extern crate failure;
extern crate handlebars;
extern crate md5;
extern crate mime_guess;
extern crate reqwest;
extern crate ron;
extern crate ws;
#[macro_use]
extern crate serde_json;

use edit_common::{
    doc_as_html, markdown::{doc_to_markdown, markdown_to_doc},
};
use extern::edit_server::{graphql::client::*, sync::*};
use handlebars::Handlebars;
use include_dir_macro::include_dir;
use mime_guess::guess_mime_type;
use oatie::doc::*;
use oatie::validate::validate_doc;
use rand::thread_rng;
use rouille::Response;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::panic;
use std::path::{Path, PathBuf};
use std::process;
use std::thread;
use std::thread::JoinHandle;
use structopt::StructOpt;

trait Dir: Sync + Send {
    fn get(&self, &Path) -> Option<Vec<u8>>;

    fn exists(&self, path: &Path) -> bool {
        self.get(path).is_some()
    }

    fn clone(&self) -> Box<Dir>;
}

#[derive(Clone)]
struct InlineDir(HashMap<&'static Path, &'static [u8]>);

impl Dir for InlineDir {
    fn get(&self, path: &Path) -> Option<Vec<u8>> {
        self.0.get(path).map(|x| x.to_vec())
    }

    fn clone(&self) -> Box<Dir> {
        Box::new(InlineDir(self.0.clone()))
    }
}

#[derive(Clone)]
struct LocalDir(PathBuf);

impl Dir for LocalDir {
    fn get(&self, path: &Path) -> Option<Vec<u8>> {
        if let Some(mut f) = File::open(self.0.join(path)).ok() {
            let mut s = vec![];
            if let Err(_) = f.read_to_end(&mut s) {
                return None;
            }
            Some(s)
        } else {
            None
        }
    }

    fn clone(&self) -> Box<Dir> {
        Box::new(LocalDir(self.0.clone()))
    }
}

pub fn default_doc() -> Doc {
    const INPUT: &'static str = r#"

# Hello world!

This is edit-text, a web-based rich text editor.

* This is a very early preview.

* Supports collaborative editing.

* Written in Rust in the backend, cross-compiled to WebAssembly on the frontend.

* Supports Markdown export.

This app might be easy to break! That's okay though. We'll notice and fix it, and it'll break less in the future.

Type github.com/tcr/edit-text into your search bar for more information.

"#;

    // Should be no errors
    let doc = Doc(markdown_to_doc(&INPUT).unwrap());
    validate_doc(&doc).expect("Initial Markdown document was malformed");
    doc
}

fn run_http_server(port: u16, client_proxy: bool) {
    let dist_dir: Box<Dir>;
    let template_dir: Box<Dir>;
    if cfg!(feature = "standalone") {
        dist_dir = Box::new(InlineDir(include_dir!("edit-frontend/dist")));
        template_dir = Box::new(InlineDir(include_dir!("edit-frontend/templates")));
    } else {
        dist_dir = Box::new(LocalDir(PathBuf::from("../edit-frontend/dist")));
        template_dir = Box::new(LocalDir(PathBuf::from("../edit-frontend/templates")));
    }

    // Necessary files
    assert!(template_dir.exists(Path::new("multi.html")));
    assert!(template_dir.exists(Path::new("client.hbs")));
    assert!(template_dir.exists(Path::new("presentation.hbs")));
    assert!(template_dir.exists(Path::new("favicon.png")));

    println!("Listening on http://0.0.0.0:{}/", port);

    let reg = Handlebars::new();

    #[allow(unused)]
    #[allow(unreachable_code)]
    rouille::start_server(format!("0.0.0.0:{}", port), move |request| {
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

        fn random_id() -> String {
            let mut rng = thread_rng();
            return ::rand::seq::sample_iter(&mut rng, 0..26u8, 8)
                .unwrap()
                .into_iter()
                .map(|x| (b'a' + x) as char)
                .collect::<String>();
        }

        router!(request,

            // Redirect root page to random page ID
            (GET) ["/"] => {
                let id = random_id();

                let load_doc = request.get_param("from")
                    .ok_or(format_err!("no from parameter to download from"))
                    .and_then(|from| {
                        let mut client = reqwest::Client::new();
                        let mut res = client.get(&from).send()?;
                        if !res.status().is_success() {
                            bail!("Unsuccessful request")
                        }
                        let md = res.text()?;
                        let doc = Doc(markdown_to_doc(&md)?);
                        Ok(if let Ok(_) = validate_doc(&doc) {
                            doc
                        } else {
                            Doc(doc_span![
                                DocGroup({"tag": "pre"}, [DocChars("Error decoding document.")]),
                            ])
                        })
                    })
                    .unwrap_or(default_doc());

                // Initialize the "hello world" post.
                eprintln!("creating helloworld post for {:?}", id);
                create_page_graphql(&id, &load_doc);

                return Response::redirect_302(format!("/{}", id));
            },
            (GET) ["/index.html"] => {
                return Response::redirect_302("/");
            },

            (GET) ["/favicon.png"] => {
                return Response::from_data(
                    "image/png",
                    template_dir.get(Path::new("favicon.png")).unwrap(),
                );
            },

            (GET) ["/favicon.ico"] => {
                return Response::from_data(
                    "image/x-icon",
                    template_dir.get(Path::new("favicon.ico")).unwrap(),
                );
            },

            (GET) ["/$/multi"] => {
                return Response::from_data(
                    "text/html",
                    update_config_var(
                        &template_dir.get(Path::new("multi.html")).unwrap(),
                    ),
                );
            },
            (GET) ["/$/multi/"] => {
                return Response::redirect_302("/$/multi");
            },

            (GET) ["/$/{target}", target: String] => {
                if let Some(data) = dist_dir.get(Path::new(&target)) {
                    return Response::from_data(
                        guess_mime_type(&target).to_string(),
                        data.clone(),
                    ).with_etag(request,
                        format!("{:x}", md5::compute(data)),
                    );
                } else {
                    return Response::empty_404();
                }
            },

            (GET) ["/{id}/presentation", id: String] => {
                let mut template = String::from_utf8_lossy(&update_config_var(
                    &template_dir.get(Path::new("presentation.hbs")).unwrap(),
                )).to_owned().to_string();

                // Preload content into the file using the db connection.
                let body: String = doc_to_markdown(
                    &get_or_create_page_graphql(
                        &id,
                        &Doc(doc_span![DocGroup({"tag": "h1"}, [DocChars(&id)])]),
                    ).unwrap().0
                ).unwrap();

                let payload = reg.render_template(&template, &json!({
                    "body": &body,
                })).unwrap();

                return Response::from_data(
                    "text/html",
                    payload.into_bytes(),
                );
            },
            (GET) ["/{id}/presentation/", id: String] => {
                return Response::redirect_302(format!("/{}/presentation", id));
            },

            (GET) ["/{id}", id: String] => {
                // Inline the stylesheet.
                let stylesheet = dist_dir.get(Path::new("edit.css")).unwrap();
                let stylesheet = String::from_utf8_lossy(&stylesheet).to_string();

                let mut template = String::from_utf8_lossy(&update_config_var(
                    &template_dir.get(Path::new("client.hbs")).unwrap(),
                )).to_owned().to_string();

                // Preload content into the file using the db connection.
                let body: String = doc_as_html(
                    &get_or_create_page_graphql(
                        &id,
                        &Doc(doc_span![DocGroup({"tag": "h1"}, [DocChars(&id)])]),
                    ).unwrap().0
                );

                let payload = reg.render_template(&template, &json!({
                    "body": &body,
                    "stylesheet": &stylesheet,
                })).unwrap();

                return Response::from_data(
                    "text/html",
                    payload.into_bytes(),
                );
            },
            (GET) ["/{id}/", id: String] => {
                return Response::redirect_302(format!("/{}", id));
            },

            _ => Response::empty_404()
        )
    });
}

fn spawn_sync_socket_server() -> JoinHandle<()> {
    // port + 1
    thread::spawn(|| {
        let opt = Opt::from_args();
        sync_socket_server(opt.port + 1, opt.period);
    })
}

#[derive(StructOpt, Debug)]
#[structopt(name = "edit", about = "Sync server.")]
struct Opt {
    #[structopt(long = "port", help = "Port", default_value = "8000")]
    port: u16,

    #[structopt(long = "period", help = "Sync period", default_value = "100")]
    period: usize,

    #[structopt(help = "Enable client proxy", long = "client-proxy", short = "c")]
    client_proxy: bool,
}

fn main() {
    // Set aborting process handler.
    let orig_handler = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        orig_handler(panic_info);
        process::exit(1);
    }));

    let opt = Opt::from_args();

    // let ron_out = ::ron::ser::to_string(&Doc(::edit_common::markdown::de::markdown_to_doc("# hi").unwrap())).unwrap();
    // println!("---> ron: {}", ron_out);
    // let ron_in: Doc = ::ron::de::from_str(&ron_out).unwrap();
    // println!("---> ron: {:?}", ron_in);
    // ::std::process::exit(1);

    println!("client proxy: {:?}", opt.client_proxy);

    let _ = spawn_sync_socket_server();

    run_http_server(opt.port, opt.client_proxy)
}
