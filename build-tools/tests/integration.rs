// rustfmt-edition: 2018

// The nightly features that are commonly needed with async / await
#![recursion_limit = "128"]
#![feature(await_macro, async_await, futures_api)]
#![cfg(feature = "integration")]

#[macro_use]
extern crate tokio;

mod common;

use failure::Error;
use self::common::*;

/*
/// Basic test that moves the cursor to the end of the line, types a Ghost emoji,
/// waits for the dust to settle, then sees if we get two Ghost emoji on both clients.
#[test]
fn integration_spooky_test() {
    let markdown = "# a cold freezing night";
    concurrent_editing(markdown, async move |mut debug, test_id, _checkpoint| {
        // Position the caret at the end of the current line.
        await!(debug.caret_to_end_of_line());
        await!(sleep_ms(1_000));

        // Type a ghost emoji.
        // Wait an arbitrary 4s for clients to receive all pending operations.
        await!(debug.keypress_char('\u{01f47b}'));
        await!(sleep_ms(4_000));

        // Get the Markdown content.
        let markdown = await!(debug.as_markdown())?;
        eprintln!("[{}] markdown content: {:?}", test_id, markdown);

        // End condition: Did the two characters appear across all clients?
        Ok(markdown
            .lines()
            .next()
            .unwrap()
            .ends_with("\u{01f47b}\u{01f47b}"))
    });
}

/// Clients typing in a sequential order should have predictable results.
#[test]
fn integration_sequential_test() {
    let markdown = "xxxxx";
    concurrent_editing(markdown, async move |mut debug, test_id, mut checkpoint| {
        // Type a character.
        checkpoint.sequential();
        await!(debug.keypress_char(if checkpoint.index == 0 { '0' } else { '1' }))?;
        checkpoint.sync();

        // Type a character.
        checkpoint.sequential();
        await!(debug.keypress_char(if checkpoint.index == 0 { '0' } else { '1' }))?;
        checkpoint.sync();

        // Type a character.
        checkpoint.sequential();
        await!(debug.keypress_char(if checkpoint.index == 0 { '0' } else { '1' }))?;
        checkpoint.sync();

        // Wait an arbitrary 4s for clients to receive all pending operations.
        await!(sleep_ms(4_000))?;

        // Get the Markdown content.
        let markdown = await!(debug.as_markdown())?;
        eprintln!("[{}] markdown content: {:?}", test_id, markdown);

        Ok(markdown.find("000").is_some() && markdown.find("111").is_some())
    });
}
*/

/// Checks that bold and italic text is actually rendered in their respective
/// styles.
#[test]
fn integration_styles() {
    let markdown = r#"
paragraph text

**bold text**

*italic text*

***bold and italic text***
    "#;

    individual_editing(markdown, async move |mut debug, test_id, mut checkpoint| {
        // (font-weight, font-style, background)
        type ComputedStyle = (u64, String, String);
        async fn get_style(debug: &mut DebugClient, selector: String) -> Result<ComputedStyle, Error> {
            Ok(serde_json::from_value(await!(debug.js(&format!(r#"
                let style = window.getComputedStyle({});
                return [Number(style.fontWeight), String(style.fontStyle), String(style.backgroundColor)];
            "#, selector)))?)?)
        }

        // Normal text pararaph.
        let (weight, style, _) = await!(get_style(&mut debug, "DEBUG.root().querySelectorAll('span')[0]".to_string()))?;
        assert_eq!(weight, 400, "Font weight is 400 (normal)");
        assert_ne!(style, "italic", "Font style is not italic");

        // Bold paragraph.
        let (weight, style, _) = await!(get_style(&mut debug, "DEBUG.root().querySelectorAll('span')[1]".to_string()))?;
        assert!(weight > 400, "Font weight is greater than 400 (normal)");
        assert_ne!(style, "italic", "Font style is not italic");

        // Italic paragraph.
        let (weight, style, _) = await!(get_style(&mut debug, "DEBUG.root().querySelectorAll('span')[2]".to_string()))?;
        assert_eq!(weight, 400, "Font weight is 400 (normal)");
        assert_eq!(style, "italic", "Font style is italic");

        // Bold and italic paragraph.
        let (weight, style, _) = await!(get_style(&mut debug, "DEBUG.root().querySelectorAll('span')[3]".to_string()))?;
        assert!(weight > 400, "Font weight is greater than 400 (normal)");
        assert_eq!(style, "italic", "Font style is italic");

        // No selection.
        let (_, _, bg) = await!(get_style(&mut debug, "DEBUG.root().querySelectorAll('span')[0]".to_string()))?;
        assert_eq!(bg, "rgba(0, 0, 0, 0)", "Background is transparent");

        // Select all, check that selection changes background.
        await!(debug.js("DEBUG.selectAll()"));
        let (_, _, bg) = await!(get_style(&mut debug, "DEBUG.root().querySelectorAll('span')[0]".to_string()))?;
        assert_ne!(bg, "rgba(0, 0, 0, 0)", "Selection is not transparent");

        Ok(true)
    });
}
