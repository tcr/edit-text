use oatie::doc::*;

// The server is the synchronization server.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ServerCommand {
    // Connect(String),
    Commit(String, Op, usize),
    Log(String),
    TerminateProxy,
}

// Client is an individual user / machine.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ClientCommand {
    // Client id assignment, initial doc, initial version
    Init(String, DocSpan, usize),

    // New document, version, client-id, operation
    Update(usize, String, Op),
}

// Controller is the client interface that is exposed to the frnontend.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ControllerCommand {
    // Connect(String),
    Keypress(u32, bool, bool, bool), // code, meta, shift, alt
    Button(u32),
    Character(u32),
    InsertText(String),
    RenameGroup(String, CurSpan),
    // Load(DocSpan),
    Cursor(Option<CurSpan>, Option<CurSpan>),
    // Target(CurSpan),
    RandomTarget(f64),
    Monkey(bool),
}

// Frontend is the editor components in JavaScript.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum FrontendCommand {
    Init(String),
    Controls(Controls),
    PromptString(String, String, ControllerCommand),
    Update(String, String, Op),
    UpdateFull(String, String),
    Error(String),
    ServerCommand(ServerCommand),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Ui {
    // label, callback, selected
    Button(String, usize, bool),
    ButtonGroup(Vec<Ui>),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Controls {
    pub keys: Vec<(u32, bool, bool)>,
    pub buttons: Vec<Ui>,
}
