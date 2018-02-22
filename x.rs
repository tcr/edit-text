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
    #[structopt(name = "test")]
    Test {
        args: Vec<String>
    },

    #[structopt(name = "mercutio-replay")]
    MercutioReplay {
        args: Vec<String>
    },

    #[structopt(name = "wasm-proxy")]
    WasmProxy {
        args: Vec<String>,
    },

    #[structopt(name = "mercutio-sync")]
    MercutioSyncRun {
        #[structopt(long = "no-log", help = "Do not export log")]
        no_log: bool,
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
        Cli::Test { args } => {
            cmd!(
                ("./transform-test.sh") [args]
            )
                .current_dir("oatie")
                .status()?;
        }

        Cli::MercutioReplay { args } => {
            cmd!(
                cargo run ("--release") ("--bin") ("mercutio-replay") ("--") [args]
            )
                .current_dir("mercutio")
                .env("RUST_BACKTRACE", "1")
                .env("CARGO_INCREMENTAL", "1")
                .status()?;
        }

        Cli::WasmProxy { args } => {
            cmd!(
                cargo run ("--release") ("--bin") ("mercutio-wasm-proxy") ("--") [args]
            )
                .current_dir("mercutio")
                .env("RUST_BACKTRACE", "1")
                .env("CARGO_INCREMENTAL", "1")
                .env("MERCUTIO_WASM_LOG", "1")
                .status()?;
        }

        Cli::MercutioSyncRun { no_log, args } => {
            let release_flag = if release { vec!["--release"] } else { vec![] };
            cmd!(
                cargo run ("--bin") ("mercutio-sync") [release_flag] ("--") ("--period") ("100") [args]
            )
                .current_dir("mercutio")
                .env("RUST_BACKTRACE", "1")
                .env("CARGO_INCREMENTAL", "1")
                .env("MERCUTIO_SYNC_LOG", if no_log { "0" } else { "1" })
                .status()?;
        }

        Cli::MercutioSyncBuild { args } => {
            let release_flag = if release { vec!["--release"] } else { vec![] };
            cmd!(
                cargo build ("--bin") ("mercutio-sync") [release_flag] ("--") ("--period") ("100") [args]
            )
                .current_dir("mercutio")
                .env("RUST_BACKTRACE", "1")
                .env("CARGO_INCREMENTAL", "1")
                .status()?;
        }

        Cli::MercutioSyncCallgrind { args } => {
            cmd!(
                cargo build ("--release") ("--bin") ("mercutio-sync")
            )
                .current_dir("mercutio")
                .env("CARGO_INCREMENTAL", "1")
                .status()?;

            cmd!(
                cargo profiler callgrind ("--bin") ("./target/release/mercutio-sync") ("--") ("--period") ("100") [args]
            )
                .env("RUST_BACKTRACE", "1")
                .env("MERCUTIO_SYNC_LOG", "1")
                .status()?;
        }
    }
});
