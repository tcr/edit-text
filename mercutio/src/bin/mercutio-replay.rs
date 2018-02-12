#![feature(crate_in_paths, use_nested_groups)]

extern crate failure;
extern crate ron;
extern crate mercutio;
extern crate crossbeam_channel;
#[macro_use]
extern crate maplit;
extern crate colored;

use colored::Colorize;
use failure::Error;
use std::io::prelude::*;
use mercutio::{
    SyncServerCommand,
    wasm::{
        Client,
        LogWasm,
        state::ClientDoc,
        ClientCommand,
    },
};
use std::sync::{
    atomic::AtomicBool,
    Arc
};
use crossbeam_channel::{Receiver, unbounded};

fn main() {
    run().expect("No errors");
}

fn init_new_client(client_id: &str) -> (Client, Receiver<ClientCommand>, Receiver<SyncServerCommand>) {
    let (tx_client, rx_client) = unbounded();
    let (tx_sync, rx_sync) = unbounded();
    let mut client = Client {
        client_id: client_id.to_owned(),
        client_doc: ClientDoc::new(),

        monkey: Arc::new(AtomicBool::new(false)),
        alive: Arc::new(AtomicBool::new(true)),

        tx_client,
        tx_sync,
    };
    (client, rx_client, rx_sync)
}

fn run() -> Result<(), Error> {
    let f = ::std::fs::File::open("log/client")?;
    let file = ::std::io::BufReader::new(&f);

    let mut clients = hashmap![];
    
    for line in file.lines() {
        if let Ok(line) = line {
            if line.trim().len() != 0 {
                let hi: LogWasm = ron::de::from_str(&line)?;

                match hi {
                    LogWasm::Setup(client_id) => {
                        clients.insert(client_id.clone(), init_new_client(&client_id));
                    }
                    LogWasm::Task(client_id, task) => {
                        println!("{}", format!("{:?}: {:?}", client_id, task).green().bold());
                        println!();
                        match clients.get_mut(&client_id) {
                            Some(&mut (ref mut client, _, _)) => {
                                client.handle_task(task)?;
                            }
                            None => {
                                panic!("Client {:?} was not set up.", client_id);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    println!("hi sweetie {:?}", f);
    Ok(())
}
