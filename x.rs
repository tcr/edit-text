#!/usr/bin/env run-cargo-script
//! ```cargo
//! [dependencies]
//! commandspec = "0.3.0"
//! failure = "*"
//! quicli = "*"
//! ```

#![deny(warnings)]

#[macro_use]
extern crate commandspec;
#[macro_use]
extern crate quicli;

use commandspec::*;
use quicli::prelude::*;

/// edit-text build scripts
#[derive(StructOpt)]
#[structopt(name = "edit-text build scripts", about = "Build scripts for mercutio and oatie", author = "")]
enum Cli {
    #[structopt(name = "wasm-build")]
    Wasm,

    #[structopt(name = "client-proxy")]
    WasmProxy { args: Vec<String> },

    #[structopt(name = "oatie-build")]
    OatieBuild { args: Vec<String> },

    #[structopt(name = "server")]
    MercutioSyncRun {
        #[structopt(long = "log", help = "Export a log")]
        log: bool,
        args: Vec<String>,
    },

    #[structopt(name = "server-build")]
    MercutioSyncBuild { args: Vec<String> },

    #[structopt(name = "server-callgrind")]
    MercutioSyncCallgrind { args: Vec<String> },

    #[structopt(name = "replay")]
    Replay { args: Vec<String> },

    #[structopt(name = "test")]
    Test { args: Vec<String> },

    #[structopt(name = "frontend-build")]
    JsBuild { args: Vec<String> },

    #[structopt(name = "frontend-watch")]
    JsWatch { args: Vec<String> },

    #[structopt(name = "deploy")]
    Deploy,
}

main!(|| {
    // Pass arguments directly to subcommands (no -h, -v, or verification)
    let mut args = ::std::env::args().collect::<Vec<_>>();
    if args.len() > 2 && args[1] != "help" {
        args.insert(2, "--".into());
    }

    let release = args.iter().find(|x| *x == "--release").is_some();
    args = args.into_iter().filter(|x| *x != "--release").collect();

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
                    cp target/wasm32-unknown-unknown/release/mercutio.wasm mercutio-frontend/dist
                ",
            )?;
        }

        Cli::WasmProxy { args } => {
            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd mercutio
                    export CARGO_INCREMENTAL=1
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
                    export CARGO_INCREMENTAL=1
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
                    export CARGO_INCREMENTAL=1
                    export MERCUTIO_WASM_LOG={use_log}
                    export RUST_BACKTRACE=1
                    cargo run {release_flag} --bin mercutio-server -- --period 100 {args}
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
                    export CARGO_INCREMENTAL=1
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
                    export CARGO_INCREMENTAL=1
                    cargo build --release --bin mercutio-server
                ",
            )?;

            execute!(
                r"
                    cd mercutio-server
                    export MERCUTIO_SYNC_LOG=1
                    export RUST_BACKTRACE=1
                    cargo profiler callgrind --bin ./target/release/mercutio-server -- --period 100 {args}
                ",
                args = args,
            )?;
        }

        Cli::Replay { args } => {
            let release_flag = if release { Some("--release") } else { None };

            execute!(
                r"
                    cd mercutio
                    export CARGO_INCREMENTAL=1
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
                    cd mercutio/frontend
                    npx webpack ./src/app.ts ./dist/mercutio.js {args}
                ",
                args = args,
            )?;
        }

        Cli::JsWatch { args } => {
            execute!(
                r"
                    cd mercutio/frontend
                    npx webpack --watch ./src/app.ts ./dist/mercutio.js {args}
                ",
                args = args,
            )?;
        }

        Cli::Deploy => {
            // Frontend JavaScript
            execute!(
                r"
                    ./x.rs frontend-build
                "
            )?;

            // WASM client code
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
            execute!(
                "
                    cd dist
                    vagrant up
                "
            )?;
            execute!(
                r#"
                    cd dist
                    vagrant ssh -c "
                        cd /vagrant/mercutio
                        rustup override set `cat ../rust-toolchain`
                        cargo build --release --target=x86_64-unknown-linux-gnu --bin mercutio-sync
                    "
                "#,
            )?;
            execute!(
                "
                    cp target/x86_64-unknown-linux-gnu/release/mercutio-sync dist
                "
            )?;

            // Shell out for uploading the file to dokku.
            // Doing this in one command may cause dokku to hang, from experience,
            // so we first upload the tarball then pipe it in.
            shell!(
                r#"
                    set -e
                    cd dist

                    tar c . | bzip2 | ssh root@sandbox.edit.io "bunzip2 > /tmp/mercutio.tar"
                    ssh root@sandbox.edit.io "cat /tmp/mercutio.tar | dokku tar:in edit-text"
                "#,
            )?;
        }
    }
});
