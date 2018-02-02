use super::actions::*;
use super::*;
use super::super::{SyncClientCommand, SyncServerCommand};
use crossbeam_channel::{unbounded, Sender};
use failure::Error;
use oatie::OT;
use oatie::doc::*;
use rand;
use rand::Rng;
use serde_json;
use std::{panic, process};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use ws;

macro_rules! clone_all {
    ( $( $x:ident ),* ) => {
        $(let $x = $x.clone();)*
    };
}

macro_rules! monkey_task {
    ( $alive:expr, $monkey:expr, $tx:expr, $wait_params:expr, $task:expr ) => {
        {
            let tx = $tx.clone();
            let alive = $alive.clone();
            let monkey = $monkey.clone();
            thread::spawn::<_, Result<(), Error>>(move || {
                let mut rng = rand::thread_rng();
                while alive.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(
                        $wait_params.0 + rng.gen_range($wait_params.1, $wait_params.2),
                    ));
                    if monkey.load(Ordering::Relaxed) {
                        tx.send($task)?;
                    }
                }
                Ok(())
            })
        }
    };
}

pub type MonkeyParam = (u64, u64, u64);

// "Human-like"
pub const MONKEY_BUTTON: MonkeyParam = (500, 0, 2000);
pub const MONKEY_LETTER: MonkeyParam = (50, 0, 200);
pub const MONKEY_ARROW: MonkeyParam = (0, 0, 500);
pub const MONKEY_BACKSPACE: MonkeyParam = (0, 0, 200);
pub const MONKEY_ENTER: MonkeyParam = (600, 0, 3_000);

// Race
// const MONKEY_BUTTON: MonkeyParam = (0, 0, 100);
// const MONKEY_LETTER: MonkeyParam = (0, 0, 100);
// const MONKEY_ARROW: MonkeyParam = (0, 0, 100);
// const MONKEY_BACKSPACE: MonkeyParam = (0, 0, 100);
// const MONKEY_ENTER: MonkeyParam = (0, 0, 1_000);

#[allow(unused)]
fn setup_monkey(alive: Arc<AtomicBool>, monkey: Arc<AtomicBool>, tx: Sender<Task>) {
    // Button monkey.
    monkey_task!(alive, monkey, tx, MONKEY_BUTTON, Task::ButtonMonkey);
    monkey_task!(alive, monkey, tx, MONKEY_LETTER, Task::LetterMonkey);
    monkey_task!(alive, monkey, tx, MONKEY_ARROW, Task::ArrowMonkey);
    monkey_task!(alive, monkey, tx, MONKEY_BACKSPACE, Task::BackspaceMonkey);
    monkey_task!(alive, monkey, tx, MONKEY_ENTER, Task::EnterMonkey);
}

fn setup_client(name: &str, out: ws::Sender, ws_port: u16) -> (Arc<AtomicBool>, Arc<AtomicBool>, Sender<Task>) {
    let (tx, rx) = unbounded();

    let monkey = Arc::new(AtomicBool::new(false));
    let alive = Arc::new(AtomicBool::new(true));

    let mut client = Client {
        client_id: name.to_owned(),
        client_doc: ClientDoc {
            doc: Doc(vec![]),
            version: 100,

            original_doc: Doc(vec![]),
            pending_op: None,
            local_op: Op::empty(),
        },

        monkey: monkey.clone(),
        alive: alive.clone(),

        out,
        tx,
    };

    // Send initial setup packet.
    client.setup();

    let (tx_task, rx_task) = unbounded();

    // Setup monkey tasks.
    {
        setup_monkey(alive.clone(), monkey.clone(), tx_task.clone());
    }

    // Connect to the sync server.
    {
        clone_all!(tx_task);
        thread::spawn(move || {
            ws::connect(format!("ws://127.0.0.1:{}", ws_port), move |out| {
                // While we receive packets, send them to the websocket.
                {
                    clone_all!(rx);
                    thread::spawn(move || {
                        while let Ok(command) = rx.recv() {
                            out.send(serde_json::to_string(&command).unwrap()).unwrap();
                        }
                    });
                }

                // Receive packets from sync and act on them.
                {
                    clone_all!(tx_task);
                    move |msg: ws::Message| {
                        // Handle messages received on this connection
                        println!("wasm got a packet from sync '{}'. ", msg);

                        let req_parse: Result<SyncClientCommand, _> =
                            serde_json::from_slice(&msg.into_data());
                        match req_parse {
                            Err(err) => {
                                println!("Packet error: {:?}", err);
                            }
                            Ok(value) => {
                                tx_task.send(Task::SyncClientCommand(value));
                            }
                        }

                        Ok(())
                    }
                }
            }).unwrap();
            panic!("sync server socket disconnected.")
        });
    }

    // Operate on all incoming tasks.
    thread::spawn::<_, Result<(), Error>>(move || {
        while let Ok(task) = rx_task.recv() {
            client.handle_task(task)?;
        }
        Ok(())
    });

    (alive, monkey, tx_task)
}

pub struct SocketHandler {
    ws_port: u16,
    out: Option<ws::Sender>,
    alive: Option<Arc<AtomicBool>>,
    monkey: Option<Arc<AtomicBool>>,
    tx_task: Option<Sender<Task>>,
}

impl ws::Handler for SocketHandler {
    fn on_open(&mut self, shake: ws::Handshake) -> Result<(), ws::Error> {
        let client_id = shake.request.resource()[1..].to_string();
        let (alive, monkey, tx_task) = setup_client(&client_id, self.out.take().unwrap(), self.ws_port);
        self.alive = Some(alive);
        self.monkey = Some(monkey);
        self.tx_task = Some(tx_task);
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> Result<(), ws::Error> {
        // Handle messages received on this connection
        println!("client command: '{}'. ", msg);

        let req_parse: Result<NativeCommand, _> = serde_json::from_slice(&msg.into_data());
        match req_parse {
            Err(err) => {
                println!("Packet error: {:?}", err);
            }
            Ok(value) => {
                self.tx_task.as_mut().map(|x| x.send(Task::NativeCommand(value)));
            }
        }

        Ok(())
    }

    fn on_error(&mut self, err: ws::Error) {
        println!("Killing after error");
        self.monkey.as_ref().unwrap().store(false, Ordering::Relaxed);
        self.alive.as_ref().unwrap().store(false, Ordering::Relaxed);
    }

    fn on_close(&mut self, _code: ws::CloseCode, _reason: &str) {
        println!("Killing after close");
        self.monkey.as_ref().unwrap().store(false, Ordering::Relaxed);
        self.alive.as_ref().unwrap().store(false, Ordering::Relaxed);
    }
}

pub fn server(url: &str, ws_port: u16) {
    ws::listen(url, |out| {
        // Websocket message handler.
        SocketHandler {
            ws_port,
            out: Some(out),
            alive: None,
            monkey: None,
            tx_task: None,
        }
    }).unwrap();
}

pub fn start_websocket_server(port: u16) {
    server(&format!("127.0.0.1:{}", port), port - 1);
}
