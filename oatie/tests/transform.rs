extern crate oatie;

use oatie::schema::RtfSchema;
use oatie::transform_test::*;
// use std::io;
// use std::io::prelude::*;
use std::fs;

#[test]
fn main() {
    let root_path = &::std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("oatie/tests/transform/");

    // eprintln!("HELP: {:?}", root_path);
    let paths = fs::read_dir(&root_path).unwrap();

    for entry in paths {
        // println!("Name: {}", path.unwrap().path().display()

        if let Ok(entry) = entry {
            if entry.metadata().unwrap().is_file() {
                let value = fs::read_to_string(entry.path()).unwrap();
                match run_transform_test::<RtfSchema>(&value) {
                    Ok(..) => {
                        println!("all set!");
                    }
                    Err(err) => {
                        eprintln!("transform test error: {:?}", err);
                        ::std::process::exit(1);
                    }
                }
            }
        }
    }
}
