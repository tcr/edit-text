#!/usr/bin/env ./etc/run-cargo-script-header.sh
//! ```cargo
//! [dependencies]
//! command-macro = "0.1.0"
//! failure = "*"
//! quicli = "*"
//! ```

// #![deny(warnings)]

#[macro_use]
extern crate command_macro;
#[macro_use]
extern crate quicli;

use quicli::prelude::*;
use command_macro::*;

/// edit-text build scripts
#[derive(StructOpt)]
#[structopt(
    name = "edit-text",
    about = "Build scripts for mercutio and oatie",
    author = "",
)]
enum Cli {
    #[structopt(name = "wasm-build")]
    Wasm,

    #[structopt(name = "client-proxy")]
    WasmProxy {
        args: Vec<String>,
    },

    #[structopt(name = "oatie-build")]
    OatieBuild {
        args: Vec<String>,
    },

    #[structopt(name = "server")]
    MercutioSyncRun {
        #[structopt(long = "log", help = "Export a log")]
        log: bool,
        args: Vec<String>,
    },

    #[structopt(name = "server-build")]
    MercutioSyncBuild {
        args: Vec<String>,
    },

    #[structopt(name = "server-callgrind")]
    MercutioSyncCallgrind {
        args: Vec<String>,
    },

    #[structopt(name = "replay")]
    Replay {
        args: Vec<String>,
    },

    #[structopt(name = "test")]
    Test {
        args: Vec<String>,
    },

    #[structopt(name = "frontend-build")]
    JsBuild {
        args: Vec<String>,
    },

    #[structopt(name = "frontend-watch")]
    JsWatch {
        args: Vec<String>,
    },
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

            command!(
                cd: "mercutio/mercutio-wasm",
                "cargo check {release_flag} --lib --target wasm32-unknown-unknown",
                release_flag=release_flag,
            ).execute();

            command!(
                cd: "mercutio/mercutio-wasm",
                "cargo build {release_flag} --lib --target wasm32-unknown-unknown",
                release_flag=release_flag,
            ).execute();

            command!(
                cd: "mercutio/mercutio-wasm",
                "cp ../../target/wasm32-unknown-unknown/release/mercutio.wasm ../frontend/dist",
            ).execute();
        }

        Cli::WasmProxy { args } => {
            let release_flag = if release { Some("--release") } else { None };
            
            command!(
                cd: "mercutio",
                env: CARGO_INCREMENTAL=1,
                env: MERCUTIO_WASM_LOG=0,
                env: RUST_BACKTRACE=1,
                "cargo run {release_flag} --bin mercutio-wasm-proxy -- {args}",
                release_flag=release_flag,
                args=args,
            ).execute();
        }

        Cli::OatieBuild { args } => {
            let release_flag = if release { Some("--release") } else { None };

            command!(
                cd: "oatie",
                env: CARGO_INCREMENTAL=1,
                "cargo build {release_flag} {args}",
                release_flag=release_flag,
                args=args,
            ).execute();
        }

        Cli::MercutioSyncRun { log, args } => {
            let release_flag = if release { Some("--release") } else { None };

            command!(
                cd: "mercutio",
                env: CARGO_INCREMENTAL=1,
                env: MERCUTIO_WASM_LOG=if log { 1 } else { 0 },
                env: RUST_BACKTRACE=1,
                "cargo run {release_flag} --bin mercutio-sync -- --period 100 {args}",
                release_flag=release_flag,
                args=args,
            ).execute();
        }

        Cli::MercutioSyncBuild { args } => {
            let release_flag = if release { Some("--release") } else { None };

            command!(
                cd: "mercutio",
                env: CARGO_INCREMENTAL=1,
                env: RUST_BACKTRACE=1,
                "cargo build {release_flag} --bin mercutio-sync {args}",
                release_flag=release_flag,
                args=args,
            ).execute();
        }

        Cli::MercutioSyncCallgrind { args } => {
            command!(
                cd: "mercutio",
                env: CARGO_INCREMENTAL=1,
                "cargo build --release --bin mercutio-sync",
            ).execute();

            command!(
                cd: "mercutio",
                env: MERCUTIO_SYNC_LOG=1,
                env: RUST_BACKTRACE=1,
                "cargo profiler callgrind --bin ./target/release/mercutio-sync -- --period 100 {args}",
                args=args,
            ).execute();
        }

        Cli::Replay { args } => {
            let release_flag = if release { Some("--release") } else { None };

            command!(
                cd: "mercutio",
                env: CARGO_INCREMENTAL=1,
                env: RUST_BACKTRACE=1,
                "cargo run {release_flag} --bin mercutio-replay -- {args}",
                release_flag=release_flag,
                args=args,
            ).execute();
        }

        Cli::Test { args } => {
            command!(
                cd: "oatie",
                "./transform-test.sh {args}",
                args=args,
            ).execute();
        }

        Cli::JsBuild { args } => {
            command!(
                cd: "mercutio/frontend",
                "npx webpack ./src/app.ts ./dist/mercutio.js {args}",
                args=args,
            ).execute();
        }

        Cli::JsWatch { args } => {
            command!(
                cd: "mercutio/frontend",
                "npx webpack --watch ./src/app.ts ./dist/mercutio.js {args}",
                args=args,
            ).execute();
        }
    }
});
