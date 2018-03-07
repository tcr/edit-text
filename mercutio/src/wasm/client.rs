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

fn key_handlers() -> Vec<(u32, bool, bool, Box<Fn(&mut Client) -> Result<(), Error>>)> {
    vec![
        // backspace
        (
            8,
            false,
            false,
            Box::new(|client: &mut Client| client.client_op(|doc| delete_char(doc))),
        ),
        // left
        (
            37,
            false,
            false,
            Box::new(|client: &mut Client| client.client_op(|doc| caret_move(doc, false))),
        ),
        // right
        (
            39,
            false,
            false,
            Box::new(|client: &mut Client| client.client_op(|doc| caret_move(doc, true))),
        ),
        // up
        (
            38,
            false,
            false,
            Box::new(|client: &mut Client| client.client_op(|doc| caret_block_move(doc, false))),
        ),
        // down
        (
            40,
            false,
            false,
            Box::new(|client: &mut Client| client.client_op(|doc| caret_block_move(doc, true))),
        ),
        // enter
        (
            13,
            false,
            false,
            Box::new(|client: &mut Client| client.client_op(|doc| split_block(doc, false))),
        ),
        // enter
        (
            13,
            false,
            true,
            Box::new(|client: &mut Client| {
                client.client_op(|doc| add_char(doc, 10))
            }),
        ),
    ]
}

pub fn button_handlers() -> Vec<(&'static str, Box<Fn(&mut Client) -> Result<(), Error>>)> {
    vec![
        (
            "Heading 1",
            Box::new(|client: &mut Client| client.client_op(|doc| replace_block(doc, "h1"))),
        ),
        (
            "Heading 2",
            Box::new(|client: &mut Client| client.client_op(|doc| replace_block(doc, "h2"))),
        ),
        (
            "Heading 3",
            Box::new(|client: &mut Client| client.client_op(|doc| replace_block(doc, "h3"))),
        ),
        (
            "Paragraph",
            Box::new(|client: &mut Client| client.client_op(|doc| replace_block(doc, "p"))),
        ),
        (
            "Code",
            Box::new(|client: &mut Client| client.client_op(|doc| replace_block(doc, "pre"))),
        ),
        (
            "List",
            Box::new(|client: &mut Client| client.client_op(|doc| toggle_list(doc))),
        ),
        (
            "HR",
            Box::new(|client: &mut Client| client.client_op(|doc| split_block(doc, true))),
        ),
    ]
}




// TODO move the below to a random.rs ?

use oatie::writer::CurWriter;

pub struct RandomCursorContext {
    cur: CurWriter,
    history: Vec<CurSpan>,
}

impl Default for RandomCursorContext {
    fn default() -> Self {
        RandomCursorContext {
            cur: CurWriter::new(),
            history: vec![],
        }
    }
}

pub fn collect_cursors_span(ctx: &mut RandomCursorContext, span: &DocSpan) -> Result<(), Error> {
    for elem in span {
        match *elem {
            DocGroup(_, ref span) => {
                {
                    let mut c = ctx.cur.clone();
                    c.place(&CurElement::CurGroup);
                    c.exit_all();
                    ctx.history.push(c.result());
                }

                ctx.cur.begin();
                collect_cursors_span(ctx, span)?;
                ctx.cur.exit();
            }
            DocChars(ref text) => {
                ensure!(text.chars().count() > 0, "Empty char string");

                for _ in 0..text.chars().count() {
                    // Push a cursor to this character.
                    let mut c = ctx.cur.clone();
                    c.place(&CurElement::CurChar);
                    c.exit_all();
                    ctx.history.push(c.result());

                    // But also increment the base cursor to skip this char.
                    ctx.cur.place(&CurElement::CurSkip(1));
                }
            }
        }
    }
    Ok(())
}

pub fn collect_cursors(doc: &Doc) -> Result<Vec<CurSpan>, Error> {
    let mut ctx = RandomCursorContext::default();
    collect_cursors_span(&mut ctx, &doc.0)?;
    Ok(ctx.history)
}


fn native_command(client: &mut Client, req: NativeCommand) -> Result<(), Error> {
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
            let cursors = collect_cursors(&client.client_doc.doc)?;
            let idx = (pos * (cursors.len() as f64)) as usize;

            client.client_op(|doc| cur_to_caret(doc, &cursors[idx]))?;
        }
        NativeCommand::Target(cur) => {
            client.client_op(|doc| cur_to_caret(doc, &cur))?;
        }
        NativeCommand::Monkey(setting) => {
            println!("received monkey setting: {:?}", setting);
            client.monkey.store(setting, Ordering::Relaxed);
        }
        NativeCommand::RequestMarkdown => {
            let markdown = markdown::doc_to_markdown(&client.client_doc.doc.0)?;
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

    #[cfg(not(target_arch="wasm32"))]
    pub tx_client: Sender<ClientCommand>,
    #[cfg(not(target_arch="wasm32"))]
    pub tx_sync: Sender<SyncServerCommand>,
}

// use std::cell::RefCell;
// thread_local! {
//     static BAR: RefCell<Vec<i64>> = RefCell::new(vec![]);
// }

impl Client {
    // TODO this
    // pub fn new() -> (Client, Receiver, Receiver) {
    //     Client {
    //         client_id: "hello".to_string(),
    //         client_doc: ClientDoc::new(),

    //         monkey: Arc::new(AtomicBool::new(false)),
    //         alive: Arc::new(AtomicBool::new(true)),

    //         tx_client,
    //         tx_sync,
    //     };
    // }

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
        // let start = ::std::time::Instant::now();

        match value.clone() {
            // Handle commands from Native.
            Task::NativeCommand(command) => {
                if self.client_id == "$$$$$$" {
                    return Ok(());
                }

                native_command(self, command)?;
            }

            // Sync sent us an Update command with a new document version.
            Task::SyncClientCommand(
                SyncClientCommand::Init(new_client_id, doc_span, version)
            ) => {
                self.client_id = new_client_id.clone();
                self.client_doc.init(&Doc(doc_span), version);

                // Announce.
                println!("inital version is {:?}", version);

                log_wasm!(Setup(self.client_id.clone()));

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
                let res = ClientCommand::Update(doc_as_html(&self.client_doc.doc.0), None);
                self.send_client(&res).unwrap();
            }

            // Sync sent us an Update command with a new document version.
            Task::SyncClientCommand(
                SyncClientCommand::Update(doc_span, version, client_id, input_op)
            ) => {
                if self.client_id == "$$$$$$" {
                    return Ok(());
                }

                // TODO this can be generated from original_doc X input_op too
                let doc = Doc(doc_span);

                // If this operation is an acknowledgment...
                if self.client_id == client_id {
                    if let Some(local_op) = self.client_doc.sync_confirmed_pending_op(&doc, version) {
                        // Send our next operation.
                        self.upload(local_op)?;
                    }
                } else {
                    // Update with new version.
                    println!("---> sync sent new version");
                    self.client_doc.sync_sent_new_version(&doc, version, &input_op);
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
                let res = ClientCommand::Update(doc_as_html(&self.client_doc.doc.0), None);
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

        log_wasm!(Task(self.client_id.clone(), value.clone()));

        Ok(())
    }

    pub fn upload(&self, local_op: Op) -> Result<(), Error> {
        log_wasm!(Debug("CLIENTOP".to_string()));
        Ok(self.send_sync(SyncServerCommand::Commit(
            self.client_id.clone(),
            local_op,
            self.client_doc.version,
        ))?)
    }

    // TODO combine with client_op?
    fn with_action_context<C, T>(&mut self, callback: C) -> Result<T, Error>
    where
        C: Fn(ActionContext) -> Result<T, Error>,
    {
        callback(ActionContext {
            doc: self.client_doc.doc.clone(),
            client_id: self.client_id.clone(),
        })
    }

    fn client_op<C>(&mut self, callback: C) -> Result<(), Error>
    where
        C: Fn(ActionContext) -> Result<Op, Error>,
    {
        // Apply operation.
        let op = self.with_action_context(callback)?;

        // Apply new operation.
        self.client_doc.apply_local_op(&op);

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
        validate_doc(&self.client_doc.doc).expect("Local op was malformed");

        // Render the update.
        let res = ClientCommand::Update(doc_as_html(&self.client_doc.doc.0), Some(op));
        self.send_client(&res)?;

        // Send any queued payloads.
        if let Some(local_op) = self.client_doc.next_payload() {
            self.upload(local_op)?;
        }

        Ok(())
    }

    // server

    #[cfg(not(target_arch="wasm32"))]
    pub fn send_client(&self, req: &ClientCommand) -> Result<(), Error> {
        log_wasm!(SendClient(req.clone()));
        self.tx_client.send(req.clone())?;
        Ok(())
    }

    #[cfg(not(target_arch="wasm32"))]
    pub fn send_sync(&self, req: SyncServerCommand) -> Result<(), Error> {
        log_wasm!(SendSync(req.clone()));
        self.tx_sync.send(req)?;
        Ok(())
    }

    // wasm

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
