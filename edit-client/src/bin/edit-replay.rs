use ron;
#[macro_use]
extern crate maplit;

#[macro_use]
extern crate quicli;

// use quicli::prelude::*;
use colored::Colorize;
use crossbeam_channel::{
    unbounded,
    Receiver,
};
use edit_client::{
    client::ClientDoc,
    log::*,
    proxy::ProxyClientController,
    Client,
    ClientController,
};
use edit_common::commands::*;
use failure::Error;
use std::cell::RefCell;
use std::io::prelude::*;
use std::rc::Rc;
use std::sync::{
    atomic::AtomicBool,
    Arc,
};
use structopt::StructOpt;

fn init_new_client(
    client_id: &str,
) -> (
    ProxyClientController,
    Receiver<FrontendCommand>,
    Receiver<ServerCommand>,
) {
    let (tx_client, rx_client) = unbounded();
    let (tx_sync, rx_sync) = unbounded();
    let client = ProxyClientController {
        state: Rc::new(RefCell::new(Client {
            client_id: client_id.to_owned(),
            client_doc: ClientDoc::new(),
            last_controls: None,

            monkey: Arc::new(AtomicBool::new(false)),
            alive: Arc::new(AtomicBool::new(true)),
            task_count: 0,
        })),

        tx_client,
        tx_sync,
    };
    (client, rx_client, rx_sync)
}

#[derive(StructOpt)]
struct Opt {
    #[structopt(long = "filter")]
    filter: Option<String>,
}

main!(|opts: Opt| {
    let (tx_line, rx_line) = unbounded();
    ::std::thread::spawn(move || -> Result<(), Error> {
        // let f = ::std::fs::File::open("../logs/client")?;
        // let file = ::std::io::BufReader::new(&*f);
        let file = ::std::io::stdin();

        for line in file.lock().lines() {
            if let Ok(line) = line {
                if line.trim().len() != 0 {
                    let hi: LogWasm = ron::de::from_str(&line)?;
                    tx_line.send(hi);
                }
            }
        }

        Ok(())
    });

    let mut clients = hashmap![];

    let mut i = 0;

    if let Some(ref filter_id) = opts.filter {
        println!("\n!!! Using filter {:?}\n", filter_id);
    }

    while let Some(hi) = rx_line.recv() {
        i += 1;
        println!("TASK ~~~~ {:?} ~~~~", i);
        match hi {
            LogWasm::Setup(client_id) => {
                clients.insert(client_id.clone(), init_new_client(&client_id));
            }
            LogWasm::Task(client_id, task) => {
                // TODO real command-line subfilters
                // if let Some(ref filter_id) = opts.filter {
                //     if client_id != *filter_id {
                //         continue;
                //     }
                // }

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

    eprintln!();
    eprintln!("(edit-replay is done.)");
});
