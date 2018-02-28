#![feature(nll)]

extern crate ctrlc;
extern crate failure;
extern crate nix;
#[macro_use]
extern crate quicli;
extern crate regex;
extern crate shlex;
extern crate toml;

pub mod dtrace;

use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use toml::Value;
use std::cell::RefCell;
use dtrace::*;

fn await_ctrlc() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    while running.load(Ordering::SeqCst) {
        ::std::thread::sleep(::std::time::Duration::from_millis(10));
    }

    eprintln!();
}

#[derive(Debug)]
struct Frame {
    count: i64,
    library: Option<String>,
    target: String,
    address: Option<String>,
}

fn toml_to_frame(toml: &Value) -> Option<Frame> {
    if let Some((Some(count), Some(target))) = toml.as_table().map(|x| {
        (
            x.get("count").and_then(|v| v.as_integer()),
            x.get("value").and_then(|v| v.as_str()),
        )
    }) {
        let mut iter = target.rsplitn(2, '`');
        let target = iter.next().unwrap().to_owned();
        let library = iter.next().map(str::to_owned);
        let mut iter = target.splitn(2, '+');
        let target = iter.next().unwrap().to_owned();
        let address = iter.next().map(str::to_owned);
        Some(Frame {
            count,
            library,
            target,
            address,
        })
    } else {
        None
    }
}

thread_local! {
    static REGEX_CACHE: RefCell<HashMap<String, Regex>> = RefCell::new(HashMap::new());
}

fn regex_replace(input: &str, re: &str, rep: &str) -> String {
    REGEX_CACHE.with(|f| {
        let mut cache = f.borrow_mut();
        if !cache.contains_key(re) {
            cache.insert(re.to_owned(), Regex::new(re).unwrap());
        }
        cache[re].replace_all(input, rep).to_string()
    })
}

/// https://github.com/Yamakaky/rust-unmangle/blob/master/rust-unmangle
fn demangle_maybe(input: &str) -> String {
    if input.starts_with("_") {
        demangle(&input[1..])
    } else {
        input.to_owned()
    }
}

fn demangle(input: &str) -> String {
    let mut input = input.to_owned();

    input = regex_replace(&input, r"::h[0-9a-f]{16}", "");
    input = regex_replace(&input, r"\+0x[0-9a-f]+", "");

    // Convert special characters
    input = regex_replace(&input, r"\$C\$", r",");
    input = regex_replace(&input, r"\$SP\$", r"@");
    input = regex_replace(&input, r"\$BP\$", r"*");
    input = regex_replace(&input, r"\$RF\$", r"&");
    input = regex_replace(&input, r"\$LT\$", r"<");
    input = regex_replace(&input, r"\$GT\$", r">");
    input = regex_replace(&input, r"\$LP\$", r"(");
    input = regex_replace(&input, r"\$RP\$", r")");
    input = regex_replace(&input, r"\$u20\$", r" ");
    input = regex_replace(&input, r"\$u27\$", r"'");
    input = regex_replace(&input, r"\$u5b\$", r"[");
    input = regex_replace(&input, r"\$u5d\$", r"]");
    input = regex_replace(&input, r"\$u7b\$", r"{");
    input = regex_replace(&input, r"\$u7d\$", r"}");
    input = regex_replace(&input, r"\$u7e\$", "~");

    // Fix . and _
    input = regex_replace(&input, r"\.\.", "::");
    input = regex_replace(&input, r"[^\.]\.[^\.]", ".");
    input = regex_replace(&input, r"([;:])_", "$1");

    return input;
}

main!(|| {
    let script = r#"
    
#pragma D option quiet
profile:::profile-1000hz
/pid == $target/
{
    @pc[arg1] = count();
}
dtrace:::END
{
    printa("[[entry]]\ncount=%@d\nvalue='''%A'''\n\n", @pc);
}

"#;

    let probe = dtrace_probe("./target/release/mercutio-wasm-proxy", script)?;

    // Once the user hits Ctrl+C, instruct dtrace to dump its output.
    await_ctrlc();
    probe.stop()?;

    // Process the frames.
    // println!();
    // println!("<<<STARTING BELOW>>>>");
    // println!();

    let mut frames_cache = HashMap::<_, Frame>::new();
    let mut total = 0;
    for toml in probe {
        if let Some(frame) = toml_to_frame(&toml) {
            if frame.target == "0x0" {
                continue;
            }

            // Add total so we can count percentages.
            total += frame.count;

            // Add to cache
            let id = (frame.library.clone(), frame.target.clone());
            if let Some(existing_frame) = frames_cache.get_mut(&id) {
                existing_frame.count += frame.count;
            } else {
                frames_cache.insert(id, frame);
            }
        }
    }

    let mut frames = frames_cache.values().collect::<Vec<_>>();
    frames.sort_by(|a, b| a.count.cmp(&b.count));

    for item in &frames {
        let pct = ((item.count as f64) * 100f64) / (total as f64);
        let lib = item.library.clone().unwrap_or("???".to_string());
        println!(
            "{:>10.2} {:>6} {:>25.25}  {}",
            pct,
            item.count,
            lib,
            demangle_maybe(&item.target),
        );
    }

    eprintln!();
    eprintln!("done.");
});
