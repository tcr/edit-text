extern crate mercutio;
extern crate serde_json;
extern crate ws;

use std::thread;
use mercutio::wasm::start_websocket_server;
use std::time::Duration;
use mercutio::wasm::NativeCommand;

pub fn main() {
    println!("started \"wasm\" server");
    virtual_monkeys();
    start_websocket_server();
}

#[cfg(feature="virtual_monkeys")]
fn virtual_monkeys() {
    println!("(!) virtual monkeys enabled");

    thread::spawn(|| {
        thread::sleep(Duration::from_millis(1000));

        for key in 0..8 {
            thread::spawn(move || {
                let url = format!("ws://127.0.0.1:3012/{}", ('a' as u8 + key as u8) as char);
                println!("Connecting to {:?}", url);

                ws::connect(url.as_str(), move |out| {
                    thread::sleep(Duration::from_millis(1000 + ((key as u64) * 200)));
                    
                    // Start monkey
                    let command = NativeCommand::Monkey(true);
                    let json = serde_json::to_string(&command).unwrap();
                    out.send(json.as_str()).unwrap();

                    move |msg: ws::Message| {
                        // println!("wasm got a packet from sync '{}'. ", msg);

                        Ok(())
                    }
                }).unwrap();
            });
        }
    });
}

#[cfg(not(feature="virtual_monkeys"))]
fn virtual_monkeys() {
    println!("(!) virtual monkeys disabled")
}
