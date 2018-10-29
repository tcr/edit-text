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
        await!(debug.debug_end_of_line());
        await!(sleep_ms(1_000));

        // Type a ghost emoji.
        // Wait an arbitrary 4s for clients to receive all pending operations.
        await!(debug.keypress("0x1f47b"));
        await!(sleep_ms(4_000));

        // Get the Markdown content.
        let markdown = await!(debug.as_markdown())?;
        eprintln!("[{}] markdown content: {:?}", test_id, markdown);

        // End condition: Did the two characters appear across all clients?
        Ok(markdown.lines().next().unwrap().ends_with("👻👻"))
    });
}

/// Clients typing in a sequential order should have predictable results.
#[test]
fn integration_sequential_test() {
    concurrent_editing(async move |mut debug, test_id, mut checkpoint| {
        // Type a character.
        checkpoint.sequential();
        await!(debug.keypress(&format!(r#""{}".charCodeAt(0)"#, checkpoint.index)));
        checkpoint.sync();

        // Type a character.
        checkpoint.sequential();
        await!(debug.keypress(&format!(r#""{}".charCodeAt(0)"#, checkpoint.index)));
        checkpoint.sync();

        // Type a character.
        checkpoint.sequential();
        await!(debug.keypress(&format!(r#""{}".charCodeAt(0)"#, checkpoint.index)));
        checkpoint.sync();

        // Wait an arbitrary 4s for clients to receive all pending operations.
        await!(sleep_ms(4_000));

        // Get the Markdown content.
        let markdown = await!(debug.as_markdown())?;
        eprintln!("[{}] markdown content: {:?}", test_id, markdown);

        Ok(markdown.find("000").is_some() && markdown.find("111").is_some())
    });
}

