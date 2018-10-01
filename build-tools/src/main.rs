#!/usr/bin/env run-cargo-script
//! ```cargo
//! [dependencies]
//! commandspec = "0.10"
//! failure = "0.1"
//! structopt = "0.2"
//! clap = "2.31"
//! ```

// Don't add additional noise to cargo-script.
#![deny(warnings)]

extern crate commandspec;
extern crate structopt;
extern crate failure;
extern crate clap;

use commandspec::*;
use std::path::Path;
use std::env;
use failure::Error;
use structopt::StructOpt;
use clap::Shell;
use structopt::clap::AppSettings;

#[cfg(windows)]
const WEBPACK_PATH: &str = ".\\node_modules\\.bin\\webpack.cmd";

#[cfg(not(windows))]
const WEBPACK_PATH: &str = "./node_modules/.bin/webpack";

fn abs_string_path<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    Ok(Path::new(".")
        .canonicalize()?
        .join(path)
        .to_string_lossy()
        .into_owned())
}

// Thin wrapper around run()
fn main() {
    commandspec::cleanup_on_ctrlc();

    match run() {
        Ok(_) => {},
        Err(ref err) => {
            eprintln!("Error: {}", err);
            ::std::process::exit(1);
        }
    }
}

/// edit-text build scripts
#[derive(StructOpt)]
#[structopt(name = "./tools", about = "Build tools and commands for developing edit-text.", author = "")]
enum Cli {
    #[structopt(name = "wasm-build", about = "Compile the WebAssembly bundle.")]
    Wasm {
        #[structopt(name = "no-vendor")]
        no_vendor: bool,
    },

    #[structopt(name = "wasm-watch", about = "Watch the WebAssembly bundle.")]
    WasmWatch {
        #[structopt(name = "no-vendor")]
        no_vendor: bool,
    },

    #[structopt(name = "client-proxy", about = "Run client code in your terminal.")]
    ClientProxy { args: Vec<String> },

    #[structopt(name = "client-proxy-build", about = "Build the client proxy.")]
    ClientProxyBuild { args: Vec<String> },

    #[structopt(name = "oatie-build", about = "Build the operational transform library.", raw(setting = "AppSettings::Hidden"))]
    OatieBuild { args: Vec<String> },

    #[structopt(name = "server", about = "Run the edit-text server.")]
    MercutioServerRun {
        #[structopt(long = "log", help = "Export a log")]
        log: bool,
        args: Vec<String>,
    },

    #[structopt(name = "server-build", about = "Build the edit-text server.")]
    MercutioServerBuild { args: Vec<String> },

    #[structopt(name = "replay", about = "Replay an edit-text log.", raw(setting = "AppSettings::Hidden"))]
    Replay { args: Vec<String> },

    #[structopt(name = "test")]
    Test { args: Vec<String> },

    #[structopt(name = "build")]
    Build { args: Vec<String> },

    #[structopt(name = "frontend-build", about = "Bundle the frontend JavaScript code.")]
    FrontendBuild { args: Vec<String> },

    #[structopt(name = "frontend-watch", about = "Watch the frontend JavaScript code, building continuously.")]
    FrontendWatch { args: Vec<String> },

    #[structopt(name = "deploy", about = "Deploy to sandbox.edit.io.", raw(setting = "AppSettings::Hidden"))]
    Deploy {
        #[structopt(long = "skip-download")]
        skip_download: bool,
    },

    #[structopt(name = "book-build", about = "Builds the book.")]
    BookBuild,

    #[structopt(name = "book-watch", about = "Watches and rebuilds the book.")]
    BookWatch,

    #[structopt(name = "completions", about = "Generates completion scripts for your shell.", raw(setting = "AppSettings::Hidden"))]
    Completions {
        #[structopt(name = "SHELL")]
        shell: Shell,
    },

    #[structopt(name = "logs", about = "Dump database logs.")]
    Logs { args: Vec<String> },
}


fn run() -> Result<(), Error> {
    #[allow(non_snake_case)]
    let SELF_PATH = vec!["cargo", "run", "--bin", "build-tools", "--"];

    // Pass arguments directly to subcommands: don't capture -h, -v, or verification
    // Do this by adding "--" into the args flag after the subcommand.
    let mut args = ::std::env::args().collect::<Vec<_>>();
    // TODO this is broken for {self_path} server, or {self_path} deploy, and
    // both require different behavior! why?
    if args.len() > 2 && args[1] != "help" {
        args.insert(2, "--".into());
    }

    // We interpret the --release flag at the build level.
    let release = args.iter().find(|x| *x == "--release").is_some();
    args = args.into_iter().filter(|x| *x != "--release").collect();

    // Respect the CLICOLOR env variable.
    let force_color = ::std::env::var("CLICOLOR").map(|x| x == "1").unwrap_or(false);
    let force_color_flag = if force_color { Some("--color=always") } else { None };

    // Run the subcommand.
    let parsed_args = Cli::from_iter(args.iter());
    match parsed_args {
        Cli::WasmWatch { no_vendor } => {
            execute!(
                "
                    rustup target add wasm32-unknown-unknown
                "
            )?;
            execute!(
                r"
                    cargo watch -i edit-frontend/** -i tools -x 'run --bin build-tools -- wasm-build {no_vendor}'
                ",
                no_vendor = if no_vendor { Some("--no-vendor") } else { None },
            )?;
        },

        Cli::Wasm { no_vendor } => {
            // wasm must always be --release
            let release_flag = Some("--release");

            execute!(
                "
                    rustup target add wasm32-unknown-unknown
                "
            )?;

            eprintln!("Building...");
            execute!(
                r"
                    cd edit-client
                    cargo build {release_flag} --lib --target wasm32-unknown-unknown
                ",
                release_flag = release_flag,
            )?;

            if !no_vendor {
                eprintln!("Vendoring...");

                ::std::fs::create_dir_all("./edit-frontend/src/bindgen")?;

                execute!(
                    r"
                        wasm-bindgen ./target/wasm32-unknown-unknown/release/edit_client.wasm \
                            --out-dir ./edit-frontend/src/bindgen \
                            --typescript
                    ",
                )?;

                // execute!(
                //     r"
                //         cd ./edit-frontend/src/bindgen
                //         wasm2es6js \
                //             --base64 -o edit_client_bg.js edit_client_bg.wasm
                //     ",
                // )?;

                // ::std::fs::remove_file("./edit-frontend/src/bindgen/edit_client_bg.wasm")?;

                eprintln!("Done.");
            }
        }

        Cli::ClientProxy { args } => {
            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd edit-client
                    export MERCUTIO_WASM_LOG=0
                    export RUST_BACKTRACE=1
                    cargo run {release_flag} --bin edit-client-proxy -- {args}
                ",
                release_flag = release_flag,
                args = args,
            )?;
        }

        Cli::ClientProxyBuild { args } => {
            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd edit-client
                    export MERCUTIO_WASM_LOG=0
                    export RUST_BACKTRACE=1
                    cargo build {release_flag} --bin edit-client-proxy -- {args}
                ",
                release_flag = release_flag,
                args = args,
            )?;
        }

        Cli::OatieBuild { args } => {
            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd oatie
                    cargo build {release_flag} {args}
                ",
                release_flag = release_flag,
                args = args,
            )?;
        }

        Cli::MercutioServerRun { log, args } => {
            if release {
                eprintln!("Building and running edit-text server (release mode)...");
            } else {
                eprintln!("Building and running edit-text server (debug mode)...");
            }

            if !Path::new("edit-server/edit.sqlite3").exists() {
                eprintln!("Building database on first startup...");
                execute!(
                    r"
                        cd edit-server
                        diesel setup
                    ",
                )?;
            } else {
                println!("Database path: edit-server/edit.sqlite3");
            }

            // Build dist folder if it doesn't exist.
            if !Path::new("edit-frontend/dist/edit.js").exists() {
                execute!(
                    r"
                        {self_path} wasm-build
                    ",
                    self_path = SELF_PATH,
                )?;
                execute!(
                    r"
                        {self_path} frontend-build
                    ",
                    self_path = SELF_PATH,
                )?;
            }

            eprintln!("Starting server...");

            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd edit-server
                    export MERCUTIO_WASM_LOG={use_log}
                    export RUST_BACKTRACE=1
                    export DATABASE_URL=edit-server/edit.sqlite3
                    cargo run {force_color_flag} {release_flag} \
                        --bin edit-server -- {args}
                ",
                use_log = if log { 1 } else { 0 },
                release_flag = release_flag,
                force_color_flag = force_color_flag,
                args = args,
            )?;

            eprintln!("Server exited.");
        }

        Cli::MercutioServerBuild { args } => {
            let release_flag = if release { Some("--release") } else { None };

            // Build dist folder if it doesn't exist.
            if !Path::new("edit-frontend/dist/edit.js").exists() {
                execute!(
                    r"
                        {self_path} wasm-build
                    ",
                    self_path = SELF_PATH,
                )?;
                execute!(
                    r"
                        {self_path} frontend-build
                    ",
                    self_path = SELF_PATH,
                )?;
            }

            execute!(
                r"
                    cd edit-server
                    cargo build {force_color_flag} {release_flag} \
                        --bin edit-server {args}
                ",
                release_flag = release_flag,
                force_color_flag = force_color_flag,
                args = args,
            )?;
        }

        Cli::Replay { args } => {
            // let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd edit-client
                    export RUST_BACKTRACE=1
                    cargo run --release --bin edit-replay -- {args}
                ",
                // release_flag = release_flag,
                args = args,
            )?;
        }

        Cli::Test { args } => {
            eprintln!("building server...");
            execute!(
                r"
                    {self_path} server-build
                ",
                self_path = SELF_PATH,
            )?;

            eprintln!("running server...");
            let _server_guard = command!(
                r"
                    {self_path} server {args}
                ",
                self_path = SELF_PATH,
                args = args,
            )?.scoped_spawn().unwrap();

            ::std::thread::sleep(::std::time::Duration::from_millis(3000));

            eprintln!("running tests...");
            execute!(
                r"
                    cd tests
                    cargo run --features integration
                ",
            )?;
        }

        Cli::Build { args } => {
            eprintln!("[wasm-build]");
            execute!(
                r"
                    {self_path} wasm-build {args}
                ",
                self_path = SELF_PATH,
                args = args,
            )?;

            eprintln!("");
            eprintln!("[frontend-build]");
            execute!(
                r"
                    {self_path} frontend-build {args}
                ",
                self_path = SELF_PATH,
                args = args,
            )?;

            eprintln!("");
            eprintln!("[server-build]");
            execute!(
                r"
                    {self_path} server-build {args}
                ",
                self_path = SELF_PATH,
                args = args,
            )?;

            eprintln!("");
            eprintln!("[client-proxy-build]");
            execute!(
                r"
                    {self_path} client-proxy-build {args}
                ",
                self_path = SELF_PATH,
                args = args,
            )?;

            eprintln!("");
            eprintln!("[book-build]");
            execute!(
                r"
                    {self_path} book-build {args}
                ",
                self_path = SELF_PATH,
                args = args,
            )?;
        }

        Cli::FrontendBuild { args } => {
            // Install latest npm dependencies
            eprintln!("1");
            execute!(
                r"
                    cd edit-frontend
                    npm install --no-audit
                ",
            )?;
            eprintln!("2");

            execute!(
                r"
                    cd edit-frontend
                    {webpack_path} \
                        ./src/index.js --mode development --output-filename='edit.js' {args}
                ",
                webpack_path = WEBPACK_PATH,
                args = args,
            )?;
            eprintln!("3");
        }

        Cli::FrontendWatch { args } => {
            // Install latest npm dependencies
            execute!(
                r"
                    cd edit-frontend
                    npm install --no-audit
                ",
            )?;

            let _cargo_watch_guard = command!(
                r"
                    cargo watch -i edit-frontend/** -i tools -x 'run --bin build-tools -- wasm-build'
                ",
            )?.scoped_spawn()?;

            execute!(
                r"
                    cd edit-frontend
                    {webpack_path} --watch \
                        ./src/index.js --mode development --output-filename='edit.js' {args}
                ",
                webpack_path = WEBPACK_PATH,
                args = args,
            )?;
        }

        Cli::Deploy { skip_download } => {
            let edit_deploy_url = env::var("EDIT_DEPLOY_URL").unwrap_or("sandbox.edit.io".to_string());
            let edit_dokku_name = env::var("EDIT_DOKKU_NAME").unwrap_or("edit-text".to_string());

            // WASM client code
            eprintln!();
            eprintln!("Compiling WebAssembly...");
            execute!(
                "
                    rustup target add wasm32-unknown-unknown
                "
            )?;
            execute!(
                "
                    {self_path} wasm-build
                ",
                self_path = SELF_PATH,
            )?;

            // Frontend JavaScript
            eprintln!("Building frontend...");
            execute!(
                r"
                    {self_path} frontend-build
                ",
                self_path = SELF_PATH,
            )?;

            // Linux binary
            eprintln!();
            eprintln!("Building Linux server binary...");
            execute!(
                "
                    rustup target add x86_64-unknown-linux-gnu
                "
            )?;
            // TODO replace --skip-download with a smarter heuristic
            if !skip_download {
                // TODO replace this with discrete execute! commands.
                sh_execute!(
                    r#"
                        cd {dir_self}

                        set -e
                        set -x

                        LINKROOT="$(pwd)/dist/link"

                        rm -rf $LINKROOT
                        mkdir -p $LINKROOT

                        cd $LINKROOT

                        export URL=http://security.debian.org/debian-security/pool/updates/main/o/openssl/libssl-dev_1.1.0f-3+deb9u2_amd64.deb
                        curl -O $URL
                        ar p $(basename $URL) data.tar.xz | tar xvf -

                        export URL=http://security.debian.org/debian-security/pool/updates/main/o/openssl/libssl1.1_1.1.0f-3+deb9u2_amd64.deb
                        curl -O $URL
                        ar p $(basename $URL) data.tar.xz | tar xvf -

                        export URL=http://ftp.us.debian.org/debian/pool/main/g/glibc/libc6_2.24-11+deb9u3_amd64.deb
                        curl -O $URL
                        ar p $(basename $URL) data.tar.xz | tar xvf -
                    "#,
                    dir_self = abs_string_path(".")?,
                );
            }
            execute!(
                r#"
                    cd edit-server

                    export LD_LIBRARY_PATH="{dir_link}/usr/lib/x86_64-linux-gnu;{dir_link}/lib/x86_64-linux-gnu"
                    export OPENSSL_LIB_DIR="{dir_link}/usr/lib/x86_64-linux-gnu/"
                    export OPENSSL_DIR="{dir_link}/usr/"
                    export TARGET_CC="x86_64-unknown-linux-gnu-gcc"
                    export TARGET_CFLAGS="-I {dir_link}/usr/include/x86_64-linux-gnu -isystem {dir_link}/usr/include"

                    cargo build --release --target=x86_64-unknown-linux-gnu \
                        --bin edit-server --features 'standalone'

                "#,
                // Must expand absolute path for linking
                dir_link = format!("{}/dist/link", abs_string_path(".")?),
            )?;
            eprintln!();
            eprintln!("Copying directories...");
            execute!(
                "
                    cp target/x86_64-unknown-linux-gnu/release/edit-server dist/deploy
                "
            )?;

            // Shell out for uploading the file to dokku.
            eprintln!();
            eprintln!("Uploading...");
            sh_execute!(
                r#"
                    cd dist/deploy

                    # Doing these two commands as one pipe may cause dokku to hang
                    # (from experience) so first, upload the tarball, then load it.
                    tar c . | bzip2 | ssh root@{dokku_url} "bunzip2 > /tmp/edit.tar"
                    ssh root@{dokku_url} 'cat /tmp/edit.tar | dokku tar:in {dokku_name}'
                "#,
                dokku_url = edit_deploy_url,
                dokku_name = edit_dokku_name,
            )?;
        }

        Cli::BookBuild => {
            execute!(
                r"
                    cd docs
                    mdbook build
                ",
            )?;
        }

        Cli::BookWatch => {
            execute!(
                r"
                    cd docs
                    mdbook serve
                ",
            )?;
        }

        Cli::Completions { shell } => {
            let mut app = Cli::clap();
            app.gen_completions_to(
                "tools", 
                shell,
                &mut ::std::io::stdout()
            );
        }

        Cli::Logs { args } => {
            execute!(
                r"
                    cd edit-server
                    export RUST_BACKTRACE=1
                    export DATABASE_URL={database_url}
                    cargo run --bin edit-server-logs -- {args}
                ",
                database_url = env::var("DATABASE_URL").unwrap_or("edit-server/edit.sqlite3".to_string()),
                // release_flag = release_flag,
                args = args,
            )?;
        }
    }

    Ok(())
}
