//! Debug interface custom to edit-text.
//!
//! See debug.ts for client side impl of DEBUG global.

use fantoccini::{
    error::CmdError,
    Client,
};
use futures::future::Future;
use serde_json::value::Value;

pub struct DebugClient {
    client: Client,
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
        self.js(&format!(
            r#"
var event = new KeyboardEvent("keypress", {{
    bubbles: true,
    cancelable: true,
    charCode: {},
}});
document.dispatchEvent(event);
            "#,
            key as usize,
        ))
        .map(|_| ())
    }

    pub fn mousedown(&mut self, x: &str, y: &str) -> impl Future<Item = (), Error = CmdError> {
        self.js(&format!(
            r#"
var evt = new MouseEvent("mousedown", {{
    bubbles: true,
    cancelable: true,
    clientX: {},
    clientY: {},
}});
document.querySelector('.edit-text').dispatchEvent(evt);
            "#,
            x, y
        ))
        .map(|_| ())
    }

    pub fn debug_end_of_line(&mut self) -> impl Future<Item = (), Error = CmdError> {
        self.js(r#"

// DEBUG.endOfLine();

let marker = document.querySelector('.edit-text div[data-tag=h1] span');
let clientX = marker.getBoundingClientRect().right;
let clientY = marker.getBoundingClientRect().top;

var evt = new MouseEvent("mousedown", {
    bubbles: true,
    cancelable: true,
    clientX: clientX - 3,
    clientY: clientY + 3,
});
document.querySelector('.edit-text').dispatchEvent(evt);

            "#)
            .map(|_| ())
    }

    pub fn as_markdown(&mut self) -> impl Future<Item = String, Error = CmdError> {
        self.js("return DEBUG.asMarkdown()")
            .map(|x| x.as_str().unwrap().to_owned())
    }
}
