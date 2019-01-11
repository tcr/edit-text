mod actions;
mod clientdoc;

pub use self::actions::*;
pub use self::clientdoc::*;

use crate::random::*;
use crate::walkers::Pos;
use edit_common::{
    commands::*,
    doc_as_html,
};
use failure::Error;
use oatie::doc::*;
use oatie::rtf::*;
use oatie::validate::validate_doc;
use std::char::from_u32;
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};
use std::sync::Arc;

// Shorthandler
// code, meta, shift, alt, callback
struct KeyHandler<C: ClientController>(
    u32,
    bool,
    bool,
    bool,
    Box<dyn Fn(&mut C) -> Result<(), Error>>,
);

impl<C: ClientController> KeyHandler<C> {
    fn matches(&self, code: u32, meta_key: bool, shift_key: bool, alt_key: bool) -> bool {
        self.0 == code && self.1 == meta_key && self.2 == shift_key && self.3 == alt_key
    }

    fn invoke(&self, client: &mut C) -> Result<(), Error> {
        self.4(client)
    }
}

fn key_handlers<C: ClientController>() -> Vec<KeyHandler<C>> {
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
        // opt + shift + left
        KeyHandler(
            37,
            false,
            true,
            true,
            Box::new(|client| client.client_op(|doc| caret_word_move(doc, false, true))),
        ),
        // opt + shift + right
        KeyHandler(
            39,
            false,
            true,
            true,
            Box::new(|client| client.client_op(|doc| caret_word_move(doc, true, true))),
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
            Box::new(|client| {
                client.client_op(|doc| add_string(doc, "\n").map(|ctx| ctx.result()))
            }),
        ),
        // tab
        KeyHandler(
            9,
            false,
            false,
            false,
            Box::new(|client| client.client_op(|doc| toggle_list(doc))),
        ),
        // opt + left
        KeyHandler(
            37,
            false,
            false,
            true,
            Box::new(|client| client.client_op(|doc| caret_word_move(doc, false, false))),
        ),
        // opt + left
        KeyHandler(
            39,
            false,
            false,
            true,
            Box::new(|client| client.client_op(|doc| caret_word_move(doc, true, false))),
        ),
        // cmd + a
        KeyHandler(
            65,
            true,
            false,
            false,
            Box::new(|client| client.client_op(|doc| caret_select_all(doc))),
        ),
    ]
}

pub fn button_handlers<C: ClientController>(
    state: Option<CaretState>,
) -> (Vec<Box<dyn Fn(&mut C) -> Result<(), Error>>>, Vec<Ui>) {
    let mut callbacks: Vec<Box<dyn Fn(&mut C) -> Result<(), Error>>> = vec![];

    macro_rules! callback {
        ($t:expr) => {{
            callbacks.push(Box::new($t));
            callbacks.len() - 1
        }};
    }

    let is_bold = state
        .as_ref()
        .map(|x| x.styles.contains(&RtfStyle::Bold))
        .unwrap_or(false);
    let is_italic = state
        .as_ref()
        .map(|x| x.styles.contains(&RtfStyle::Italic))
        .unwrap_or(false);

    let ui = vec![
        Ui::ButtonGroup(vec![
            Ui::Button(
                "Text".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, Attrs::Para))),
                state.as_ref().map(|x| x.block == "p").unwrap_or(false),
            ),
            Ui::Button(
                "H1".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, Attrs::Header(1)))),
                // TODO i wish we could match on strings, use matches! here
                state.as_ref().map(|x| x.block == "h1").unwrap_or(false),
            ),
            Ui::Button(
                "H2".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, Attrs::Header(2)))),
                state.as_ref().map(|x| x.block == "h2").unwrap_or(false),
            ),
            Ui::Button(
                "H3".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, Attrs::Header(3)))),
                state.as_ref().map(|x| x.block == "h3").unwrap_or(false),
            ),
            Ui::Button(
                "H4".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, Attrs::Header(4)))),
                state.as_ref().map(|x| x.block == "h4").unwrap_or(false),
            ),
            Ui::Button(
                "H5".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, Attrs::Header(5)))),
                state.as_ref().map(|x| x.block == "h5").unwrap_or(false),
            ),
            Ui::Button(
                "H6".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, Attrs::Header(6)))),
                state.as_ref().map(|x| x.block == "h6").unwrap_or(false),
            ),
            Ui::Button(
                "Code".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, Attrs::Code))),
                state.as_ref().map(|x| x.block == "pre").unwrap_or(false),
            ),
            Ui::Button(
                "HTML".to_string(),
                callback!(|client| client.client_op(|doc| replace_block(doc, Attrs::Html))),
                state.as_ref().map(|x| x.block == "html").unwrap_or(false),
            ),
        ]),
        Ui::Button(
            "List".to_string(),
            callback!(|client| client.client_op(|doc| toggle_list(doc))),
            state.as_ref().map(|x| x.in_list).unwrap_or(false),
        ),
        Ui::Button(
            "HR".to_string(),
            callback!(|client| client.client_op(|doc| split_block(doc, true))),
            false,
        ),
        Ui::ButtonGroup(vec![
            Ui::Button(
                "Bold".to_string(),
                callback!(move |client| client.client_op(|doc| if is_bold {
                    remove_styles(doc, StyleSet::from(hashset![RtfStyle::Bold]))
                } else {
                    apply_style(doc, RtfStyle::Bold, None)
                })),
                is_bold,
            ),
            Ui::Button(
                "Italic".to_string(),
                callback!(move |client| client.client_op(|doc| if is_italic {
                    remove_styles(doc, StyleSet::from(hashset![RtfStyle::Italic]))
                } else {
                    apply_style(doc, RtfStyle::Italic, None)
                })),
                is_italic,
            ),
        ]),
    ];

    (callbacks, ui)
}

fn controller_command<C: ClientController>(
    client: &mut C,
    req: ControllerCommand,
) -> Result<(), Error> {
    match req {
        ControllerCommand::RenameGroup { tag: _, curspan: _ } => {
            unimplemented!();
            // client.client_op(|doc| replace_block(doc, &tag))?;
        }
        ControllerCommand::Button { button: index } => {
            // Find which button handler to respond to this command.
            let caret_state = client.state().last_caret_state.clone();
            button_handlers(caret_state)
                .0
                .get(index as usize)
                .map(|handler| handler(client));
        }
        ControllerCommand::Keypress {
            key_code,
            meta_key,
            shift_key,
            alt_key,
        } => {
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
        ControllerCommand::Character { char_code } => {
            client.client_op(|doc| {
                let c: char = from_u32(char_code).unwrap_or('?');
                if c == '\0' {
                    bail!("expected non-null character");
                }

                add_string(doc, &format!("{}", c)).map(|ctx| ctx.result())
            })?;
        }
        ControllerCommand::InsertText { text } => {
            client.client_op(|doc| add_string(doc, &text).map(|ctx| ctx.result()))?;
        }
        ControllerCommand::RandomTarget { .. } => {
            // This should never happen! We rewrite RandomTarget beforehand in
            // the method handle_task.
            unreachable!();
        }
        ControllerCommand::Cursor { focus, anchor } => {
            // FIXME Why is random click failing?
            // console_log!("cursor ------> {:?} {:?}", focus, anchor);
            match (focus, anchor) {
                (Some(focus), Some(anchor)) => {
                    client.client_op(|ctx| -> Result<Op<RtfSchema>, Error> {
                        Ok(Op::transform_advance(
                            &cur_to_caret(&ctx, &focus, Pos::Focus)?,
                            &cur_to_caret(&ctx, &anchor, Pos::Anchor)?,
                        ))
                    })?;
                }
                (Some(focus), None) => {
                    client.client_op(|ctx| cur_to_caret(&ctx, &focus, Pos::Focus))?;
                }
                (None, Some(anchor)) => {
                    client.client_op(|ctx| cur_to_caret(&ctx, &anchor, Pos::Anchor))?;
                }
                (None, None) => {} // ???
            }
        }
        ControllerCommand::CursorSelectWord { focus } => {
            client.client_op(|ctx| caret_word_select(&ctx, &focus))?;
        }
        ControllerCommand::Monkey { enabled: setting } => {
            console_log!("received monkey setting: {:?}", setting);
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
    pub client_doc: ClientDoc,
    pub last_caret_state: Option<CaretState>,
    pub last_controls: Option<Controls>,

    pub monkey: Arc<AtomicBool>,
    pub alive: Arc<AtomicBool>,
    pub task_count: usize,
}

use std::cell::RefMut;

/// Trait shared by the "wasm" and "client proxy" implementations.
/// Most methods are implemented on this trait, not its implementors.
pub trait ClientController {
    fn state(&mut self) -> RefMut<Client>;
    fn send_frontend(&self, req: &FrontendCommand) -> Result<(), Error>;
    fn send_server(&self, req: &ServerCommand) -> Result<(), Error>;

    fn setup_controls(&mut self, caret_state: Option<CaretState>)
    where
        Self: Sized,
    {
        self.state().last_caret_state = caret_state.clone();

        let controls_object = Controls {
            keys: key_handlers::<Self>()
                .into_iter()
                .map(|x| (x.0, x.1, x.2))
                .collect(),
            buttons: button_handlers::<Self>(caret_state).1,
        };

        if Some(controls_object.clone()) != self.state().last_controls {
            self.send_frontend(&FrontendCommand::Controls(controls_object.clone()))
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
                let delay_log = self.state().client_doc.client_id == "$$$$$$";

                // Rewrite random targets here.
                if let Task::ControllerCommand(ControllerCommand::RandomTarget { position: pos }) =
                    value
                {
                    let cursors = random_cursor(&self.state().client_doc.doc)?;

                    let idx = (pos * (cursors.len() as f64)) as usize;
                    console_log!("WHAT {:?} {:?} {:?}", pos, cursors.len(), idx);
                    let cursor = cursors[idx].clone();
                    value = Task::ControllerCommand(ControllerCommand::Cursor {
                        focus: Some(cursor.clone()),
                        anchor: Some(cursor),
                    });
                }

                if !delay_log {
                    log_wasm!(Task(
                        self.state().client_doc.client_id.clone(),
                        value.clone()
                    ));
                }

                match value.clone() {
                    // Handle all commands from Frontend.
                    Task::ControllerCommand(command) => {
                        if self.state().client_doc.client_id == "$$$$$$" {
                            println!("FRONTEND COMMAND ARRIVED TOO EARLY");
                            return Ok(());
                        }

                        controller_command(self, command)?;
                    }

                    // Server sent the client the initial document.
                    Task::ClientCommand(ClientCommand::Init(new_client_id, doc_span, version)) => {
                        self.state().client_doc.client_id = new_client_id.clone();
                        self.state().client_doc.init(&Doc(doc_span), version);

                        // Announce.
                        println!("inital version is {:?}", version);

                        log_wasm!(Setup(self.state().client_doc.client_id.clone()));

                        // If the caret doesn't exist or was deleted, reinitialize it.
                        if !self
                            .with_action_context(|ctx| Ok(ctx.get_walker(Pos::Focus).is_ok()))
                            .ok()
                            .unwrap_or(true)
                        {
                            // console_log!("add caret");
                            self.client_op(|doc| init_caret(doc)).unwrap();
                        }

                        let res = FrontendCommand::Init(new_client_id);
                        self.send_frontend(&res).unwrap();

                        // Native drives client state.
                        let state = self.state();
                        let res = FrontendCommand::RenderFull(doc_as_html(&state.client_doc.doc.0));
                        drop(state);
                        self.send_frontend(&res).unwrap();
                    }

                    // Server sent us a new document version.
                    Task::ClientCommand(ClientCommand::Update(version, client_id, input_op)) => {
                        if self.state().client_doc.client_id == "$$$$$$" {
                            return Ok(());
                        }

                        // Generated from original_doc transformed with input_op
                        // let mut bc = vec![];
                        let doc = Op::apply(&self.state().client_doc.original_doc, &input_op);

                        // If this operation is an acknowledgment...
                        if self.state().client_doc.client_id == client_id {
                            // Confirm pending op, send out next if one is available.
                            let local_op = self
                                .state()
                                .client_doc
                                .sync_confirmed_pending_op(&doc, version);
                            if let Some(local_op) = local_op {
                                // Send our next operation.
                                self.upload(local_op)?;
                            }
                        } else {
                            // bc = ::oatie::apply::apply_op_bc(&self.state().client_doc.original_doc, &input_op);

                            // A new operation was sent, transform and update our client.
                            println!("---> sync sent new version");
                            let (last_doc, input_op) = self
                                .state()
                                .client_doc
                                .sync_sent_new_version(&doc, version, &input_op);

                            // Client drives frontend frontend state.
                            let res = if cfg!(feature = "full_client_updates") {
                                // Fully refresh the client.
                                FrontendCommand::RenderFull(doc_as_html(
                                    &self.state().client_doc.doc.0,
                                ))
                            } else {
                                // Render delta.
                                FrontendCommand::RenderDelta(
                                    serde_json::to_string(&oatie::apply::apply_op_bc(
                                        &last_doc.0,
                                        &input_op,
                                    ))
                                    .unwrap(),
                                    input_op,
                                )
                            };
                            self.send_frontend(&res).unwrap();
                        }

                        // Announce.
                        println!("new version is {:?}", version);

                        // If the caret doesn't exist or was deleted by this update,
                        // reinitialize it.
                        if !self
                            .with_action_context(|ctx| Ok(ctx.get_walker(Pos::Focus).is_ok()))
                            .ok()
                            .unwrap_or(true)
                        {
                            // console_log!("adding caret after last op");
                            self.client_op(|doc| init_caret(doc)).unwrap();
                        }
                    }

                    Task::ClientCommand(ClientCommand::ServerDisconnect) => {
                        // Notify frontend.
                        self.send_frontend(&FrontendCommand::ServerDisconnect)
                            .unwrap();
                    }
                }

                if delay_log {
                    log_wasm!(Task(
                        self.state().client_doc.client_id.clone(),
                        value.clone()
                    ));
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

    fn upload(&mut self, local_op: Op<RtfSchema>) -> Result<(), Error> {
        log_wasm!(Debug("CLIENTOP".to_string()));
        let client_id = self.state().client_doc.client_id.clone();
        let version = self.state().client_doc.version;
        Ok(self.send_server(&ServerCommand::Commit(client_id, local_op, version))?)
    }

    // TODO combine with client_op?
    fn with_action_context<C, T>(&mut self, callback: C) -> Result<T, Error>
    where
        C: Fn(ActionContext) -> Result<T, Error>,
    {
        let doc = self.state().client_doc.doc.clone();
        let client_id = self.state().client_doc.client_id.clone();

        callback(ActionContext::new(doc, client_id))
    }

    fn client_op<C>(&mut self, callback: C) -> Result<(), Error>
    where
        C: Fn(ActionContext) -> Result<Op<RtfSchema>, Error>,
        Self: Sized,
    {
        // Apply operation.
        let op = self.with_action_context(callback)?;

        // Apply new operation.
        // eprintln!("apply to (d) {:?}", self.state().client_doc.doc);
        let bc = ::oatie::apply::apply_op_bc(&self.state().client_doc.doc.0, &op);
        self.state().client_doc.apply_local_op(&op);

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
        //         // println!("  {} 1️⃣: let original = doc!{:?};", name, client.original_doc);
        //         // println!("  {} 1️⃣: let latest_doc = doc!{:?};", name, client.doc);
        //         let _ = Op::apply(&client.original_doc, &check_op_a);
        //     }

        //     assert_eq!(Op::apply(&client.original_doc, &check_op_a), client.doc);
        // }

        // Validate local changes.
        validate_doc(&self.state().client_doc.doc).expect("Local op was malformed");

        // Render our local update.
        let res = if cfg!(feature = "full_client_updates") {
            // Fully refresh the client.
            FrontendCommand::RenderFull(doc_as_html(&self.state().client_doc.doc.0))
        } else {
            // Send a delta update.
            FrontendCommand::RenderDelta(serde_json::to_string(&bc).unwrap(), op)
        };
        self.send_frontend(&res)?;

        // Send any queued payloads.
        let local_op = self.state().client_doc.next_payload();
        if let Some(local_op) = local_op {
            self.upload(local_op)?;
        }

        // Update the controls state.
        // TODO should optimize this to not always send this out.
        let caret_state = self.with_action_context(|doc| identify_block(doc))?;
        self.setup_controls(Some(caret_state));

        Ok(())
    }
}
