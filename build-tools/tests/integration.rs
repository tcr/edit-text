// rustfmt-edition: 2018

// vvv The nightly features that are commonly needed with async / await
#![recursion_limit = "128"]
#![feature(await_macro, async_await, futures_api)]
#![cfg(feature = "integration")]
// ^^^

#![feature(integer_atomics)]

#[macro_use]
extern crate tokio;

mod common;

use self::common::*;

use fantoccini::Client;
use failure::Error;


#[test]
fn integration_spooky_test() {
    concurrent_editing(async move |
        mut c: Client,
        test_id: String,
        _checkpoint: Checkpoint,
    | -> Result<bool, Error> {
        // Position the caret at the end of the current line.
        await!(sleep_ms(1_000));
        await!(code(&mut c).debug_end_of_line());

        // Type a ghost emoji.
        await!(sleep_ms(1_000));
        await!(code(&mut c).keypress("0x1f47b").execute());

        // todo "DEBUG.keypress()""

        // Wait 4s for clients to receive all pending operations.
        await!(sleep_ms(4_000));

        // Get the innerText of the first h1.
        let heading = await!(
            code(&mut c)
                .js(r#"

// DEBUG.asMarkdown().match(/\S.*$/m);

let h1 = document.querySelector('.edit-text div[data-tag=h1]');
return h1.innerText;

        "#)
                .execute()
        )?;
        eprintln!("[{}] header content: {}", test_id, heading.to_string());

        // End condition: Did the two characters appear across all clients?
        Ok(heading.as_str().unwrap().ends_with("👻👻"))
    });
}
