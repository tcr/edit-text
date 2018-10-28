// rustfmt-edition: 2018

// The nightly features that are commonly needed with async / await
#![recursion_limit = "128"]
#![feature(await_macro, async_await, futures_api)]
#![cfg(feature = "integration")]

#[macro_use]
extern crate tokio;

mod common;

use self::common::*;

/// Basic test that moves the cursor to the end of the line, types a Ghost emoji,
/// waits for the dust to settle, then sees if we get two Ghost emoji on both clients.
#[test]
fn integration_spooky_test() {
    concurrent_editing(async move |mut debug, test_id, _checkpoint| {
        // Position the caret at the end of the current line.
        await!(sleep_ms(1_000));
        await!(debug.debug_end_of_line());

        // Type a ghost emoji.
        await!(sleep_ms(1_000));
        await!(debug.keypress("0x1f47b"));

        // Wait 4s for clients to receive all pending operations.
        await!(sleep_ms(4_000));

        // Get the innerText of the first h1.
        let markdown = await!(debug.as_markdown())?;
        eprintln!("[{}] markdown content: {:?}", test_id, markdown);

        // End condition: Did the two characters appear across all clients?
        Ok(markdown.lines().next().unwrap().ends_with("👻👻"))
    });
}
