use oatie::doc::*;
use wasm_bindgen::prelude::*;
use wasm_typescript_definition::*;

// The server is the synchronization server.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, TypescriptDefinition)]
#[serde(tag = "tag", content = "fields")]
pub enum ServerCommand {
    // Connect(String),
    Commit(String, Op, usize),
    Log(String),
    TerminateProxy,
}

// Client is an individual user / machine.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, TypescriptDefinition)]
#[serde(tag = "tag", content = "fields")]
pub enum ClientCommand {
    // Client id assignment, initial doc, initial version
    Init(String, DocSpan, usize),

    // New document, version, client-id, operation
    Update(usize, String, Op),

    ServerDisconnect,
}

// Controller is the client interface that is exposed to the frnontend.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, TypescriptDefinition)]
#[serde(tag = "tag", content = "fields")]
pub enum ControllerCommand {
    // Connect(String),
    // Load(DocSpan),
    // Target(CurSpan),
    Keypress {
        key_code: u32,
        meta_key: bool,
        shift_key: bool,
        alt_key: bool,
    },
    Button {
        button: u32,
    },
    Character {
        char_code: u32,
    },
    InsertText {
        text: String,
    },
    RenameGroup {
        tag: String,
        curspan: CurSpan,
    },
    Cursor {
        focus: Option<CurSpan>,
        anchor: Option<CurSpan>,
    },
    RandomTarget {
        position: f64,
    },
    Monkey {
        enabled: bool,
    },
}

// Frontend is the editor components in JavaScript.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, TypescriptDefinition)]
#[serde(tag = "tag", content = "fields")]
pub enum FrontendCommand {
    Init(String),
    Controls(Controls),
    PromptString(String, String, ControllerCommand),
    // Bytecode, Op
    RenderDelta(String, Op),
    // HTML
    RenderFull(String),
    Error(String),
    ServerCommand(ServerCommand),

    ServerDisconnect,
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
