#![feature(crate_in_paths)]

extern crate failure;
extern crate ron;
extern crate mercutio;

use ::failure::Error;
use ::std::io::prelude::*;
use mercutio::wasm::LogWasm;

fn main() {
    run().expect("No errors");
}

fn run() -> Result<(), Error> {
    let mut f = ::std::fs::File::open("log/client")?;
    let mut file = ::std::io::BufReader::new(&f);
    for line in file.lines() {
        if let Ok(line) = line {
            if line.trim().len() != 0 {
                let hi: LogWasm = ron::de::from_str(&line)?;

                /*
                let client = Client {
                    //
                };
                */

                match hi {
                    LogWasm::Task(client_id, task) => {
                        println!("{:?}: {:?}", client_id, task);
                    }
                    _ => {}
                }
            }
        }
    }
    println!("hi sweetie {:?}", f);
    Ok(())
}
