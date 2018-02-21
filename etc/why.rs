// cargo-deps: quicli = "0.2" maplit="*" ron="*" serde="*"

#[macro_use] extern crate quicli;
#[macro_use] extern crate maplit;
extern crate ron;
#[macro_use] extern crate serde;

use quicli::prelude::*;
use ron::ser::to_string;

// Add cool slogan for your app here, e.g.:
/// Get first n lines of a file
#[derive(Debug, StructOpt)]
struct Cli {
    // Add a CLI argument `--count`/-n` that defaults to 3, and has this help text:
    // How many lines to get
    // #[structopt(long = "count", short = "n", default_value = "3")]
    // count: usize,
    // // Add a positional argument that the user has to supply:
    // /// The file to read
    // file: String,
    // /// Pass many times for more log output
    // #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    // verbosity: u8,
}

#[derive(Debug, Serialize)]
enum DocElem {
    DocGroup(::std::collections::HashMap<String, String>, Vec<DocElem>),
    DocChars(String),
}

main!(|args: Cli| {
    println!("hey {:?}", args);

    let d = DocElem::DocGroup(hashmap!{
        "alpha".into() => "beta".into(),
    }, vec![DocElem::DocChars("hi\ntim".to_string())]);

    println!("ron: {}", to_string(&d).unwrap());
});

