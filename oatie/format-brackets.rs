// cargo-deps: clipboard="*"

extern crate clipboard;

use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

fn main() {
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    let input = ctx.get_contents().unwrap();

    let mut out = String::new();
    let mut len = "".to_string();
    let mut chars = input.chars().peekable();
    loop {
        let mut c = if let Some(c) = chars.next() { c } else { break; };
        if c == '[' {
            out.push(c);

            while chars.peek().unwrap().is_whitespace() {
                let _ = chars.next();
            }
            if chars.peek() == Some(&']') {
                c = chars.next().unwrap();
            } else {
                out.push_str("\n");

                len.push_str("    ");
                out.push_str(&len);
            }
        } else if c == ']' {
            len = len[0..len.len()-4].to_string();
            out.push_str("\n");
            out.push_str(&len);
        } else if c == '\n' {
            out.push(c);
            out.push_str(&len);
        } else {
            out.push(c);
        }

        if c == ']' {
            out.push(c);
            if chars.peek() == Some(&')') {
                out.push(chars.next().unwrap());
                if chars.peek() == Some(&',') {
                    out.push(chars.next().unwrap());
                    while chars.peek().unwrap().is_whitespace() {
                        let _ = chars.next();
                    }
                    out.push_str("\n");
                    out.push_str(&len);
                }
            }
        }
    }
    ctx.set_contents(out).unwrap();
}