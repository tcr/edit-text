#!/usr/bin/env run-cargo-script
//! ```cargo
//! [dependencies]
//! commandspec = "0.6.1"
//! failure = "0.1"
//! quicli = "0.2"
//! ```

// Don't add additional noise to cargo-script.
#![deny(warnings)]

#[macro_use]
extern crate commandspec;
#[macro_use]
extern crate quicli;
extern crate failure;

use commandspec::*;
use quicli::prelude::*;
use std::path::Path;
use failure::Error;

// https://github.com/killercup/quicli/issues/66
use std::result::Result;

/// edit-text build scripts
#[derive(StructOpt)]
#[structopt(name = "edit-text build scripts", about = "Build scripts for mercutio and oatie", author = "")]
enum Cli {
    #[structopt(name = "wasm-build", about = "Compile the WebAssembly bundle.")]
    Wasm,

    #[structopt(name = "client-proxy", about = "Run client code in your terminal.")]
    WasmProxy { args: Vec<String> },

    #[structopt(name = "oatie-build", about = "Build the operational transform library.")]
    OatieBuild { args: Vec<String> },

    #[structopt(name = "server", about = "Run the edit-text server.")]
    MercutioSyncRun {
        #[structopt(long = "log", help = "Export a log")]
        log: bool,
        args: Vec<String>,
    },

    #[structopt(name = "server-build", about = "Build the edit-text server.")]
    MercutioSyncBuild { args: Vec<String> },

    #[structopt(name = "server-callgrind")]
    MercutioSyncCallgrind { args: Vec<String> },

    #[structopt(name = "replay", about = "Replay an edit-text log.")]
    Replay { args: Vec<String> },

    #[structopt(name = "test")]
    Test { args: Vec<String> },

    #[structopt(name = "frontend-build", about = "Bundle the frontend JavaScript code.")]
    JsBuild { args: Vec<String> },

    #[structopt(name = "frontend-watch", about = "Watch the frontend JavaScript code, building continuously.")]
    JsWatch { args: Vec<String> },

    #[structopt(name = "deploy", about = "Deploy to sandbox.edit.io.")]
    Deploy,
}

fn abs_string_path<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    Ok(Path::new(".")
        .canonicalize()?
        .join(path)
        .to_string_lossy()
        .into_owned())
}

main!(|| {
    // Pass arguments directly to subcommands: don't capture -h, -v, or verification
    // Do this by adding "--" into the args flag after the subcommand.
    let mut args = ::std::env::args().collect::<Vec<_>>();
    if args.len() > 2 && args[1] != "help" {
        args.insert(2, "--".into());
    }

    // We interpret the --release flag at the build level.
    let release = args.iter().find(|x| *x == "--release").is_some();
    args = args.into_iter().filter(|x| *x != "--release").collect();

    // Run the subcommand.
    let args = Cli::from_iter(args.into_iter());
    match args {
        Cli::Wasm => {
            // wasm must always be --release
            let release_flag = Some("--release");

            execute!(
                r"
                    cd mercutio-wasm
                    cargo check {release_flag} --lib --target wasm32-unknown-unknown
                ",
                release_flag = release_flag,
            )?;

            execute!(
                r"
                    cd mercutio-wasm
                    cargo build {release_flag} --lib --target wasm32-unknown-unknown
                ",
                release_flag = release_flag,
            )?;

            execute!(
                r"
                    cp target/wasm32-unknown-unknown/release/mercutio.wasm \
                        mercutio-frontend/dist
                ",
            )?;
        }

        Cli::WasmProxy { args } => {
            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd mercutio-client
                    export MERCUTIO_WASM_LOG=0
                    export RUST_BACKTRACE=1
                    cargo run {release_flag} --bin mercutio-wasm-proxy -- {args}
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

        Cli::MercutioSyncRun { log, args } => {
            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd mercutio-server
                    export MERCUTIO_WASM_LOG={use_log}
                    export RUST_BACKTRACE=1
                    cargo run {release_flag} --bin mercutio-server -- \
                        --period 100 {args}
                ",
                use_log = if log { 1 } else { 0 },
                release_flag = release_flag,
                args = args,
            )?;
        }

        Cli::MercutioSyncBuild { args } => {
            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd mercutio-server
                    export RUST_BACKTRACE=1
                    cargo build {release_flag} --bin mercutio-server {args}
                ",
                release_flag = release_flag,
                args = args,
            )?;
        }

        Cli::MercutioSyncCallgrind { args } => {
            execute!(
                r"
                    cd mercutio-server
                    cargo build --release --bin mercutio-server
                ",
            )?;

            execute!(
                r"
                    cd mercutio-server
                    export MERCUTIO_SYNC_LOG=1
                    export RUST_BACKTRACE=1
                    cargo profiler callgrind --bin ./target/release/mercutio-server -- \
                        --period 100 {args}
                ",
                args = args,
            )?;
        }

        Cli::Replay { args } => {
            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd mercutio
                    export RUST_BACKTRACE=1
                    cargo run {release_flag} --bin mercutio-replay -- {args}
                ",
                release_flag = release_flag,
                args = args,
            )?;
        }

        Cli::Test { args } => {
            execute!(
                r"
                    cd oatie
                    ./transform-test.sh {args}
                ",
                args = args,
            )?;
        }

        Cli::JsBuild { args } => {
            execute!(
                r"
                    cd mercutio-frontend
                    ./node_modules/.bin/webpack \
                        ./src/app.ts ./dist/mercutio.js {args}
                ",
                args = args,
            )?;
        }

        Cli::JsWatch { args } => {
            execute!(
                r"
                    cd mercutio-frontend
                    ./node_modules/.bin/webpack --watch \
                        ./src/app.ts ./dist/mercutio.js {args}
                ",
                args = args,
            )?;
        }

        Cli::Deploy => {
            // Frontend JavaScript
            eprintln!("Building frontend...");
            execute!(
                r"
                    ./x.rs frontend-build
                "
            )?;

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
                    ./x.rs wasm-build
                "
            )?;

            // Linux binary
            eprintln!();
            eprintln!("Building Linux server binary...");
            execute!(
                "
                    docker build \
                        -f dist/build/Dockerfile dist/build/ \
                        -t mercutio-build-server
                "
            )?;
            execute!(
                r"
                    docker run --rm \
                        -v {dir_git}:/usr/local/cargo/git \
                        -v {dir_registry}:/usr/local/cargo/registry \
                        -v {dir_rustup}:/usr/local/rustup/toolchains \
                        -v {dir_self}:/app \
                        -w /app/mercutio-server \
                        -t -i mercutio-build-server \
                        cargo build --release --target=x86_64-unknown-linux-gnu --bin mercutio-server
                ",
                dir_git = abs_string_path("dist/build/cargo-git-cache")?,
                dir_registry = abs_string_path("dist/build/cargo-registry-cache")?,
                dir_rustup = abs_string_path("dist/build/rustup-toolchain-cache")?,
                dir_self = abs_string_path(".")?,
            )?;
            execute!(
                "
                    cp target/x86_64-unknown-linux-gnu/release/mercutio-server dist/deploy
                "
            )?;

            // Shell out for uploading the file to dokku.
            eprintln!();
            eprintln!("Uploading...");
            shell_sh!(
                r#"
                    cd dist/deploy

                    # Doing these two commands as one pipe may cause dokku to hang
                    # (from experience) so first, upload the tarball, then load it.
                    tar c . | bzip2 | ssh root@sandbox.edit.io "bunzip2 > /tmp/mercutio.tar"
                    ssh root@sandbox.edit.io "cat /tmp/mercutio.tar | dokku tar:in edit-text"
                "#,
            )?;
        }
    }
});
