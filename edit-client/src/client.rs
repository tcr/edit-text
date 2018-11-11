use crate::{
    actions::*,
    random::*,
    state::*,
};

use edit_common::{
    commands::*,
    doc_as_html,
    markdown::doc_to_markdown,
};
use failure::Error;
use oatie::{
    doc::*,
    validate::validate_doc,
    OT,
};
use std::{
    char::from_u32,
    sync::atomic::{
        AtomicBool,
        Ordering,
    },
    sync::Arc,
};

// Shorthandler
// code, meta, shift, alt, callback
struct KeyHandler<C: ClientImpl>(u32, bool, bool, bool, Box<Fn(&mut C) -> Result<(), Error>>);

impl<C: ClientImpl> KeyHandler<C> {
    fn matches(&self, code: u32, meta_key: bool, shift_key: bool, alt_key: bool) -> bool {
        self.0 == code && self.1 == meta_key && self.2 == shift_key && self.3 == alt_key
    }

    fn invoke(&self, client: &mut C) -> Result<(), Error> {
        self.4(client)
    }
}

fn key_handlers<C: ClientImpl>() -> Vec<KeyHandler<C>> {
    vec![
        // backspace
        KeyHandler(
            8,
            false,
            false,
            false,
            Box::new(|client| client.client_op(|doc| delete_char(doc))),
        ),
        // left
        KeyHandler(
            37,
            false,
            false,
            false,
            Box::new(|client| client.client_op(|doc| caret_move(doc, false, false))),
        ),
        // right
        KeyHandler(
            39,
            false,
            false,
            false,
            Box::new(|client| client.client_op(|doc| caret_move(doc, true, false))),
        ),
        // shift + left
        KeyHandler(
            37,
            false,
            true,
            false,
            Box::new(|client| client.client_op(|doc| caret_move(doc, false, true))),
        ),
        // shift + right
        KeyHandler(
            39,
            false,
            true,
            false,
            Box::new(|client| client.client_op(|doc| caret_move(doc, true, true))),
        ),
        // up
        KeyHandler(
            38,
            false,
            false,
            false,
            Box::new(|client| client.client_op(|doc| caret_block_move(doc, false))),
        ),
        // down
        KeyHandler(
            40,
            false,
            false,
            false,
            Box::new(|client| client.client_op(|doc| caret_block_move(doc, true))),
        ),
        // enter
        KeyHandler(
            13,
            false,
            false,
            false,
            Box::new(|client| client.client_op(|doc| split_block(doc, false))),
        ),
        // enter
        KeyHandler(
            13,
            false,
            true,
            false,
            Box::new(|client| client.client_op(|doc| add_string(doc, "\n"))),
        ),
        // tab
        KeyHandler(
            9,
            false,
            false,
            false,
            Box::new(|client| client.client_op(|doc| toggle_list(doc))),
        ),
        // OPT-left
        KeyHandler(
            37,
            false,
            false,
            true,
            Box::new(|client| client.client_op(|doc| caret_word_move(doc, false))),
        ),
        // OPT-left
        KeyHandler(
            39,
            false,
            false,
            true,
            Box::new(|client| client.client_op(|doc| caret_word_move(doc, true))),
        ),
        // OPT-a
        KeyHandler(
            65,
            true,
            false,
            false,
            Box::new(|client| client.client_op(|doc| caret_select_all(doc))),
        ),
    ]
}

pub fn button_handlers<C: ClientImpl>(
    state: Option<(String, bool)>,
) -> (Vec<Box<Fn(&mut C) -> Result<(), Error>>>, Vec<Ui>) {
    let mut callbacks: Vec<Box<Fn(&mut C) -> Result<(), Error>>> = vec![];

    macro_rules! callback {
        ($t:expr) => {{
            callbacks.push(Box::new($t));
            callbacks.len() - 1
        }};
    }

    let ui = vec![
        Ui::ButtonGroup(vec![
            Ui::Button(
                "Text".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, "p"))),
                state.as_ref().map(|x| x.0 == "p").unwrap_or(false),
            ),
            Ui::Button(
                "H1".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, "h1"))),
                // TODO i wish we could match on strings, use matches! here
                state.as_ref().map(|x| x.0 == "h1").unwrap_or(false),
            ),
            Ui::Button(
                "H2".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, "h2"))),
                state.as_ref().map(|x| x.0 == "h2").unwrap_or(false),
            ),
            Ui::Button(
                "H3".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, "h3"))),
                state.as_ref().map(|x| x.0 == "h3").unwrap_or(false),
            ),
            Ui::Button(
                "H4".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, "h4"))),
                state.as_ref().map(|x| x.0 == "h4").unwrap_or(false),
            ),
            Ui::Button(
                "H5".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, "h5"))),
                state.as_ref().map(|x| x.0 == "h5").unwrap_or(false),
            ),
            Ui::Button(
                "H6".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, "h6"))),
                state.as_ref().map(|x| x.0 == "h6").unwrap_or(false),
            ),
            Ui::Button(
                "Code".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, "pre"))),
                state.as_ref().map(|x| x.0 == "pre").unwrap_or(false),
            ),
            Ui::Button(
                "HTML".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, "html"))),
                state.as_ref().map(|x| x.0 == "html").unwrap_or(false),
            ),
        ]),
        Ui::Button(
            "List".to_string(),
            callback!(|client| client.client_op(|doc| toggle_list(doc))),
            state.as_ref().map(|x| x.1).unwrap_or(false),
        ),
        Ui::Button(
            "HR".to_string(),
            callback!(|client| client.client_op(|doc| split_block(doc, true))),
            false,
        ),
        Ui::ButtonGroup(vec![
            Ui::Button(
                "Bold".to_string(),
                callback!(|client| client.client_op(|doc| apply_style(doc, Style::Bold, None))),
                // state.as_ref().map(|x| x.0 == "html").unwrap_or(false),
                false, // TODO what?
            ),
            Ui::Button(
                "Italic".to_string(),
                callback!(|client| client.client_op(|doc| apply_style(doc, Style::Italic, None))),
                // state.as_ref().map(|x| x.0 == "html").unwrap_or(false),
                false, // TODO what?
            ),
            Ui::Button(
                "Clear".to_string(),
                callback!(|client| client.client_op(|doc| remove_styles(
                    doc,
                    hashset![Style::Bold, Style::Italic, Style::Link]
                ))),
                // state.as_ref().map(|x| x.0 == "html").unwrap_or(false),
                false, // TODO what?
            ),
        ]),
    ];

    (callbacks, ui)
}

fn native_command<C: ClientImpl>(client: &mut C, req: ControllerCommand) -> Result<(), Error> {
    match req {
        ControllerCommand::RenameGroup(tag, _) => {
            client.client_op(|doc| replace_block(doc, &tag))?;
        }
        ControllerCommand::Button(index) => {
            // Find which button handler to respond to this command.
            button_handlers(None)
                .0
                .get(index as usize)
                .map(|handler| handler(client));
        }
        ControllerCommand::Keypress(key_code, meta_key, shift_key, alt_key) => {
            println!(
                "key: {:?} {:?} {:?} {:?}",
                key_code, meta_key, shift_key, alt_key
            );

            // Find which key handler to process this command.
            for command in key_handlers() {
                if command.matches(key_code, meta_key, shift_key, alt_key) {
                    command.invoke(client)?;
                    break;
                }
            }
        }
        ControllerCommand::Character(char_code) => {
            client.client_op(|doc| {
                let c: char = from_u32(char_code).unwrap_or('?');
                if c == '\0' {
                    bail!("expected non-null character");
                }

                add_string(doc, &format!("{}", c))
            })?;
        }
        ControllerCommand::InsertText(text) => {
            client.client_op(|doc| add_string(doc, &text))?;
        }
        ControllerCommand::RandomTarget(pos) => {
            // TODO this should never happen, because we clarify RandomTarget
            // beforehand

            let cursors = random_cursor(&client.state().client_doc.doc)?;
            let idx = (pos * (cursors.len() as f64)) as usize;

            client.client_op(|doc| cur_to_caret(doc, &cursors[idx], true))?;
        }
        ControllerCommand::Cursor(focus, anchor) => {
            match (focus, anchor) {
                (Some(focus), Some(anchor)) => {
                    client.client_op(|mut ctx| {
                        let op = cur_to_caret(ctx.clone(), &focus, true)?;
                        ctx.doc = Op::apply(&ctx.doc, &op);
                        let op2 = cur_to_caret(ctx, &anchor, false)?;
                        Ok(Op::compose(&op, &op2))
                    })?;
                }
                (Some(focus), None) => {
                    client.client_op(|doc| cur_to_caret(doc, &focus, true))?;
                }
                (None, Some(anchor)) => {
                    client.client_op(|doc| cur_to_caret(doc, &anchor, false))?;
                }
                (None, None) => {} // ???
            }
        }
        ControllerCommand::Monkey(setting) => {
            println!("received monkey setting: {:?}", setting);
            client.state().monkey.store(setting, Ordering::Relaxed);
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Task {
    ClientCommand(ClientCommand),
    ControllerCommand(ControllerCommand),
}

pub struct Client {
    pub client_id: String,
    pub client_doc: ClientDoc,
    pub last_controls: Option<Controls>,

    pub monkey: Arc<AtomicBool>,
    pub alive: Arc<AtomicBool>,
    pub task_count: usize,
}

/// Trait shared by the "wasm" and "client proxy" implementations.
/// Most methods are implemented on this trait, not its implementors.
pub trait ClientImpl {
    fn state(&mut self) -> &mut Client;
    fn send_client(&self, req: &FrontendCommand) -> Result<(), Error>;
    fn send_sync(&self, req: ServerCommand) -> Result<(), Error>;

    fn setup_controls(&mut self, state: Option<(String, bool)>)
    where
        Self: Sized,
    {
        let controls_object = Controls {
            keys: key_handlers::<Self>()
                .into_iter()
                .map(|x| (x.0, x.1, x.2))
                .collect(),
            buttons: button_handlers::<Self>(state).1,
        };

        if Some(controls_object.clone()) != self.state().last_controls {
            self.send_client(&FrontendCommand::Controls(controls_object.clone()))
                .expect("Could not send initial state");

            self.state().last_controls = Some(controls_object);
        };
    }

    // TODO can we catch_unwind inside handle task so we can add our own
    // "TASK: data" dump into the error payload? So then it's easy to
    // corrolate with the logs.
    fn handle_task(&mut self, mut value: Task) -> Result<(), Error>
    where
        Self: Sized,
    {
        // let start = ::std::time::Instant::now();

        self.state().task_count += 1;
        let task_count = self.state().task_count;
        eprintln!("TASK ~~~~ {} ~~~~", task_count);

        // TODO needing to wrap this in an unwind to create an artificial panic boundary
        // is only cause of sloppy coding. use panic less, throw more Results<> and it
        // might be easy to remove this catch_unwind.
        // TODO Also is it possible to correct the use of AssertUnwindSafe? So it's correct?
        let res = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(
            move || -> Result<(), Error> {
                let delay_log = self.state().client_id == "$$$$$$";

                // Rewrite random targets here.
                if let Task::ControllerCommand(ControllerCommand::RandomTarget(pos)) = value {
                    let cursors = random_cursor(&self.state().client_doc.doc)?;
                    let idx = (pos * (cursors.len() as f64)) as usize;

                    value = Task::ControllerCommand(ControllerCommand::Cursor(
                        Some(cursors[idx].clone()),
                        None,
                    ));
                }

                if !delay_log {
                    log_wasm!(Task(self.state().client_id.clone(), value.clone()));
                }

                match value.clone() {
                    // Handle commands from Native.
                    Task::ControllerCommand(command) => {
                        if self.state().client_id == "$$$$$$" {
                            println!("NATIVE COMMAND TOO EARLY");
                            return Ok(());
                        }

                        native_command(self, command)?;
                    }

                    // Sync sent us an Update command with a new document version.
                    Task::ClientCommand(ClientCommand::Init(new_client_id, doc_span, version)) => {
                        self.state().client_id = new_client_id.clone();
                        self.state().client_doc.init(&Doc(doc_span), version);

                        // Announce.
                        println!("inital version is {:?}", version);

                        log_wasm!(Setup(self.state().client_id.clone()));

                        // If the caret doesn't exist or was deleted, reinitialize it.
                        if !self
                            .with_action_context(|ctx| Ok(has_caret(ctx, true)))
                            .ok()
                            .unwrap_or(true)
                        {
                            // console_log!("add caret");
                            self.client_op(|doc| init_caret(doc)).unwrap();
                        }

                        let res = FrontendCommand::Init(new_client_id);
                        self.send_client(&res).unwrap();

                        // Native drives client state.
                        let state = self.state();
                        let res = FrontendCommand::UpdateFull(
                            doc_as_html(&state.client_doc.doc.0),
                            doc_to_markdown(&state.client_doc.doc.0).unwrap(),
                        );
                        self.send_client(&res).unwrap();
                    }

                    // Sync sent us an Update command with a new document version.
                    Task::ClientCommand(ClientCommand::Update(version, client_id, input_op)) => {
                        if self.state().client_id == "$$$$$$" {
                            return Ok(());
                        }

                        // Generated from original_doc transformed with input_op
                        // let mut bc = vec![];
                        let doc = Op::apply(&self.state().client_doc.original_doc, &input_op);

                        // If this operation is an acknowledgment...
                        if self.state().client_id == client_id {
                            // Confirm pending op, send out next if one is available.
                            if let Some(local_op) = self
                                .state()
                                .client_doc
                                .sync_confirmed_pending_op(&doc, version)
                            {
                                // Send our next operation.
                                self.upload(local_op)?;
                            }
                        } else {
                            // bc = ::oatie::apply::apply_op_bc(&self.state().client_doc.original_doc, &input_op);

                            // A new operation was sent, transform and update our client.
                            println!("---> sync sent new version");
                            let (last_doc, input_op) = self.state()
                                .client_doc
                                .sync_sent_new_version(&doc, version, &input_op);

                            // Native drives client state.
                            let state = self.state();
                            let res = FrontendCommand::Update(
                                // doc_as_html(&state.client_doc.doc.0),
                                ::serde_json::to_string(
                                    &::oatie::apply::apply_op_bc(&last_doc.0, &input_op)
                                ).unwrap(),
                                doc_to_markdown(&state.client_doc.doc.0).unwrap(),
                                input_op,
                            );
                            self.send_client(&res).unwrap();
                        }

                        // Announce.
                        println!("new version is {:?}", version);

                        // If the caret doesn't exist or was deleted, reinitialize it.
                        if !self
                            .with_action_context(|ctx| Ok(has_caret(ctx, true)))
                            .ok()
                            .unwrap_or(true)
                        {
                            // console_log!("adding caret after last op");
                            self.client_op(|doc| init_caret(doc)).unwrap();
                        }
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

                if delay_log {
                    log_wasm!(Task(self.state().client_id.clone(), value.clone()));
                }

                Ok(())
            },
        ));

        if let Ok(value) = res {
            value
        } else if let Err(err) = res {
            // TODO does this actually dump out the error stack trace? otherwise
            // we should just rethrow err? directly or
            bail!("task {} panicked: {:?}", task_count, err);
        } else {
            unreachable!();
        }
    }

    fn upload(&mut self, local_op: Op) -> Result<(), Error> {
        log_wasm!(Debug("CLIENTOP".to_string()));
        let client_id = self.state().client_id.clone();
        let version = self.state().client_doc.version;
        Ok(self.send_sync(ServerCommand::Commit(client_id, local_op, version))?)
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
        Self: Sized,
    {
        // Apply operation.
        let op = self.with_action_context(callback)?;

        // Apply new operation.
        // eprintln!("apply to (d) {:?}", self.state().client_doc.doc);
        let bc = ::oatie::apply::apply_op_bc(&self.state().client_doc.doc.0, &op);
        self.state().client_doc.apply_local_op(&op);

        eprintln!("-----> {:?}", op);

        // Check that our operations can compose well.
        // if cfg!(not(target_arch = "wasm32")) {
        //     // println!("ORIGINAL: {:?}", client.original_doc);
        //     let mut check_op_a = client.op_outstanding.clone().unwrap_or(op_span!([], []));
        //     for (i, op) in client.ops.iter().enumerate() {
        //         // println!("  {}: applying {:?}/{:?}", name, i + 1, client.ops.len());
        //         // println!("  {} 1️⃣: let op_left = op_span!{:?};", name, check_op_a);
        //         // println!("  {} 1️⃣: let op_right = op_span!{:?};", name, op);
        //         check_op_a = Op::compose(&check_op_a, &op);
        //         // println!("  {} 1️⃣: let res = op_span!{:?};", name, check_op_a);
        //         // println!("  {} 1️⃣: let original = doc_span!{:?};", name, client.original_doc);
        //         // println!("  {} 1️⃣: let latest_doc = doc_span!{:?};", name, client.doc);
        //         let _ = Op::apply(&client.original_doc, &check_op_a);
        //     }

        //     assert_eq!(Op::apply(&client.original_doc, &check_op_a), client.doc);
        // }

        // Validate local changes.
        validate_doc(&self.state().client_doc.doc).expect("Local op was malformed");

        // Render the update.
        let state = self.state();
        let res = FrontendCommand::Update(
            ::serde_json::to_string(&bc).unwrap(),
            // doc_as_html(&state.client_doc.doc.0),
            doc_to_markdown(&state.client_doc.doc.0).unwrap(),
            op,
        );
        self.send_client(&res)?;

        // Send any queued payloads.
        if let Some(local_op) = self.state().client_doc.next_payload() {
            self.upload(local_op)?;
        }

        // Update the controls state.
        // TODO should optimize this to not always send this out.
        // console_log!("CUR DOC {:?}", doc);
        let (cur_block, in_list) = self.with_action_context(|doc| identify_block(doc))?;
        println!("current block: {:?}", cur_block);
        println!("in list: {:?}", in_list);
        self.setup_controls(Some((cur_block, in_list)));

        Ok(())
    }
}
