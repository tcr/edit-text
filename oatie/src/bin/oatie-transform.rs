extern crate oatie;

use oatie::schema::RtfSchema;
use oatie::transform_test::*;
use std::io;
use std::io::prelude::*;

fn main() {
    let mut input = String::new();
    let stdin = io::stdin();
    stdin
        .lock()
        .read_to_string(&mut input)
        .expect("Could not read stdin");

    match run_transform_test::<RtfSchema>(&input) {
        Ok(..) => {
            println!("all set!");
        }
        Err(err) => {
            eprintln!("transform test error: {:?}", err);
            ::std::process::exit(1);
        }
    }
}
