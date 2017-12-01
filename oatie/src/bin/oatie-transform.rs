extern crate oatie;

use std::io;
use std::io::prelude::*;

use oatie::transform_test::run_transform_test;

fn main() {
    let mut input = String::new();
    let stdin = io::stdin();
    stdin
        .lock()
        .read_to_string(&mut input)
        .expect("Could not read stdin");

    match run_transform_test(&input) {
        Ok(..) => {
            println!("all set!");
        }
        Err(err) => {
            println!("transform error: {}", err);
            ::std::process::exit(1);
        }
    }
}
