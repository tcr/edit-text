// rustfmt-edition: 2018

// The nightly features that are commonly needed with async / await
#![recursion_limit = "128"]
#![feature(await_macro, async_await, futures_api)]
#![cfg(feature = "integration")]

#[macro_use]
extern crate tokio;

mod common;

use self::common::*;
use failure::*;
use fantoccini::Locator;

/// Basic test that moves the cursor to the end of the line, types a Ghost emoji,
/// waits for the dust to settle, then sees if we get two Ghost emoji on both clients.
#[test]
fn integration_spooky_test() {
    let markdown = "# a cold freezing night";
    concurrent_editing(markdown, async move |mut debug, test_id, _checkpoint| {
        // Position the caret at the end of the current line.
        await!(debug.caret_to_end_of_line())?;
        await!(sleep_ms(1_000))?;

        // Type a ghost emoji.
        // Wait an arbitrary 4s for clients to receive all pending operations.
        await!(debug.keypress_char('\u{01f47b}'))?;
        await!(sleep_ms(4_000))?;

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

        // We should find a sequence of "000" and a sequence of "111", though
        // either string may occur in either order.
        Ok(markdown.find("000").is_some() && markdown.find("111").is_some())
    });
}

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

    individual_editing(markdown, async move |mut debug, _test_id, _checkpoint| {
        // (font-weight, font-style, background)
        type ComputedStyle = (u64, String, String);
        async fn get_style(
            debug: &mut DebugClient,
            selector: String,
        ) -> Result<ComputedStyle, Error> {
            Ok(serde_json::from_value(await!(debug.js(&format!(r#"
                let style = window.getComputedStyle({});
                return [Number(style.fontWeight), String(style.fontStyle), String(style.backgroundColor)];
            "#, selector)))?)?)
        }

        // Normal text pararaph.
        let (weight, style, _) = await!(get_style(
            &mut debug,
            "DEBUG.root().querySelectorAll('span')[0]".to_string()
        ))?;
        assert_eq!(weight, 400, "Font weight is 400 (normal)");
        assert_ne!(style, "italic", "Font style is not italic");

        // Bold paragraph.
        let (weight, style, _) = await!(get_style(
            &mut debug,
            "DEBUG.root().querySelectorAll('span')[1]".to_string()
        ))?;
        assert!(weight > 400, "Font weight is greater than 400 (normal)");
        assert_ne!(style, "italic", "Font style is not italic");

        // Italic paragraph.
        let (weight, style, _) = await!(get_style(
            &mut debug,
            "DEBUG.root().querySelectorAll('span')[2]".to_string()
        ))?;
        assert_eq!(weight, 400, "Font weight is 400 (normal)");
        assert_eq!(style, "italic", "Font style is italic");

        // Bold and italic paragraph.
        let (weight, style, _) = await!(get_style(
            &mut debug,
            "DEBUG.root().querySelectorAll('span')[3]".to_string()
        ))?;
        assert!(weight > 400, "Font weight is greater than 400 (normal)");
        assert_eq!(style, "italic", "Font style is italic");

        // No selection.
        let (_, _, bg) = await!(get_style(
            &mut debug,
            "DEBUG.root().querySelectorAll('span')[0]".to_string()
        ))?;
        assert_eq!(bg, "rgba(0, 0, 0, 0)", "Background is transparent");

        // Select all, check that selection changes background.
        await!(debug.js("DEBUG.selectAll()"))?;
        let (_, _, bg) = await!(get_style(
            &mut debug,
            "DEBUG.root().querySelectorAll('span')[0]".to_string()
        ))?;
        assert_ne!(bg, "rgba(0, 0, 0, 0)", "Selection is not transparent");

        Ok(true)
    });
}

/// Checks that the load/save Markdown dialog works.
#[test]
fn integration_markdown_dialog() {
    fn markdown_a() -> &'static str {
        r#"
# Header Text

Body Text
        "#.trim()
    }

    fn markdown_b() -> &'static str {
        r#"
# Goodbye
        "#.trim()
    }

    individual_editing(markdown_a(), async move |debug, _test_id, mut _checkpoint| -> Result<bool, Error> {
        let client = debug.client;

        // Open the dialog for Markdown import/export.
        let button = await!(client.wait_for_find(Locator::XPath("//*[@id = 'toolbar']//button[text()='Load/Save']")))?;
        await!(sleep_ms(2_000))?; // To finish loading
        let client = await!(button.click())?;

        let mut textarea = await!(client.wait_for_find(Locator::Css("#modal-dialog textarea")))?;
        let value = await!(textarea.prop("value"))?.unwrap();
        let mut client = textarea.client();

        // Markdown should be basically unchanged (using this simple example).
        if markdown_a() != value {
            return Ok(false);
        }

        // Focus and set the textarea contents to a new value.
        await!(client.execute(r##"
document.querySelector('#modal-dialog textarea').select();
        "##, vec![]))?;
        let mut textarea = await!(client.find(Locator::Css("#modal-dialog textarea")))?;
        await!(textarea.send_keys(markdown_b()))?;
        let mut client = textarea.client();

        // Click the "Load" button and wait for the URL to change (page reload).
        let import = await!(client.find(Locator::Css(".modal-buttons button.load")))?;
        let client = await!(import.click())?;

        // Wait for page change.
        await!(sleep_ms(5_000))?;

        let mut header = await!(client.wait_for_find(Locator::Css(".edit-text")))?;
        let header_text = await!(header.text())?;
        let client = header.client();
        assert_eq!(header_text.trim(), "Goodbye");

        // Open the dialog for Markdown import/export.
        let button = await!(client.wait_for_find(Locator::XPath("//*[@id = 'toolbar']//button[text()='Load/Save']")))?;
        let client = await!(button.click())?;
        let mut textarea = await!(client.wait_for_find(Locator::Css("#modal-dialog textarea")))?;
        let value = await!(textarea.prop("value"))?.unwrap();

        // Markdown should be basically unchanged (using this simple example).
        if markdown_b() != value {
            return Ok(false);
        }

        Ok(true)
    });
}
