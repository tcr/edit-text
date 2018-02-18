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
#[structopt(name = "git", about = "the stupid content tracker")]
enum Cli {
    #[structopt(name = "test")]
    Test {
        args: Vec<String>
    },

    #[structopt(name = "replay")]
    Replay {
        args: Vec<String>
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

        Cli::Replay { args } => {
            cmd!(
                cargo run ("--release") ("--bin") ("mercutio-replay") ("--") [args]
            )
                .current_dir("mercutio")
                .env("RUST_BACKTRACE", "1")
                .env("CARGO_INCREMENTAL", "1")
                .status()?;
        }
    }
});
