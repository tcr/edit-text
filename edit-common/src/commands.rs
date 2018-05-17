use oatie::doc::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UserToSyncCommand {
    // Connect(String),
    Commit(String, Op, usize),
    TerminateProxy,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyncToUserCommand {
    // Client id assignment, initial doc, initial version
    Init(String, DocSpan, usize),

    // New document, version, client-id, operation
    Update(DocSpan, usize, String, Op),
}

// Commands received from frontend.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum FrontendToUserCommand {
    // Connect(String),
    Keypress(u32, bool, bool, bool), // code, meta, shift, alt
    Button(u32),
    Character(u32),
    RenameGroup(String, CurSpan),
    // Load(DocSpan),
    Target(CurSpan),
    RandomTarget(f64),
    Monkey(bool),
}

// Commands to send to Frontend.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UserToFrontendCommand {
    Init(String),
    Controls {
        keys: Vec<(u32, bool, bool)>,
        buttons: Vec<(usize, String, bool)>,
    },
    PromptString(String, String, FrontendToUserCommand),
    Update(String, Option<Op>),
    Error(String),
    UserToSyncCommand(UserToSyncCommand),
}
