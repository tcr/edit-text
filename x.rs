#!/usr/bin/env run-cargo-script
//! ```cargo
//! [dependencies]
//! command-macros = "*"
//! failure = "*"
//! quicli = "*"
//! clap = "*"
//! ```

#![deny(warnings)]

#[macro_use]
extern crate command_macros;
#[macro_use]
extern crate quicli;
extern crate clap;

use quicli::prelude::*;
// use clap::{App, SubCommand, Arg};

// Add cool slogan for your app here, e.g.:
/// Get first n lines of a file
#[derive(StructOpt)]
#[structopt(name = "edit-text", about = "Build scripts for mercutio and oatie")]
enum Cli {
    #[structopt(name = "wasm")]
    Wasm,

    #[structopt(name = "wasm-proxy")]
    WasmProxy {
        args: Vec<String>,
    },

    #[structopt(name = "oatie-build")]
    OatieBuild {
        args: Vec<String>,
    },

    #[structopt(name = "mercutio-sync")]
    MercutioSyncRun {
        #[structopt(long = "log", help = "Export a log")]
        log: bool,
        args: Vec<String>,
    },

    #[structopt(name = "mercutio-sync-build")]
    MercutioSyncBuild {
        args: Vec<String>,
    },

    #[structopt(name = "mercutio-sync-callgrind")]
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

    #[structopt(name = "js-build")]
    JsBuild {
        args: Vec<String>,
    },

    #[structopt(name = "js-watch")]
    JsWatch {
        args: Vec<String>,
    },
}

trait ExpectSuccess {
    fn expect_success(&self) -> &Self;
}

impl ExpectSuccess for ::std::process::ExitStatus {
    fn expect_success(&self) -> &Self {
        if !self.success() {
            ::std::process::exit(1);
        }
        // assert_eq!(self.success(), true, "Command was not successful.");
        self
    }
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
            let release_flag = vec!["--release"];

            cmd!(
                cargo check [release_flag] ("--lib") ("--target") ("wasm32-unknown-unknown")
            )
                .current_dir("mercutio/mercutio-wasm")
                .env("CARGO_INCREMENTAL", "1")
                .status()?
                .expect_success();

            cmd!(
                cargo build [release_flag] ("--lib") ("--target") ("wasm32-unknown-unknown")
            )
                .current_dir("mercutio/mercutio-wasm")
                .env("CARGO_INCREMENTAL", "1")
                .status()?
                .expect_success();

            cmd!(
                cp ("../../target/wasm32-unknown-unknown/release/mercutio.wasm") ("../frontend/dist")
            )
                .current_dir("mercutio/mercutio-wasm")
                .status()?
                .expect_success();
        }

        Cli::WasmProxy { args } => {
            let release_flag = if release { vec!["--release"] } else { vec![] };
            cmd!(
                cargo run [release_flag] ("--bin") ("mercutio-wasm-proxy") ("--") [args]
            )
                .current_dir("mercutio")
                .env("RUST_BACKTRACE", "1")
                .env("CARGO_INCREMENTAL", "1")
                .env("MERCUTIO_WASM_LOG", "0")
                .status()?
                .expect_success();
        }

        Cli::OatieBuild { args } => {
            let release_flag = if release { vec!["--release"] } else { vec![] };
            cmd!(
                cargo build [release_flag] [args]
            )
                .current_dir("oatie")
                .env("CARGO_INCREMENTAL", "1")
                .status()?
                .expect_success();
        }

        Cli::MercutioSyncRun { log, args } => {
            let release_flag = if release { vec!["--release"] } else { vec![] };
            cmd!(
                cargo run ("--bin") ("mercutio-sync") [release_flag] ("--") ("--period") ("100") [args]
            )
                .current_dir("mercutio")
                .env("RUST_BACKTRACE", "1")
                .env("CARGO_INCREMENTAL", "1")
                .env("MERCUTIO_SYNC_LOG", if log { "1" } else { "0" })
                .status()?
                .expect_success();
        }

        Cli::MercutioSyncBuild { args } => {
            let release_flag = if release { vec!["--release"] } else { vec![] };
            cmd!(
                cargo build [release_flag] ("--bin") ("mercutio-sync") ("--") ("--period") ("100") [args]
            )
                .current_dir("mercutio")
                .env("RUST_BACKTRACE", "1")
                .env("CARGO_INCREMENTAL", "1")
                .status()?
                .expect_success();
        }

        Cli::MercutioSyncCallgrind { args } => {
            cmd!(
                cargo build ("--release") ("--bin") ("mercutio-sync")
            )
                .current_dir("mercutio")
                .env("CARGO_INCREMENTAL", "1")
                .status()?
                .expect_success();

            cmd!(
                cargo profiler callgrind ("--bin") ("./target/release/mercutio-sync") ("--") ("--period") ("100") [args]
            )
                .env("RUST_BACKTRACE", "1")
                .env("MERCUTIO_SYNC_LOG", "1")
                .status()?
                .expect_success();
        }

        Cli::Replay { args } => {
            let release_flag = if release { vec!["--release"] } else { vec![] };
            cmd!(
                cargo run [release_flag] ("--bin") ("mercutio-replay") ("--") [args]
            )
                .current_dir("mercutio")
                .env("CARGO_INCREMENTAL", "1")
                .env("RUST_BACKTRACE", "1")
                .status()?
                .expect_success();
        }

        Cli::Test { args } => {
            cmd!(
                ("./transform-test.sh") [args]
            )
                .current_dir("oatie")
                .status()?
                .expect_success();
        }

        Cli::JsBuild { args } => {
            cmd!(
                npx webpack ("./src/app.ts") ("./dist/mercutio.js") [args]
            )
                .current_dir("mercutio/frontend")
                .status()?
                .expect_success();
        }

        Cli::JsWatch { args } => {
            cmd!(
                npx webpack ("--watch") ("./src/app.ts") ("./dist/mercutio.js") [args]
            )
                .current_dir("mercutio/frontend")
                .status()?
                .expect_success();
        }
    }
});
