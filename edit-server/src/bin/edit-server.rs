//! edit-server standalone binary for web deployment.

#![feature(extern_in_paths)]
#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate oatie;
#[macro_use]
extern crate rouille;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_json;

use md5;
use reqwest;
use crypto::md5::Md5;
use crypto::digest::Digest;
use yansi::Paint;
use edit_common::{
    doc_as_html,
    markdown::{
        doc_to_markdown,
        markdown_to_doc,
    },
};
use edit_server::{
    graphql::client::*,
    sync::*,
};
use handlebars::Handlebars;
use include_dir_macro::include_dir;
use mime_guess::guess_mime_type;
use oatie::doc::*;
use oatie::rtf::*;
use oatie::validate::validate_doc;
use rand::thread_rng;
use rouille::Response;
use std::fs::File;
use std::io::prelude::*;
use std::panic;
use std::path::{
    Path,
    PathBuf,
};
use std::thread;
use std::thread::JoinHandle;
use std::{
    cell::RefCell,
    collections::HashMap,
    env,
};
use structopt::StructOpt;

trait Dir: Sync + Send {
    fn get(&self, path: &Path) -> Option<Vec<u8>>;

    fn exists(&self, path: &Path) -> bool {
        self.get(path).is_some()
    }

    fn clone(&self) -> Box<dyn Dir>;

    fn md5(&self, path: &Path) -> Option<String>;
}

#[derive(Clone)]
struct InlineDir(HashMap<&'static Path, &'static [u8]>);

impl Dir for InlineDir {
    fn get(&self, path: &Path) -> Option<Vec<u8>> {
        self.0.get(path).map(|x| x.to_vec())
    }

    fn clone(&self) -> Box<dyn Dir> {
        Box::new(InlineDir(self.0.clone()))
    }

    fn md5(&self, path: &Path) -> Option<String> {
        thread_local! {
            pub static ETAGS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
        }
        let md5 = ETAGS.with(|etags| {
            etags
                .borrow_mut()
                .entry(path.to_string_lossy().to_string())
                .or_insert_with(|| format!("{:x}", md5::compute(&self.get(path).unwrap())))
                .to_string()
        });
        Some(md5)
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

    fn clone(&self) -> Box<dyn Dir> {
        Box::new(LocalDir(self.0.clone()))
    }

    fn md5(&self, path: &Path) -> Option<String> {
        let mut buffer = vec![0; 1024 * 1024 * 10];
        if let Ok(mut f) = File::open(self.0.join(path)) {
            let mut hasher = Md5::new();
            loop {
                if let Ok(v) = f.read(&mut buffer) {
                    if v == 0 {
                        break;
                    }
                    hasher.input(&buffer[0..v]);
                } else {
                    return None;
                }
            }
            Some(hasher.result_str())
        } else {
            eprintln!("warn: cannot open {:?}", path);
            None
        }
    }
}

pub fn default_doc() -> Doc<RtfSchema> {
    const INPUT: &'static str = r#"

# Welcome

This is a sandbox for edit-text, a web-based rich text editor. Feel free to change and delete the contents of this page, or change the URL to edit a new page. 

* This is alpha-quality software. Don't store anything important here.

* The server and clients are written in Rust, and the front-end is cross-compiled WebAssembly and TypeScript.

* Supports collaborative editing. Share this URL with a (sensible) number of people to all edit this page in realtime.

* Use the Load/Save button in the toolbar to export the page as Markdown or upload your own Markdown file

* Please file bugs if you encounter them. edit-text is open source, so you can also contribute code!

Go to <http://github.com/tcr/edit-text> for more information.

Developer: [@trimryan](http://twitter.com/trimryan)

"#;

    // Should be no errors
    let doc = Doc(markdown_to_doc(&INPUT).unwrap());
    validate_doc(&doc).expect("Initial Markdown document was malformed");
    doc
}

fn run_http_server(port: u16, client_proxy: bool) {
    let dist_dir: Box<dyn Dir>;
    let template_dir: Box<dyn Dir>;
    let static_dir: Box<dyn Dir>;
    if cfg!(feature = "standalone") {
        dist_dir = Box::new(InlineDir(include_dir!("edit-frontend/dist")));
        template_dir = Box::new(InlineDir(include_dir!("edit-frontend/templates")));
        static_dir = Box::new(InlineDir(include_dir!("edit-frontend/static")));
    } else {
        dist_dir = Box::new(LocalDir(PathBuf::from("../edit-frontend/dist")));
        template_dir = Box::new(LocalDir(PathBuf::from("../edit-frontend/templates")));
        static_dir = Box::new(LocalDir(PathBuf::from("../edit-frontend/static")));
    }

    // Necessary files
    assert!(template_dir.exists(Path::new("multi.html")));
    assert!(template_dir.exists(Path::new("client.hbs")));
    assert!(template_dir.exists(Path::new("presentation.hbs")));
    assert!(template_dir.exists(Path::new("favicon.png")));

    // Build dist folder if it doesn't exist.
    if dist_dir.get(Path::new("edit.js")).is_none() {
        eprintln!("{}", Paint::yellow(r"
=======================================================
  WARNING!
  The file edit-frontend/dist/edit.js does not exist.  
  You should generate the frontend code by running   
  `./tools frontend-build` from a terminal window.        
=======================================================
"));
    }

    println!(
        "> Server listening for HTTP connections on http://0.0.0.0:{}/",
        port
    );
    println!("  Talking to client proxy: {:?}", client_proxy);

    let reg = Handlebars::new();

    #[allow(unused)]
    #[allow(unreachable_code)]
    rouille::start_server(format!("0.0.0.0:{}", port), move |request| {
        let edit_title = env::var("EDIT_TITLE").unwrap_or("edit-text".to_string());

        // TODO want to make this better and 1:1 mapped with the config template bundle below
        let update_config_var = |data: &[u8]| -> Vec<u8> {
            let input = String::from_utf8_lossy(data);
            let output = input.replace(
                "CONFIG = {}",
                &format!(
                    "CONFIG = {}",
                    serde_json::to_string(&json!({
                    "configured": true,
                    "wasm": !client_proxy,
                    "title": &edit_title,
                    "console_command_log": cfg!(feature = "console_command_log"),
                }))
                    .unwrap()
                ),
            );
            output.into_bytes()
        };

        fn random_id_alpha() -> String {
            let mut rng = thread_rng();
            return ::rand::seq::sample_iter(&mut rng, 0..26u8, 8)
                .unwrap()
                .into_iter()
                .map(|x| (b'a' + x) as char)
                .collect::<String>();
        }

        router!(request,

            // Redirect root page to a welcome page or a downloaded URL
            (GET) ["/"] => {
                // Redirect to /welcome-{remote ip}
                let mut id = format!(
                    "welcome-{}",
                    format!("{}", request.header("X-Forwarded-For").unwrap_or("127.0.0.1")).replace(":", "-").replace(".", "-"),
                );

                // Upload files using /?from={url}
                let load_doc = request.get_param("from")
                    .ok_or(format_err!("no from parameter to download from")) // TODO what is this line for
                    .and_then(|from| {
                        // Create a randomly-named page ID for this page.
                        id = random_id_alpha();

                        // Download directly from URLs.
                        // TODO is this irresponsible
                        if from.starts_with("http") {
                            let mut client = reqwest::Client::new();
                            let mut res = client.get(&from).send()?;
                            if !res.status().is_success() {
                                bail!("Unsuccessful request")
                            }
                            let md = res.text()?;
                            let doc = Doc(markdown_to_doc(&md)?);
                            Ok(match validate_doc(&doc) {
                                Ok(_) => doc,
                                Err(err) => {
                                    eprintln!("Error decoding document: {:?}", err);
                                    Doc(doc_span![
                                        DocGroup(Attrs::Code, [
                                            DocText("Error decoding document."),
                                        ]),
                                    ])
                                }
                            })
                        } else {
                            // Interpret as a Markdown document
                            let doc = Doc(markdown_to_doc(&from)?);
                            Ok(match validate_doc(&doc) {
                                Ok(_) => doc,
                                Err(err) => {
                                    eprintln!("Error decoding document: {:?}", err);
                                    Doc(doc_span![
                                        DocGroup(Attrs::Code, [
                                            DocText("Error decoding document."),
                                        ]),
                                    ])
                                }
                            })
                        }
                    })
                    .unwrap_or(default_doc());

                // Initialize the "hello world" post.
                eprintln!("creating helloworld post for {:?}", id);
                get_or_create_page_graphql(&id, &load_doc);

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

            // React element preview.
            (GET) ["/$/react"] => {
                // Inline the stylesheet.
                let stylesheet = dist_dir.get(Path::new("edit.css")).unwrap();
                let stylesheet = String::from_utf8_lossy(&stylesheet).to_string();

                let mut template = String::from_utf8_lossy(&update_config_var(
                    &template_dir.get(Path::new("react.hbs")).unwrap(),
                )).to_owned().to_string();

                let payload = reg.render_template(&template, &json!({
                    "stylesheet": &stylesheet,
                })).unwrap();

                return Response::from_data(
                    "text/html",
                    payload.into_bytes(),
                );
            },
            (GET) ["/$/react/"] => {
                return Response::redirect_302("/$/react");
            },

            // TODO: undisable once IP safety is addressed

            // (GET) ["/$/list"] => {
            //     let lis = &get_all_pages_graphql()
            //         .unwrap()
            //         .iter()
            //         .map(|x| {
            //             format!(r#"<li><a href="/{id}">{id}</li>"#, id = x)
            //         })
            //         .collect::<Vec<_>>();

            //     return Response::from_data(
            //         "text/html".to_string(),
            //         format!("<h1>pages</h1><ul>{}</ul>", lis.join("")),
            //     )
            // },
            // (GET) ["/$/list/"] => {
            //     return Response::redirect_302("/$/list");
            // },

            // static_dir
            (GET) ["/$/static/{target}", target: String] => {
                if let Some(data) = static_dir.get(Path::new(&target)) {
                    let mut res = Response::from_data(
                        guess_mime_type(&target).to_string(),
                        data.clone(),
                    );
                    if let Some(md5) = static_dir.md5(Path::new(&target)) {
                        res = res.with_etag(request, md5);
                    }
                    res
                } else {
                    return Response::empty_404();
                }
            },

            // dist_dir
            (GET) ["/$/{target}", target: String] => {
                if let Some(data) = dist_dir.get(Path::new(&target)) {
                    let mut res = Response::from_data(
                        guess_mime_type(&target).to_string(),
                        data.clone(),
                    );
                    if let Some(md5) = dist_dir.md5(Path::new(&target)) {
                        res = res.with_etag(request, md5);
                    }
                    res
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
                        &Doc(doc_span![DocGroup(Attrs::Header(1), [DocText(&id)])]),
                    ).expect("Received malformed content from db, aborting").0
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
                        &Doc(doc_span![DocGroup(Attrs::Header(1), [
                            DocText(&id),
                        ])]),
                    ).unwrap().0
                );

                let payload = reg.render_template(&template, &json!({
                    "body": &body,
                    "id": id,
                    "stylesheet": &stylesheet,
                    "title": &edit_title,
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
        sync_socket_server(opt.port + 1);
    })
}

#[derive(StructOpt, Debug)]
#[structopt(name = "edit", about = "Sync server.")]
struct Opt {
    #[structopt(long = "port", help = "Port", default_value = "8000")]
    port: u16,

    #[structopt(help = "Enable client proxy", long = "client-proxy", short = "c")]
    client_proxy: bool,

    #[structopt(help = "Open in browser.", long = "open")]
    open: bool,
}

fn main() {
    let opt = Opt::from_args();

    // let ron_out = ::ron::ser::to_string(&Doc(::edit_common::markdown::de::markdown_to_doc("# hi").unwrap())).unwrap();
    // println!("---> ron: {}", ron_out);
    // let ron_in: Doc = ::ron::de::from_str(&ron_out).unwrap();
    // println!("---> ron: {:?}", ron_in);
    // ::std::process::exit(1);

    let _ = spawn_sync_socket_server();

    if opt.open {
        let port = opt.port;
        std::thread::spawn(move || {
            #[allow(deprecated)]
            std::thread::sleep_ms(1_000);
            // TODO use actual url generation bindings to --open browser
            let _ = open::that(&format!("http://localhost:{}", port));
        });
    }

    run_http_server(opt.port, opt.client_proxy)
}
