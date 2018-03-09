use super::*;
use super::super::SyncClientCommand;
use crossbeam_channel::{unbounded, Sender, Receiver};
use failure::Error;
use serde_json;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::{self, JoinHandle};
use ws;
use wasm::monkey::setup_monkey;

fn spawn_send_to_client(
    rx_client: Receiver<ClientCommand>,
    out: ws::Sender,
) -> JoinHandle<Result<(), Error>> {
    thread::spawn(|| -> Result<(), Error> {
        take!(rx_client, out);
        while let Ok(req) = rx_client.recv() {
            let json = serde_json::to_string(&req).unwrap();
            out.send(json)?;
        }
        Ok(())
    })
}


fn spawn_client_to_sync(
    out: ws::Sender,
    rx: Receiver<SyncServerCommand>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while let Ok(command) = rx.recv() {
            log_wasm!(Debug("HI TIM HAVE PACKET TO SEND TO SERVER".to_string()));
            out.send(serde_json::to_string(&command).unwrap()).unwrap();
        }
    })
}

fn spawn_sync_connection(
    ws_port: u16,
    page_id: String,
    tx_task: Sender<Task>,
    rx: Receiver<SyncServerCommand>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        ws::connect(format!("ws://127.0.0.1:{}/$/ws/{}", ws_port, page_id), move |out| {
            // While we receive packets from the client, send them to sync.
            spawn_client_to_sync(out, rx.clone());

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
        }).unwrap();
        unreachable!("sync server socket disconnected.")
    })
}

fn setup_client(name: &str, page_id: &str, out: ws::Sender, ws_port: u16) -> (Arc<AtomicBool>, Arc<AtomicBool>, Sender<Task>) {
    let (tx_sync, rx) = unbounded();

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
        tx_sync,
    };

    // Send initial setup packet.
    client.setup();

    let (tx_task, rx_task) = unbounded();

    // Setup monkey tasks.
    setup_monkey::<ProxyClient>(alive.clone(), monkey.clone(), tx_task.clone());

    // Connect to the sync server.
    spawn_sync_connection(
        ws_port,
        page_id.to_owned(),
        tx_task.clone(),
        rx,
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
        let page_id = shake.request.resource()[1..].to_string();
        let (alive, monkey, tx_task) = setup_client("$$$$$$", &page_id, self.out.take().unwrap(), self.ws_port);
        self.alive = Some(alive);
        self.monkey = Some(monkey);
        self.tx_task = Some(tx_task);
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> Result<(), ws::Error> {
        // Handle messages received on this connection
        // println!("client command: '{}'. ", msg);

        let req_parse: Result<NativeCommand, _> = serde_json::from_slice(&msg.into_data());
        match req_parse {
            Err(err) => {
                println!("Packet error: {:?}", err);
            }
            Ok(value) => {
                self.tx_task.as_mut().map(|x| {
                    x.send(Task::NativeCommand(value)).expect("Could not handle native command.");
                });
            }
        }

        Ok(())
    }

    fn on_error(&mut self, _err: ws::Error) {
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
    server(&format!("0.0.0.0:{}", port), port - 1);
}
