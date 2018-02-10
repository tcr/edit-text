#![feature(crate_in_paths, use_nested_groups)]

extern crate failure;
extern crate ron;
extern crate mercutio;
extern crate crossbeam_channel;

use failure::Error;
use std::io::prelude::*;
use mercutio::wasm::{
    Client,
    LogWasm,
    state::ClientDoc,
};
use std::sync::{
    atomic::AtomicBool,
    Arc
};
use crossbeam_channel::unbounded;

fn main() {
    run().expect("No errors");
}

fn run() -> Result<(), Error> {
    let mut f = ::std::fs::File::open("log/client")?;
    let mut file = ::std::io::BufReader::new(&f);


    let (tx_client, rx_client) = unbounded();
    let (tx_sync, rx_sync) = unbounded();
    let mut client = Client {
        client_id: "unknown".to_string(),
        client_doc: ClientDoc::new(),

        monkey: Arc::new(AtomicBool::new(false)),
        alive: Arc::new(AtomicBool::new(true)),

        tx_client,
        tx_sync,
    };
    
    for line in file.lines() {
        if let Ok(line) = line {
            if line.trim().len() != 0 {
                let hi: LogWasm = ron::de::from_str(&line)?;

                match hi {
                    LogWasm::Task(client_id, task) => {
                        println!("{:?}: {:?}", client_id, task);
                        println!();
                        client.handle_task(task)?;
                    }
                    _ => {}
                }
            }
        }
    }
    println!("hi sweetie {:?}", f);
    Ok(())
}
