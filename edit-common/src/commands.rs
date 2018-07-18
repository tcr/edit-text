use oatie::doc::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UserToSyncCommand {
    // Connect(String),
    Commit(String, Op, usize),
    Log(String),
    TerminateProxy,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncToUserCommand {
    // Client id assignment, initial doc, initial version
    Init(String, DocSpan, usize),

    // New document, version, client-id, operation
    Update(usize, String, Op),
}

// Commands received from frontend.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum FrontendToUserCommand {
    // Connect(String),
    Keypress(u32, bool, bool, bool), // code, meta, shift, alt
    Button(u32),
    Character(u32),
    InsertText(String),
    RenameGroup(String, CurSpan),
    // Load(DocSpan),
    CursorAnchor(CurSpan),
    CursorTarget(CurSpan),
    // Target(CurSpan),
    RandomTarget(f64),
    Monkey(bool),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Ui {
    // label, callback, selected
    Button(String, usize, bool),
    ButtonGroup(Vec<Ui>),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Controls {
    pub keys: Vec<(u32, bool, bool)>,
    pub buttons: Vec<Ui>,
}

// Commands to send to Frontend.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UserToFrontendCommand {
    Init(String),
    Controls(Controls),
    PromptString(String, String, FrontendToUserCommand),
    Update(String, Option<Op>),
    Error(String),
    UserToSyncCommand(UserToSyncCommand),
}
