use super::*;
use mercutio_common::SyncClientCommand;
use mercutio_common::socket::*;
use crossbeam_channel::{unbounded, Sender, Receiver};
use failure::Error;
use serde_json;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering;
use std::thread::{self, JoinHandle};
use ws;
use ws::CloseCode;
use monkey::setup_monkey;

// #[spawn]
fn spawn_send_to_client(
    rx_client: Receiver<ClientCommand>,
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
    rx: Receiver<SyncServerCommand>,
    sentinel: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while let Ok(command) = rx.recv() {
            if let SyncServerCommand::TerminateProxy = command {
                out.close(CloseCode::Away);
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
    rx: Receiver<SyncServerCommand>,
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

                    let req_parse: Result<SyncClientCommand, _> =
                        serde_json::from_slice(&msg.into_data());
                    match req_parse {
                        Err(err) => {
                            println!("Packet error: {:?}", err);
                        }
                        Ok(value) => {
                            let _ = tx_task.send(Task::SyncClientCommand(value));
                        }
                    }

                    Ok(())
                }
            }
        }).unwrap();

        // Client socket may have disconnected, and we closed
        // this connection via SyncServerCommand::TerminateProxy
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
) -> (Arc<AtomicBool>, Arc<AtomicBool>, Sender<Task>, Sender<SyncServerCommand>) {
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
    tx_sync: Sender<SyncServerCommand>,
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
        Ok(self.tx_task.send(Task::NativeCommand(msg))?)
    }

    fn cleanup(&mut self) -> Result<(), Error> {
        self.monkey.store(false, Ordering::Relaxed);
        self.alive.store(false, Ordering::Relaxed);

        self.tx_sync.send(SyncServerCommand::TerminateProxy)?;

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
