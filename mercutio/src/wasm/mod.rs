pub mod actions;
pub mod walkers;

#[cfg(not(target_arch="wasm32"))]
pub mod proxy;

use self::actions::*;
use failure::Error;
use oatie::{Operation, OT};
use oatie::doc::*;
use oatie::schema::RtfSchema;
use rand;
use rand::Rng;
use serde_json;
use std::{panic, process};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use super::*;

#[cfg(not(target_arch="wasm32"))]
use super::{SyncClientCommand, SyncServerCommand};
#[cfg(not(target_arch="wasm32"))]
use crossbeam_channel::{unbounded, Sender};
#[cfg(not(target_arch="wasm32"))]
use ws;
#[macro_use]
use lazy_static;

#[cfg(not(target_arch="wasm32"))]
use self::proxy::*;

macro_rules! clone_all {
    ( $( $x:ident ),* ) => {
        $(let $x = $x.clone();)*
    };
}

// Commands to send back to native.
#[derive(Serialize, Deserialize, Debug)]
pub enum NativeCommand {
    // Connect(String),
    Keypress(u32, bool, bool),
    Button(u32),
    Character(u32),
    RenameGroup(String, CurSpan),
    // Load(DocSpan),
    Target(CurSpan),
    Monkey(bool),
}

// Commands to send to JavaScript.
#[derive(Serialize, Deserialize, Debug)]
pub enum ClientCommand {
    Setup {
        keys: Vec<(u32, bool, bool)>,
        buttons: Vec<(usize, String)>,
    },
    PromptString(String, String, NativeCommand),
    Update(String, Option<Op>),
    Error(String),
    SyncServerCommand(SyncServerCommand),
}

fn doc_as_html(doc: &DocSpan) -> String {
    let mut out = String::new();
    for elem in doc {
        match elem {
            &DocGroup(ref attrs, ref span) => {
                out.push_str(&format!(
                    r#"<div
                        data-tag={}
                        data-client={}
                        class={}
                    >"#,
                    serde_json::to_string(attrs.get("tag").unwrap_or(&"".to_string())).unwrap(),
                    serde_json::to_string(attrs.get("client").unwrap_or(&"".to_string())).unwrap(),
                    serde_json::to_string(attrs.get("class").unwrap_or(&"".to_string())).unwrap(),
                ));
                out.push_str(&doc_as_html(span));
                out.push_str(r"</div>");
            }
            &DocChars(ref text) => for c in text.chars() {
                out.push_str(r"<span>");
                out.push(c);
                out.push_str(r"</span>");
            },
        }
    }
    out
}

// TODO combine with client_op?
fn with_action_context<C, T>(client: &mut Client, callback: C) -> Result<T, Error>
where
    C: Fn(ActionContext) -> Result<T, Error>,
{
    callback(ActionContext {
        doc: client.doc.clone(),
        client_id: client.name.clone(),
    })
}

fn synchronize_op(client: &mut Client) {
    if client.op_outstanding.is_none() && client.ops.len() > 0 {
        // Compose all ops.
        let ops = client.ops.split_off(0);
        let op = Operation::compose_iter(ops.iter());
        client.op_outstanding = Some(op.clone());

        // Send operation to sync server.
        client.send_sync(SyncServerCommand::Commit(
            client.name.clone(),
            op,
            client.version,
        ));
    }
}

fn client_op<C>(client: &mut Client, callback: C) -> Result<(), Error>
where
    C: Fn(ActionContext) -> Result<Op, Error>,
{
    let op = callback(ActionContext {
        doc: client.doc.clone(),
        client_id: client.name.clone(),
    })?;

    // Apply new operation.
    let new_doc = OT::apply(&client.doc, &op);

    client.ops.push(op.clone());
    client.doc = new_doc;

    // Check that our operations can compose well.
    // if cfg!(not(target_arch = "wasm32")) {
    //     // println!("ORIGINAL: {:?}", client.original_doc);
    //     let mut check_op_a = client.op_outstanding.clone().unwrap_or(op_span!([], []));
    //     for (i, op) in client.ops.iter().enumerate() {
    //         // println!("  {}: applying {:?}/{:?}", name, i + 1, client.ops.len());
    //         // println!("  {} 1️⃣: let op_left = op_span!{:?};", name, check_op_a);
    //         // println!("  {} 1️⃣: let op_right = op_span!{:?};", name, op);
    //         check_op_a = Operation::compose(&check_op_a, &op);
    //         // println!("  {} 1️⃣: let res = op_span!{:?};", name, check_op_a);
    //         // println!("  {} 1️⃣: let original = doc_span!{:?};", name, client.original_doc);
    //         // println!("  {} 1️⃣: let latest_doc = doc_span!{:?};", name, client.doc);
    //         let _ = OT::apply(&client.original_doc, &check_op_a);
    //     }
    
    //     assert_eq!(OT::apply(&client.original_doc, &check_op_a), client.doc);
    // }

    // Send update.
    let res = ClientCommand::Update(doc_as_html(&client.doc.0), Some(op));
    client.send_client(&res)?;

    synchronize_op(client);

    Ok(())
}

fn key_handlers() -> Vec<(u32, bool, bool, Box<Fn(&mut Client) -> Result<(), Error>>)> {
    vec![
        // backspace
        (
            8,
            false,
            false,
            Box::new(|client: &mut Client| client_op(client, |doc| delete_char(doc))),
        ),
        // left
        (
            37,
            false,
            false,
            Box::new(|client: &mut Client| client_op(client, |doc| caret_move(doc, false))),
        ),
        // right
        (
            39,
            false,
            false,
            Box::new(|client: &mut Client| client_op(client, |doc| caret_move(doc, true))),
        ),
        // up
        (
            38,
            false,
            false,
            Box::new(|client: &mut Client| client_op(client, |doc| caret_block_move(doc, false))),
        ),
        // down
        (
            40,
            false,
            false,
            Box::new(|client: &mut Client| client_op(client, |doc| caret_block_move(doc, true))),
        ),
        // enter
        (
            13,
            false,
            false,
            Box::new(|client: &mut Client| client_op(client, |doc| split_block(doc))),
        ),
    ]
}

fn button_handlers() -> Vec<(&'static str, Box<Fn(&mut Client) -> Result<(), Error>>)> {
    vec![
        (
            "Heading 1",
            Box::new(|client: &mut Client| client_op(client, |doc| replace_block(doc, "h1"))),
        ),
        (
            "Heading 2",
            Box::new(|client: &mut Client| client_op(client, |doc| replace_block(doc, "h2"))),
        ),
        (
            "Heading 3",
            Box::new(|client: &mut Client| client_op(client, |doc| replace_block(doc, "h3"))),
        ),
        (
            "Paragraph",
            Box::new(|client: &mut Client| client_op(client, |doc| replace_block(doc, "p"))),
        ),
        (
            "Code",
            Box::new(|client: &mut Client| client_op(client, |doc| replace_block(doc, "pre"))),
        ),
        (
            "List",
            Box::new(|client: &mut Client| client_op(client, |doc| toggle_list(doc))),
        ),
    ]
}

fn native_command(client: &mut Client, req: NativeCommand) -> Result<(), Error> {
    match req {
        NativeCommand::RenameGroup(tag, _) => client_op(client, |doc| replace_block(doc, &tag))?,
        NativeCommand::Button(index) => {
            // Find which button handler to respond to this command.
            button_handlers()
                .get(index as usize)
                .map(|handler| handler.1(client));
        }
        NativeCommand::Keypress(key_code, meta_key, shift_key) => {
            println!("key: {:?} {:?} {:?}", key_code, meta_key, shift_key);

            // Find which key handler to process this command.
            for command in key_handlers() {
                if command.0 == key_code && command.1 == meta_key && command.2 == shift_key {
                    command.3(client)?;
                    break;
                }
            }
        }
        NativeCommand::Character(char_code) => client_op(client, |doc| add_char(doc, char_code))?,
        NativeCommand::Target(cur) => {
            client_op(client, |doc| cur_to_caret(doc, &cur))?;
        }
        NativeCommand::Monkey(setting) => {
            client.monkey.store(setting, Ordering::Relaxed);
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub enum Task {
    ButtonMonkey,
    LetterMonkey,
    ArrowMonkey,
    BackspaceMonkey,
    EnterMonkey,
    SyncClientCommand(SyncClientCommand),
    NativeCommand(NativeCommand),
}

pub struct Client {
    pub name: String,

    pub doc: Doc,
    pub version: usize,
    pub ops: Vec<Op>,
    pub op_outstanding: Option<Op>,

    pub original_doc: Doc,

    pub monkey: Arc<AtomicBool>,
    pub alive: Arc<AtomicBool>,

    #[cfg(not(target_arch="wasm32"))]
    pub out: ws::Sender,
    #[cfg(not(target_arch="wasm32"))]
    pub tx: Sender<SyncServerCommand>,
}

impl Client {
    pub fn setup(&self) {
        self
            .send_client(&ClientCommand::Setup {
                keys: key_handlers()
                    .into_iter()
                    .map(|x| (x.0, x.1, x.2))
                    .collect(),
                buttons: button_handlers()
                    .into_iter()
                    .enumerate()
                    .map(|(i, x)| (i, x.0.to_string()))
                    .collect(),
            })
            .expect("Could not send initial state");
    }

    pub fn handle_task(&mut self, value: Task) -> Result<(), Error> {
        match value {
            Task::ButtonMonkey => {
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0, button_handlers().len() as u32);
                let command = NativeCommand::Button(index);
                native_command(self, command)?;
            }
            Task::LetterMonkey => {
                let mut rng = rand::thread_rng();
                let char_list = vec![
                    rng.gen_range(b'A', b'Z'),
                    rng.gen_range(b'a', b'z'),
                    rng.gen_range(b'0', b'9'),
                    b' ',
                ];
                let c = *rng.choose(&char_list).unwrap() as u32;
                let command = NativeCommand::Character(c);
                native_command(self, command)?;
            }
            Task::ArrowMonkey => {
                let mut rng = rand::thread_rng();
                let key = *rng.choose(&[37, 39, 37, 39, 37, 39, 38, 40]).unwrap();
                let command = NativeCommand::Keypress(key, false, false);
                native_command(self, command)?;
            }
            Task::BackspaceMonkey => {
                let command = NativeCommand::Keypress(8, false, false);
                native_command(self, command)?;
            }
            Task::EnterMonkey => {
                let command = NativeCommand::Keypress(13, false, false);
                native_command(self, command)?;
            }

            // Handle commands from Native.
            Task::NativeCommand(command) => {
                native_command(self, command)?;
            }

            // Sync sent us an Update command with a new document version.
            Task::SyncClientCommand(SyncClientCommand::Update(doc_span, version, client_id, input_op)) => {
                // TODO this can be generated from original_doc X input_op too
                let doc = Doc(doc_span);
                self.original_doc = doc.clone();

                // Set the document.
                if self.name == client_id {
                    // Reattach to client.
                    self.doc = OT::apply(&doc, &Operation::compose_iter(self.ops.iter()));
                } else if !self.ops.is_empty() {
                    println!("\n----> TRANSFORMING");

                    let ops = self.ops.split_off(0);

                    // Cumulative client operation.
                    let pending_op = self.op_outstanding.clone().unwrap();
                    let local_op = Operation::compose_iter(ops.iter());

                    // Transform.
                    println!();
                    println!("<test>");
                    println!("server: {:?}", input_op);
                    println!();
                    println!("pending: {:?}", pending_op);
                    println!("client: {:?}", local_op);
                    println!("</test>");
                    println!();
                    let (prending_op_transform, _) = Operation::transform::<RtfSchema>(&input_op, &pending_op);
                    let (local_op_transform, _) = Operation::transform::<RtfSchema>(&input_op, &local_op);
                    let client_op = Operation::compose(&prending_op_transform, &local_op_transform);

                    // Reattach to client.
                    self.doc = OT::apply(&doc, &client_op);

                    self.op_outstanding = Some(prending_op_transform);
                    self.ops.push(local_op_transform);
                } else {
                    self.doc = doc;
                }

                self.version = version;
                println!("new version is {:?}", version);

                // If we get an ack, we can start pushing a new op.
                if self.name == client_id {
                    self.op_outstanding = None;
                    synchronize_op(self);
                }

                // If the caret doesn't exist or was deleted, reinitialize it.
                if !with_action_context(self, |ctx| Ok(has_caret(ctx)))
                    .ok()
                    .unwrap_or(true)
                {
                    client_op(self, |doc| init_caret(doc)).unwrap();
                }

                // Native drives client state.
                let res = ClientCommand::Update(doc_as_html(&self.doc.0), None);
                self.send_client(&res).unwrap();
            }
        }
        Ok(())
    }

    #[cfg(not(target_arch="wasm32"))]
    pub fn send_client(&self, req: &ClientCommand) -> Result<(), Error> {
        let json = serde_json::to_string(&req)?;
        self.out.send(json)?;
        Ok(())
    }

    #[cfg(not(target_arch="wasm32"))]
    pub fn send_sync(&self, req: SyncServerCommand) -> Result<(), Error> {
        self.tx.send(req)?;
        Ok(())
    }

    #[cfg(target_arch="wasm32")]
    pub fn send_client(&self, req: &ClientCommand) -> Result<(), Error> {
        use std::mem;
        use std::ffi::CString;
        use std::os::raw::{c_char, c_void};

        extern "C" {
            /// Send a command *to* the js client.
            pub fn js_command(input_ptr: *mut c_char) -> u32;
        }

        let data = serde_json::to_string(&req)?;
        let s = CString::new(data).unwrap().into_raw();

        unsafe {
            let _ = js_command(s);
        }

        Ok(())
    }

    #[cfg(target_arch="wasm32")]
    pub fn send_sync(&self, req: SyncServerCommand) -> Result<(), Error> {
        self.send_client(&ClientCommand::SyncServerCommand(req))
    }

}



