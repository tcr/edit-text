//! An aggressively simple wrapper for `ws`.

#![allow(deprecated)]

use ws;
use failure::Error;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::{
    Arc,
    Mutex,
};
use ws::util::{
    Timeout,
    Token,
};
use ws::{
    CloseCode,
    Frame,
};

const PING_INTERVAL: u64 = 5_000;
const TIMEOUT_INTERVAL: u64 = 30_000;

static TOKEN_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub type Sender = Arc<Mutex<ws::Sender>>;

pub struct SocketHandler<S: SimpleSocket> {
    args: Option<S::Args>,

    out: Arc<Mutex<ws::Sender>>,
    obj: Option<S>,

    timeout: Option<Timeout>,
    ping_event: Token,
    expire_event: Token,
}

impl<S: SimpleSocket> SocketHandler<S> {
    pub fn new(args: S::Args, out: ws::Sender) -> SocketHandler<S> {
        SocketHandler {
            args: Some(args),

            out: Arc::new(Mutex::new(out)),
            obj: None,

            timeout: None,
            ping_event: Token(TOKEN_COUNTER.fetch_add(1, Ordering::SeqCst)),
            expire_event: Token(TOKEN_COUNTER.fetch_add(1, Ordering::SeqCst)),
        }
    }
}

pub trait SimpleSocket: Sized {
    type Args;
    fn initialize(args: Self::Args, url: &str, out: Arc<Mutex<ws::Sender>>) -> Result<Self, Error>;
    fn handle_message(&mut self, data: &[u8]) -> Result<(), Error>;
    fn cleanup(&mut self) -> Result<(), Error>;
}

impl<S: SimpleSocket> ws::Handler for SocketHandler<S> {
    fn on_open(&mut self, shake: ws::Handshake) -> Result<(), ws::Error> {
        self.obj = Some(
            S::initialize(
                self.args.take().unwrap(),
                shake.request.resource(),
                self.out.clone(),
            ).expect("Failed to start socket handler due to error"),
        );

        {
            let out = self.out.lock().unwrap();
            // schedule a timeout to send a ping every 5 seconds
            out.timeout(PING_INTERVAL, self.ping_event)?;
            // schedule a timeout to close the connection if there is no activity for 30 seconds
            out.timeout(TIMEOUT_INTERVAL, self.expire_event)?;
        }

        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> Result<(), ws::Error> {
        self.obj.as_mut().map(|obj| {
            obj.handle_message(&msg.into_data())
                .expect("Could not handle native command.");
        });

        Ok(())
    }

    fn on_error(&mut self, _err: ws::Error) {
        eprintln!("[ws] killing after error");
        self.obj
            .take()
            .map(|mut x| x.cleanup().expect("Failed to clean up socket"));
    }

    fn on_close(&mut self, _code: ws::CloseCode, _reason: &str) {
        println!("[ws] killing after close");
        self.obj
            .take()
            .map(|mut x| x.cleanup().expect("Failed to clean up socket"));
    }

    fn on_shutdown(&mut self) {
        eprintln!("[ws] killing after shutdown");
        self.obj
            .take()
            .map(|mut x| x.cleanup().expect("Failed to clean up socket"));
    }

    fn on_timeout(&mut self, event: Token) -> ws::Result<()> {
        if event == self.ping_event {
            let out = self.out.lock().unwrap();
            out.ping(vec![])?;
            out.timeout(PING_INTERVAL, self.ping_event)
        } else if event == self.expire_event {
            eprintln!("[ws] socket Expired {:?}", event);
            self.out.lock().unwrap().close(CloseCode::Away)
        } else {
            Ok(())
        }
    }

    fn on_new_timeout(&mut self, event: Token, timeout: Timeout) -> ws::Result<()> {
        if event == self.expire_event {
            if let Some(t) = self.timeout.take() {
                self.out.lock().unwrap().cancel(t)?;
            }
            self.timeout = Some(timeout)
        }
        Ok(())
    }

    fn on_frame(&mut self, frame: Frame) -> ws::Result<Option<Frame>> {
        // some activity has occurred, let's reset the expiration
        self.out
            .lock()
            .unwrap()
            .timeout(TIMEOUT_INTERVAL, self.expire_event)?;
        Ok(Some(frame))
    }
}
