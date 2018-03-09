extern crate mercutio;
extern crate serde_json;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate ws;

use structopt::StructOpt;
use std::thread;
use std::panic;
use std::process;
use std::time::Duration;
use mercutio::wasm::{ClientCommand, NativeCommand};
use mercutio::wasm::proxy::start_websocket_server;

#[derive(StructOpt, Debug)]
#[structopt(name = "mercutio-wasm", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(long = "monkies", help = "Monkey count")]
    monkies: Option<usize>,

    #[structopt(long = "port", help = "Port", default_value = "8002")]
    port: u16,
}

pub fn main() {
    // Set aborting process handler.
    let orig_handler = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        orig_handler(panic_info);
        process::exit(1);
    }));

    println!("started \"wasm\" server");

    let opt = Opt::from_args();
    let port = opt.port;
    let monkies = opt.monkies;

    if monkies.is_some() {
        virtual_monkeys();
    }

    start_websocket_server(port);
}

fn spawn_virtual_monkey() -> JoinHandle<()> {
    thread::spawn(move || {
        let url = format!(
            "ws://127.0.0.1:{}/{}",
            port,
            "monkey",
        );
        println!("Connecting to {:?}", url);

        ws::connect(url.as_str(), move |out| {
            thread::sleep(Duration::from_millis(1000 + ((key as u64) * 400)));

            // Ignore all incoming messages, as we have no client to update
            move |msg: ws::Message| {
                // println!("wasm got a packet from sync '{}'. ", msg);
                let req_parse: Result<ClientCommand, _> =
                    serde_json::from_slice(&msg.into_data());

                if let Ok(ClientCommand::Init(..)) = req_parse {
                    let command = NativeCommand::Monkey(true);
                    let json = serde_json::to_string(&command).unwrap();
                    out.send(json.as_str()).unwrap();
                    // monkey_started.store(true, Ordering::Relaxed);
                }

                Ok(())
            }
        }).unwrap();
    })
}

fn spawn_virtual_monkies() -> JoinHandle<()> {
    thread::spawn(move || {
        let opt = Opt::from_args();
        let port = opt.port;
        let monkies = opt.monkies.unwrap();
        
        thread::sleep(Duration::from_millis(1000));

        for key in 0..monkies {
            spawn_virtual_monkey();
        }
    })
}

fn virtual_monkeys() {
    println!("(!) virtual monkeys enabled");

    spawn_virtual_monkies();
}
