// cargo-deps: clipboard="*"

extern crate clipboard;

use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

fn main() {
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    let input = ctx.get_contents().unwrap();

    let mut out = String::new();
    let mut len = "".to_string();
    for c in input.chars() {
        if c == '[' {
            out.push(c);
            out.push_str("\n");

            len.push_str("    ");
            out.push_str(&len);
        } else if c == ']' {
            len = len[0..len.len()-4].to_string();
            out.push_str("\n");
            out.push_str(&len);
            out.push(c);
        } else if c == '\n' {
            out.push(c);
            out.push_str(&len);
        } else {
            out.push(c);
        }
    }
    ctx.set_contents(out).unwrap();
}