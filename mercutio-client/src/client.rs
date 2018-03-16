use super::*;
use failure::Error;
use oatie::doc::*;
use std::sync::atomic::{AtomicBool};
use std::sync::atomic::Ordering;
use oatie::validate::validate_doc;
use crate::markdown;

#[cfg(not(target_arch="wasm32"))]
use super::{SyncClientCommand, SyncServerCommand};
#[cfg(not(target_arch="wasm32"))]
use crossbeam_channel::Sender;

// Commands to send back to native.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum NativeCommand {
    // Connect(String),
    Keypress(u32, bool, bool),
    Button(u32),
    Character(u32),
    RenameGroup(String, CurSpan),
    // Load(DocSpan),
    Target(CurSpan),
    RandomTarget(f64),
    Monkey(bool),
    RequestMarkdown,
}

// Commands to send to JavaScript.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ClientCommand {
    Init(String),
    Setup {
        keys: Vec<(u32, bool, bool)>,
        buttons: Vec<(usize, String)>,
    },
    PromptString(String, String, NativeCommand),
    Update(String, Option<Op>),
    MarkdownUpdate(String),
    Error(String),
    SyncServerCommand(SyncServerCommand),
}

fn key_handlers<C: ClientImpl>()
    -> Vec<(u32, bool, bool, Box<Fn(&mut C) -> Result<(), Error>>)> {
    vec![
        // backspace
        (
            8,
            false,
            false,
            Box::new(|client| client.client_op(|doc| delete_char(doc))),
        ),
        // left
        (
            37,
            false,
            false,
            Box::new(|client| client.client_op(|doc| caret_move(doc, false))),
        ),
        // right
        (
            39,
            false,
            false,
            Box::new(|client| client.client_op(|doc| caret_move(doc, true))),
        ),
        // up
        (
            38,
            false,
            false,
            Box::new(|client| client.client_op(|doc| caret_block_move(doc, false))),
        ),
        // down
        (
            40,
            false,
            false,
            Box::new(|client| client.client_op(|doc| caret_block_move(doc, true))),
        ),
        // enter
        (
            13,
            false,
            false,
            Box::new(|client| client.client_op(|doc| split_block(doc, false))),
        ),
        // enter
        (
            13,
            false,
            true,
            Box::new(|client| client.client_op(|doc| add_char(doc, 10))),
        ),
    ]
}

pub fn button_handlers<C: ClientImpl>() ->
    Vec<(&'static str, Box<Fn(&mut C) -> Result<(), Error>>)> {
    vec![
        (
            "H1",
            Box::new(|client| client.client_op(|doc| replace_block(doc, "h1"))),
        ),
        (
            "H2",
            Box::new(|client| client.client_op(|doc| replace_block(doc, "h2"))),
        ),
        (
            "H3",
            Box::new(|client| client.client_op(|doc| replace_block(doc, "h3"))),
        ),
        (
            "Paragraph",
            Box::new(|client| client.client_op(|doc| replace_block(doc, "p"))),
        ),
        (
            "Code",
            Box::new(|client| client.client_op(|doc| replace_block(doc, "pre"))),
        ),
        (
            "List",
            Box::new(|client| client.client_op(|doc| toggle_list(doc))),
        ),
        (
            "HR",
            Box::new(|client| client.client_op(|doc| split_block(doc, true))),
        ),
        (
            "Raw HTML",
            Box::new(|client| client.client_op(|doc| replace_block(doc, "html"))),
        ),
    ]
}


fn native_command<C: ClientImpl>(client: &mut C, req: NativeCommand) -> Result<(), Error> {
    match req {
        NativeCommand::RenameGroup(tag, _) => {
            client.client_op(|doc| replace_block(doc, &tag))?;
        }
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
        NativeCommand::Character(char_code) => {
            client.client_op(|doc| add_char(doc, char_code))?;
        }
        NativeCommand::RandomTarget(pos) => {
            let cursors = random_cursor(&client.state().client_doc.doc)?;
            let idx = (pos * (cursors.len() as f64)) as usize;

            client.client_op(|doc| cur_to_caret(doc, &cursors[idx]))?;
        }
        NativeCommand::Target(cur) => {
            client.client_op(|doc| cur_to_caret(doc, &cur))?;
        }
        NativeCommand::Monkey(setting) => {
            println!("received monkey setting: {:?}", setting);
            client.state().monkey.store(setting, Ordering::Relaxed);
        }
        NativeCommand::RequestMarkdown => {
            let markdown = markdown::doc_to_markdown(&client.state().client_doc.doc.0)?;
            // TODO
            client.send_client(&ClientCommand::MarkdownUpdate(markdown))?;
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Task {
    SyncClientCommand(SyncClientCommand),
    NativeCommand(NativeCommand),
}

pub struct Client {
    pub client_id: String,
    pub client_doc: ClientDoc,

    pub monkey: Arc<AtomicBool>,
    pub alive: Arc<AtomicBool>,
}

// use std::cell::RefCell;
// thread_local! {
//     static BAR: RefCell<Vec<i64>> = RefCell::new(vec![]);
// }

pub trait ClientImpl {
    fn state(&mut self) -> &mut Client;
    fn send_client(&self, req: &ClientCommand) -> Result<(), Error>;
    fn send_sync(&self, req: SyncServerCommand) -> Result<(), Error>;

    fn setup(&self) where Self: Sized {
        self
            .send_client(&ClientCommand::Setup {
                keys: key_handlers::<Self>()
                    .into_iter()
                    .map(|x| (x.0, x.1, x.2))
                    .collect(),
                buttons: button_handlers::<Self>()
                    .into_iter()
                    .enumerate()
                    .map(|(i, x)| (i, x.0.to_string()))
                    .collect(),
            })
            .expect("Could not send initial state");
    }

    fn handle_task(&mut self, value: Task) -> Result<(), Error>
        where Self: Sized {
        // let start = ::std::time::Instant::now();

        match value.clone() {
            // Handle commands from Native.
            Task::NativeCommand(command) => {
                if self.state().client_id == "$$$$$$" {
                    println!("NATIVE COMMAND TOO EARLY");
                    return Ok(());
                }

                native_command(self, command)?;
            }

            // Sync sent us an Update command with a new document version.
            Task::SyncClientCommand(
                SyncClientCommand::Init(new_client_id, doc_span, version)
            ) => {
                self.state().client_id = new_client_id.clone();
                self.state().client_doc.init(&Doc(doc_span), version);

                // Announce.
                println!("inital version is {:?}", version);

                log_wasm!(Setup(self.state().client_id.clone()));

                // If the caret doesn't exist or was deleted, reinitialize it.
                if !self.with_action_context(|ctx| Ok(has_caret(ctx)))
                    .ok()
                    .unwrap_or(true)
                {
                    println!("add caret");
                    self.client_op(|doc| init_caret(doc)).unwrap();
                }
                
                let res = ClientCommand::Init(new_client_id);
                self.send_client(&res).unwrap();

                // Native drives client state.
                let res = ClientCommand::Update(doc_as_html(&self.state().client_doc.doc.0), None);
                self.send_client(&res).unwrap();
            }

            // Sync sent us an Update command with a new document version.
            Task::SyncClientCommand(
                SyncClientCommand::Update(doc_span, version, client_id, input_op)
            ) => {
                if self.state().client_id == "$$$$$$" {
                    return Ok(());
                }

                // TODO this can be generated from original_doc X input_op too
                let doc = Doc(doc_span);

                // If this operation is an acknowledgment...
                if self.state().client_id == client_id {
                    if let Some(local_op) = self.state().client_doc.sync_confirmed_pending_op(&doc, version) {
                        // Send our next operation.
                        self.upload(local_op)?;
                    }
                } else {
                    // Update with new version.
                    println!("---> sync sent new version");
                    self.state().client_doc.sync_sent_new_version(&doc, version, &input_op);
                }

                // Announce.
                println!("new version is {:?}", version);

                // If the caret doesn't exist or was deleted, reinitialize it.
                if !self.with_action_context(|ctx| Ok(has_caret(ctx)))
                    .ok()
                    .unwrap_or(true)
                {
                    println!("add caret");
                    self.client_op(|doc| init_caret(doc)).unwrap();
                }

                // Native drives client state.
                let res = ClientCommand::Update(doc_as_html(&self.state().client_doc.doc.0), None);
                self.send_client(&res).unwrap();
            }
        }

        // fn average(numbers: &[i64]) -> f32 {
        //     numbers.iter().sum::<i64>() as f32 / numbers.len() as f32
        // }

        // BAR.with(|bar| {
        //     let mut b = bar.borrow_mut();
        
        //     b.push(start.elapsed().num_milliseconds());

        //     println!("{} ms per task.", average(b.as_slice()));
        // });

        log_wasm!(Task(self.state().client_id.clone(), value.clone()));

        Ok(())
    }

    fn upload(&mut self, local_op: Op) -> Result<(), Error> {
        log_wasm!(Debug("CLIENTOP".to_string()));
        let client_id = self.state().client_id.clone();
        let version = self.state().client_doc.version;
        Ok(self.send_sync(SyncServerCommand::Commit(
            client_id,
            local_op,
            version,
        ))?)
    }

    // TODO combine with client_op?
    fn with_action_context<C, T>(&mut self, callback: C) -> Result<T, Error>
    where
        C: Fn(ActionContext) -> Result<T, Error>,
    {
        callback(ActionContext {
            doc: self.state().client_doc.doc.clone(),
            client_id: self.state().client_id.clone(),
        })
    }

    fn client_op<C>(&mut self, callback: C) -> Result<(), Error>
    where
        C: Fn(ActionContext) -> Result<Op, Error>,
    {
        // Apply operation.
        let op = self.with_action_context(callback)?;

        // Apply new operation.
        self.state().client_doc.apply_local_op(&op);

        // Check that our operations can compose well.
        // if cfg!(not(target_arch = "wasm32")) {
        //     // println!("ORIGINAL: {:?}", client.original_doc);
        //     let mut check_op_a = client.op_outstanding.clone().unwrap_or(op_span!([], []));
        //     for (i, op) in client.ops.iter().enumerate() {
        //         // println!("  {}: applying {:?}/{:?}", name, i + 1, client.ops.len());
        //         // println!("  {} 1️⃣: let op_left = op_span!{:?};", name, check_op_a);
        //         // println!("  {} 1️⃣: let op_right = op_span!{:?};", name, op);
        //         check_op_a = OT::compose(&check_op_a, &op);
        //         // println!("  {} 1️⃣: let res = op_span!{:?};", name, check_op_a);
        //         // println!("  {} 1️⃣: let original = doc_span!{:?};", name, client.original_doc);
        //         // println!("  {} 1️⃣: let latest_doc = doc_span!{:?};", name, client.doc);
        //         let _ = OT::apply(&client.original_doc, &check_op_a);
        //     }
        
        //     assert_eq!(OT::apply(&client.original_doc, &check_op_a), client.doc);
        // }

        // Validate local changes.
        validate_doc(&self.state().client_doc.doc).expect("Local op was malformed");

        // Render the update.
        let res = ClientCommand::Update(doc_as_html(&self.state().client_doc.doc.0), Some(op));
        self.send_client(&res)?;

        // Send any queued payloads.
        if let Some(local_op) = self.state().client_doc.next_payload() {
            self.upload(local_op)?;
        }

        Ok(())
    }
}

#[cfg(not(target_arch="wasm32"))]
pub struct ProxyClient {
    pub state: Client,
    pub tx_client: Sender<ClientCommand>,
    pub tx_sync: Sender<SyncServerCommand>,
}

#[cfg(not(target_arch="wasm32"))]
impl ClientImpl for ProxyClient {
    fn state(&mut self) -> &mut Client {
        &mut self.state
    }

    fn send_client(&self, req: &ClientCommand) -> Result<(), Error> {
        log_wasm!(SendClient(req.clone()));
        self.tx_client.send(req.clone())?;
        Ok(())
    }

    fn send_sync(&self, req: SyncServerCommand) -> Result<(), Error> {
        log_wasm!(SendSync(req.clone()));
        self.tx_sync.send(req)?;
        Ok(())
    }
}
