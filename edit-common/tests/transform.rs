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
        .join("edit-common/tests/transform/");

    eprintln!("root_path: {:?}", root_path);

    let mut tests = vec![];
    for entry in fs::read_dir(&root_path).unwrap() {
        if let Ok(entry) = entry {
            if entry.metadata().unwrap().is_file() {
                tests.push(entry.path());
            }
        }
    }

    tests.sort();
    for file in tests {
        let value = fs::read_to_string(&file).unwrap();
        println!();
        println!("[{:?}]", file);
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
