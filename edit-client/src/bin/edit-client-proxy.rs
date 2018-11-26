extern crate edit_client;
extern crate edit_common;
extern crate serde_json;
extern crate structopt;
extern crate structopt_derive;
extern crate ws;
#[macro_use]
extern crate taken;
extern crate bus;
extern crate crossbeam_channel;
extern crate failure;
extern crate rand;
extern crate ron;
extern crate url;

use crossbeam_channel::{
    unbounded,
    Receiver,
    Sender,
};
use edit_client::{
    monkey::*,
    proxy::*,
    *,
};
use edit_common::{
    commands::*,
    simple_ws::*,
};
use failure::Error;
use std::{
    panic,
    process,
    sync::atomic::AtomicBool,
    sync::atomic::Ordering,
    sync::{
        Arc,
        Mutex,
    },
    thread::{
        self,
        JoinHandle,
    },
    time::Duration,
};
use structopt::StructOpt;
use ws::CloseCode;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "edit-client",
    about = "An example of StructOpt usage."
)]
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

fn spawn_virtual_monkey(port: u16, key: usize) -> JoinHandle<()> {
    thread::spawn(move || {
        let url = format!("ws://127.0.0.1:{}/{}", port, "monkey",);
        println!("Connecting to {:?}", url);

        ws::connect(url.as_str(), move |out| {
            thread::sleep(Duration::from_millis(1000 + ((key as u64) * 400)));

            // Ignore all incoming messages, as we have no client to update
            move |msg: ws::Message| {
                // println!("wasm got a packet from sync '{}'. ", msg);
                let req_parse: Result<FrontendCommand, _> =
                    serde_json::from_slice(&msg.into_data());

                if let Ok(FrontendCommand::Init(..)) = req_parse {
                    let command = ControllerCommand::Monkey { enabled: true };
                    let json = serde_json::to_string(&command).unwrap();
                    out.send(json.as_str()).unwrap();
                    // monkey_started.store(true, Ordering::Relaxed);
                }

                Ok(())
            }
        })
        .unwrap();
    })
}

fn spawn_virtual_monkies() -> JoinHandle<()> {
    thread::spawn(move || {
        let opt = Opt::from_args();
        let port = opt.port;
        let monkies = opt.monkies.unwrap();

        thread::sleep(Duration::from_millis(1000));

        for key in 0..monkies {
            spawn_virtual_monkey(port, key);
        }
    })
}

fn virtual_monkeys() {
    println!("(!) virtual monkeys enabled");

    spawn_virtual_monkies();
}

// #[spawn]
fn spawn_send_to_client(
    rx_client: Receiver<FrontendCommand>,
    out: Arc<Mutex<ws::Sender>>,
) -> JoinHandle<Result<(), Error>> {
    thread::spawn(|| -> Result<(), Error> {
        take!(rx_client, out);
        while let Some(req) = rx_client.recv() {
            let json = serde_json::to_string(&req).unwrap();
            out.lock().unwrap().send(json)?;
        }
        Ok(())
    })
}

// #[spawn]
fn spawn_client_to_sync(
    out: ws::Sender,
    rx: Receiver<ServerCommand>,
    sentinel: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while let Some(command) = rx.recv() {
            if let ServerCommand::TerminateProxy = command {
                let _ = out.close(CloseCode::Away);
                sentinel.store(false, Ordering::SeqCst);
                break;
            } else {
                out.send(serde_json::to_string(&command).unwrap()).unwrap();
            }
        }
    })
}

// #[spawn]
fn spawn_sync_connection(
    ws_port: u16,
    page_id: String,
    tx_task: Sender<Task>,
    rx: Receiver<ServerCommand>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let sentinel = Arc::new(AtomicBool::new(true));
        ws::connect(format!("ws://127.0.0.1:{}/$/ws/{}", ws_port, page_id), {
            let sentinel = sentinel.clone();

            move |out| {
                // While we receive packets from the client, send them to sync.
                spawn_client_to_sync(out, rx.clone(), sentinel.clone());

                // Receive packets from sync and act on them.
                let tx_task = tx_task.clone();
                move |msg: ws::Message| {
                    // Handle messages received on this connection
                    // println!("wasm got a packet from sync '{}'. ", msg);

                    let req_parse: Result<ClientCommand, _> =
                        serde_json::from_slice(&msg.into_data());
                    match req_parse {
                        Err(err) => {
                            println!("Packet error: {:?}", err);
                        }
                        Ok(value) => {
                            let _ = tx_task.send(Task::ClientCommand(value));
                        }
                    }

                    Ok(())
                }
            }
        })
        .unwrap();

        // Client socket may have disconnected, and we closed
        // this connection via ServerCommand::TerminateProxy
        if sentinel.load(Ordering::SeqCst) == true {
            // Child client didn't disconnect us, invalid
            unreachable!("Server connection cut");
        }
    })
}

fn setup_client(
    name: &str,
    page_id: &str,
    out: Arc<Mutex<ws::Sender>>,
    ws_port: u16,
) -> (
    Arc<AtomicBool>,
    Arc<AtomicBool>,
    Sender<Task>,
    Sender<ServerCommand>,
) {
    let (tx_sync, rx_sync) = unbounded();

    // Initialize logger.
    crate::log::log_init(tx_sync.clone());

    let monkey = Arc::new(AtomicBool::new(false));
    let alive = Arc::new(AtomicBool::new(true));

    let (tx_client, rx_client) = unbounded();
    spawn_send_to_client(rx_client, out);

    let mut client = ProxyClient {
        state: Client {
            client_id: name.to_owned(),
            client_doc: ClientDoc::new(),
            last_controls: None,

            monkey: monkey.clone(),
            alive: alive.clone(),
            task_count: 0,
        },

        tx_client,
        tx_sync: tx_sync.clone(),
    };

    // Send initial controls.
    client.setup_controls(None);

    let (tx_task, rx_task) = unbounded();

    // Setup monkey tasks.
    setup_monkey::<ProxyClient>(Scheduler::new(
        tx_task.clone(),
        alive.clone(),
        monkey.clone(),
    ));

    // Connect to the sync server.
    spawn_sync_connection(ws_port, page_id.to_owned(), tx_task.clone(), rx_sync);

    // Operate on all incoming tasks.
    //TODO possible to delay naming or spawning until init was handled?
    let tx_sync_2 = tx_sync.clone();
    let _ = thread::Builder::new()
        .name(format!("setup_client({})", name))
        .spawn::<_, Result<(), Error>>(move || {
            // TODO can we inherit thread locals??
            crate::log::log_init(tx_sync_2.clone());

            while let Some(task) = rx_task.recv() {
                client.handle_task(task)?;
            }
            Ok(())
        });

    (alive, monkey, tx_task, tx_sync)
}

pub struct ProxySocket {
    alive: Arc<AtomicBool>,
    monkey: Arc<AtomicBool>,
    tx_task: Sender<Task>,
    tx_sync: Sender<ServerCommand>,
}

impl SimpleSocket for ProxySocket {
    type Args = u16;

    fn initialize(
        ws_port: u16,
        url: &str,
        out: Arc<Mutex<ws::Sender>>,
    ) -> Result<ProxySocket, Error> {
        let page_id = url[1..].to_string();
        let (alive, monkey, tx_task, tx_sync) =
            setup_client("$$$$$$", &page_id, out.clone(), ws_port);

        Ok(ProxySocket {
            alive,
            monkey,
            tx_task,
            tx_sync,
        })
    }

    fn handle_message(&mut self, data: &[u8]) -> Result<(), Error> {
        let msg = serde_json::from_slice(&data)?;
        Ok(self.tx_task.send(Task::ControllerCommand(msg)))
    }

    fn cleanup(&mut self) -> Result<(), Error> {
        self.monkey.store(false, Ordering::Relaxed);
        self.alive.store(false, Ordering::Relaxed);

        self.tx_sync.send(ServerCommand::TerminateProxy);

        Ok(())
    }
}

pub fn server(url: &str, ws_port: u16) {
    ws::listen(url, |out| {
        // Websocket message handler.
        SocketHandler::<ProxySocket>::new(ws_port, out)
    })
    .unwrap();
}

pub fn start_websocket_server(port: u16) {
    server(&format!("0.0.0.0:{}", port), port - 1);
}
