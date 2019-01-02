//! Debug interface custom to edit-text. See debug.ts for client side impl of
//! the DEBUG global variable that powers most of this. Code to use this
//! interface is injected into the browser via WebDriver.

use fantoccini::{
    error::CmdError,
    Client,
};
use futures::future::Future;
use serde_json::value::Value;

pub struct DebugClient {
    pub client: Client,
}

#[allow(unused)]
impl DebugClient {
    pub fn from(client: Client) -> DebugClient {
        DebugClient { client: client }
    }

    pub fn js(&mut self, input: &str) -> impl Future<Item = Value, Error = CmdError> {
        self.client.execute(input, vec![])
    }

    pub fn keypress_char(&mut self, key: char) -> impl Future<Item = (), Error = CmdError> {
        self.js(&format!("return DEBUG.typeChar({});", key as usize))
            .map(|_| ())
    }

    pub fn mousedown(&mut self, x: &str, y: &str) -> impl Future<Item = (), Error = CmdError> {
        self.js(&format!("return DEBUG.mousedown({}, {});", x, y))
            .map(|_| ())
    }

    pub fn caret_to_end_of_line(&mut self) -> impl Future<Item = (), Error = CmdError> {
        self.js(&format!("return DEBUG.caretToEndOfLine();"))
            .map(|_| ())
    }

    pub fn as_markdown(&mut self) -> impl Future<Item = String, Error = CmdError> {
        self.js("return DEBUG.asMarkdown();")
            .map(|x| x.as_str().unwrap().to_owned())
    }
}
