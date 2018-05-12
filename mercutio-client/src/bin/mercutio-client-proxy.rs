extern crate mercutio_common;
#[macro_use]
extern crate mercutio_client;
extern crate serde_json;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate ws;
#[macro_use]
extern crate taken;
extern crate simple_ws;
extern crate bus;
extern crate crossbeam_channel;
extern crate rand;
extern crate failure;
extern crate url;
extern crate ron;

use crossbeam_channel::{unbounded, Sender, Receiver};
use failure::Error;
use mercutio_client::*;
use simple_ws::*;
use mercutio_common::commands::*;
use std::panic;
use std::process;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use structopt::StructOpt;
use ws::CloseCode;
use rand::Rng;

#[derive(StructOpt, Debug)]
#[structopt(name = "mercutio-client", about = "An example of StructOpt usage.")]
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
                let req_parse: Result<UserToFrontendCommand, _> =
                    serde_json::from_slice(&msg.into_data());

                if let Ok(UserToFrontendCommand::Init(..)) = req_parse {
                    let command = FrontendToUserCommand::Monkey(true);
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
    rx_client: Receiver<UserToFrontendCommand>,
    out: Arc<Mutex<ws::Sender>>,
) -> JoinHandle<Result<(), Error>> {
    thread::spawn(|| -> Result<(), Error> {
        take!(rx_client, out);
        while let Ok(req) = rx_client.recv() {
            let json = serde_json::to_string(&req).unwrap();
            out.lock().unwrap().send(json)?;
        }
        Ok(())
    })
}

// #[spawn]
fn spawn_client_to_sync(
    out: ws::Sender,
    rx: Receiver<UserToSyncCommand>,
    sentinel: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while let Ok(command) = rx.recv() {
            if let UserToSyncCommand::TerminateProxy = command {
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
    rx: Receiver<UserToSyncCommand>,
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

                    let req_parse: Result<SyncToUserCommand, _> =
                        serde_json::from_slice(&msg.into_data());
                    match req_parse {
                        Err(err) => {
                            println!("Packet error: {:?}", err);
                        }
                        Ok(value) => {
                            let _ = tx_task.send(Task::SyncToUserCommand(value));
                        }
                    }

                    Ok(())
                }
            }
        }).unwrap();

        // Client socket may have disconnected, and we closed
        // this connection via UserToSyncCommand::TerminateProxy
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
) -> (Arc<AtomicBool>, Arc<AtomicBool>, Sender<Task>, Sender<UserToSyncCommand>) {
    let (tx_sync, rx_sync) = unbounded();

    let monkey = Arc::new(AtomicBool::new(false));
    let alive = Arc::new(AtomicBool::new(true));

    let (tx_client, rx_client) = unbounded();
    spawn_send_to_client(
        rx_client,
        out,
    );

    let mut client = ProxyClient {
        state: Client {
            client_id: name.to_owned(),
            client_doc: ClientDoc::new(),

            monkey: monkey.clone(),
            alive: alive.clone(),
        },

        tx_client,
        tx_sync: tx_sync.clone(),
    };

    // Send initial controls.
    client.setup_controls(None);

    let (tx_task, rx_task) = unbounded();

    // Setup monkey tasks.
    setup_monkey::<ProxyClient>(alive.clone(), monkey.clone(), tx_task.clone());

    // Connect to the sync server.
    spawn_sync_connection(
        ws_port,
        page_id.to_owned(),
        tx_task.clone(),
        rx_sync,
    );

    // Operate on all incoming tasks.
    //TODO possible to delay until init was handled?
    let _ = thread::Builder::new()
        .name(format!("setup_client({})", name))
        .spawn::<_, Result<(), Error>>(move || {
            while let Ok(task) = rx_task.recv() {
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
    tx_sync: Sender<UserToSyncCommand>,
}

impl SimpleSocket for ProxySocket {
    type Args = u16;

    fn initialize(ws_port: u16, url: &str, out: Arc<Mutex<ws::Sender>>) -> Result<ProxySocket, Error> {
        let page_id = url[1..].to_string();
        let (alive, monkey, tx_task, tx_sync) = setup_client(
            "$$$$$$",
            &page_id,
            out.clone(),
            ws_port,
        );

        Ok(ProxySocket {
            alive,
            monkey,
            tx_task,
            tx_sync,
        })
    }

    fn handle_message(&mut self, data: &[u8]) -> Result<(), Error> {
        let msg = serde_json::from_slice(&data)?;
        Ok(self.tx_task.send(Task::FrontendToUserCommand(msg))?)
    }

    fn cleanup(&mut self) -> Result<(), Error> {
        self.monkey.store(false, Ordering::Relaxed);
        self.alive.store(false, Ordering::Relaxed);

        self.tx_sync.send(UserToSyncCommand::TerminateProxy)?;

        Ok(())
    }
}

pub fn server(url: &str, ws_port: u16) {
    ws::listen(url, |out| {
        // Websocket message handler.
        SocketHandler::<ProxySocket>::new(ws_port, out)
    }).unwrap();
}

pub fn start_websocket_server(port: u16) {
    server(&format!("0.0.0.0:{}", port), port - 1);
}


#[cfg(not(target_arch="wasm32"))]
pub struct ProxyClient {
    pub state: Client,
    pub tx_client: Sender<UserToFrontendCommand>,
    pub tx_sync: Sender<UserToSyncCommand>,
}

#[cfg(not(target_arch="wasm32"))]
impl ClientImpl for ProxyClient {
    fn state(&mut self) -> &mut Client {
        &mut self.state
    }

    fn send_client(&self, req: &UserToFrontendCommand) -> Result<(), Error> {
        log_wasm!(SendClient(req.clone()));
        self.tx_client.send(req.clone())?;
        Ok(())
    }

    fn send_sync(&self, req: UserToSyncCommand) -> Result<(), Error> {
        log_wasm!(SendSync(req.clone()));
        self.tx_sync.send(req)?;
        Ok(())
    }
}

macro_rules! spawn_monkey_task {
    ( $alive:expr, $monkey:expr, $tx:expr, $wait_params:expr, $task:expr ) => {
        {
            let tx = $tx.clone();
            let alive = $alive.clone();
            let monkey = $monkey.clone();
            thread::spawn::<_, Result<(), Error>>(move || {
                let mut rng = rand::thread_rng();
                while alive.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(
                        rng.gen_range($wait_params.0, $wait_params.1),
                    ));
                    if monkey.load(Ordering::Relaxed) {
                        tx.send(Task::FrontendToUserCommand($task))?;
                    }
                }
                Ok(())
            })
        }
    };
}

pub type MonkeyParam = (u64, u64);

// "Human-like"
pub const MONKEY_BUTTON: MonkeyParam = (0, 1500);
pub const MONKEY_LETTER: MonkeyParam = (0, 200);
pub const MONKEY_ARROW: MonkeyParam = (0, 500);
pub const MONKEY_BACKSPACE: MonkeyParam = (0, 300);
pub const MONKEY_ENTER: MonkeyParam = (6_000, 10_000);
pub const MONKEY_CLICK: MonkeyParam = (400, 1000);

// Race
// const MONKEY_BUTTON: MonkeyParam = (0, 0, 100);
// const MONKEY_LETTER: MonkeyParam = (0, 0, 100);
// const MONKEY_ARROW: MonkeyParam = (0, 0, 100);
// const MONKEY_BACKSPACE: MonkeyParam = (0, 0, 100);
// const MONKEY_ENTER: MonkeyParam = (0, 0, 1_000);

#[allow(unused)]
pub fn setup_monkey<C: ClientImpl + Sized>(alive: Arc<AtomicBool>, monkey: Arc<AtomicBool>, tx: Sender<Task>) {

    spawn_monkey_task!(alive, monkey, tx, MONKEY_BUTTON, {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0, button_handlers::<C>(None).len() as u32);
        FrontendToUserCommand::Button(index)
    });

    spawn_monkey_task!(alive, monkey, tx, MONKEY_LETTER, {
        let mut rng = rand::thread_rng();
        let char_list = vec![
            rng.gen_range(b'A', b'Z'),
            rng.gen_range(b'a', b'z'),
            rng.gen_range(b'0', b'9'),
            b' ',
        ];
        let c = *rng.choose(&char_list).unwrap() as u32;
        FrontendToUserCommand::Character(c)
    });

    spawn_monkey_task!(alive, monkey, tx, MONKEY_ARROW, {
        let mut rng = rand::thread_rng();
        let key = *rng.choose(&[37, 39, 37, 39, 37, 39, 38, 40]).unwrap();
        FrontendToUserCommand::Keypress(key, false, false, false)
    });

    spawn_monkey_task!(alive, monkey, tx, MONKEY_BACKSPACE, {
        FrontendToUserCommand::Keypress(8, false, false, false)
    });

    spawn_monkey_task!(alive, monkey, tx, MONKEY_ENTER, {
        FrontendToUserCommand::Keypress(13, false, false, false)
    });

    spawn_monkey_task!(alive, monkey, tx, MONKEY_CLICK, {
        let mut rng = rand::thread_rng();
        FrontendToUserCommand::RandomTarget(rng.gen::<f64>())
    });
}
