#!/usr/bin/env run-cargo-script
//! ```cargo
//! [dependencies]
//! command-macros = "*"
//! failure = "*"
//! quicli = "*"
//! ```

#[macro_use]
extern crate command_macros;
#[macro_use]
extern crate quicli;

use quicli::prelude::*;

// Add cool slogan for your app here, e.g.:
/// Get first n lines of a file
#[derive(StructOpt)]
#[structopt(name = "edit-text", about = "scripts")]
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
        args: Vec<String>,
    },

    #[structopt(name = "mercutio-sync-callgrind")]
    MercutioSyncCallgrind {
        args: Vec<String>,
    },
}

main!(|args: Cli| {
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

        Cli::MercutioSyncRun { args } => {
            cmd!(
                cargo run ("--bin") ("mercutio-sync") ("--") ("--period") ("100") [args]
            )
                .current_dir("mercutio")
                .env("RUST_BACKTRACE", "1")
                .env("CARGO_INCREMENTAL", "1")
                .env("MERCUTIO_SYNC_LOG", "1")
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
