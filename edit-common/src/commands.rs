use oatie::doc::*;
use wasm_bindgen::prelude::*;
use wasm_typescript_definition::*;

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

use wasm_bindgen::describe::WasmDescribe;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct JsonEncodable<T>(T);

impl<T> WasmDescribe for JsonEncodable<T> {
    fn describe() {
        JsValue::describe();
    }
}

impl<T> JsonEncodable<T> {
    pub fn inner(&self) -> &T {
        &self.0
    }

    pub fn new(inner: T) -> Self {
        JsonEncodable(inner)
    }
}


// Controller is the client interface that is exposed to the frnontend.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, TypescriptDefinition)]
pub enum ControllerCommand {
    // Connect(String),
    Keypress {
        key_code: u32,
        meta_key: bool,
        shift_key: bool,
        alt_key: bool,
    },
    Button {
        button: u32
    },
    Character {
        char_code: u32,
    },
    InsertText {
        text: String,
    },
    RenameGroup {
        tag: String,
        curspan: JsonEncodable<CurSpan>,
    },
    // Load(DocSpan),
    Cursor {
        focus: JsonEncodable<Option<CurSpan>>,
        anchor: JsonEncodable<Option<CurSpan>>,
    },
    // Target(CurSpan),
    RandomTarget {
        position: f64,
    },
    Monkey {
        enabled: bool,
    },
}


// Frontend is the editor components in JavaScript.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum FrontendCommand {
    Init(String),
    Controls(Controls),
    PromptString(String, String, ControllerCommand),
    // Bytecode, Op
    Update(String, Op),
    // HTML
    UpdateFull(String),
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
