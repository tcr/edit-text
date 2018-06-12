#![feature(integer_atomics)]

extern crate fantoccini;
extern crate futures;
extern crate tokio_core;
#[macro_use]
extern crate commandspec;
#[macro_use]
extern crate taken;
extern crate rand;
#[macro_use]
extern crate failure;

use fantoccini::{Client, Locator};
// use futures::prelude::*;
use commandspec::*;
use failure::Error;
use futures::future::{ok, Future};
use rand::thread_rng;
use std::process::Stdio;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

static DRIVER_PORT_COUNTER: AtomicU16 = AtomicU16::new(4445);

fn in_ci() -> bool {
    ::std::env::var("CI")
        .ok()
        .map(|x| x == "true")
        .unwrap_or(false)
}

fn main() {
    commandspec::forward_ctrlc();

    let test_id1 = format!("test{}", random_id());
    let test_id2 = test_id1.clone();

    let both_barrier = Arc::new(Barrier::new(2));
    let seq_barrier = Arc::new(Barrier::new(2));

    let j1 = ::std::thread::spawn({
        take!(=both_barrier, =seq_barrier);
        move || run(&test_id1, both_barrier, Some(seq_barrier))
    });
    let j2 = ::std::thread::spawn({
        take!(=both_barrier, =seq_barrier);
        move || {
            seq_barrier.wait();
            println!("ok...");
            run(&test_id2, both_barrier, None)
        }
    });

    let ret1 = j1.join().unwrap().expect("Program failed:");
    let ret2 = j2.join().unwrap().expect("Program failed:");

    assert!(ret1, "client 1 failed to see ghosts");
    assert!(ret2, "client 2 failed to see ghosts");

    eprintln!("test successful.");
}

fn random_id() -> String {
    let mut rng = thread_rng();
    return ::rand::seq::sample_iter(&mut rng, 0..26u8, 8)
        .unwrap()
        .into_iter()
        .map(|x| (b'a' + x) as char)
        .collect::<String>();
}

#[allow(unused)]
#[derive(Debug)]
enum Driver {
    Chrome,
    Gecko,
}

fn run(
    test_id: &str,
    both_barrier: Arc<Barrier>,
    seq_barrier: Option<Arc<Barrier>>,
) -> Result<bool, Error> {
    // TODO accept port ID and alternative drivers.
    let port = DRIVER_PORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let driver = Driver::Gecko; // TODO do not hardcode this

    println!("---> Connecting to driver {:?} on port {:?}", driver, port);

    let mut cmd = match driver {
        Driver::Chrome => {
            let mut cmd = command!("chromedriver")?;
            cmd.arg(format!("--port={}", port)).arg(port.to_string());
            cmd
        }
        Driver::Gecko => {
            let mut cmd = command!("geckodriver")?;
            cmd.arg("-p").arg(port.to_string());
            cmd
        }
    };

    // Launch child.
    let _webdriver_guard = cmd
        // .stdout(Stdio::inherit())
        // .stderr(Stdio::inherit())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn_guard()?;

    // Wait for webdriver startup.
    ::std::thread::sleep(::std::time::Duration::from_millis(3_000));

    // Connect
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let c = Client::new(&format!("http://0.0.0.0:{}/", port), &core.handle());
    let c = core.run(c).unwrap();

    println!("Connected...");

    let ret_value = {
        // we want to have a reference to c so we can use it in the and_thens below
        let c = &c;

        // now let's set up the sequence of steps we want the browser to take
        // first, go to the Wikipedia page for Foobar
        let f = c
            .goto(&format!("http://0.0.0.0:8000/{}", test_id))
            .and_then(move |_| c.current_url())
            .and_then(move |url| {
                println!("1");
                println!("URL {:?}", url);

                // Wait for page to load
                ::std::thread::sleep(::std::time::Duration::from_millis(2_000));

                // Align threads
                println!("\n\n\n\n\n");
                if let Some(seq_barrier) = seq_barrier {
                    seq_barrier.wait();
                }
                both_barrier.wait();
                println!("Barrier done, continuing...");

                // assert_eq!(url.as_ref(), "https://en.wikipedia.org/wiki/Foobar");
                // click "Foo (disambiguation)"
                c.wait_for_find(Locator::Css(r#"div[data-tag="caret"]"#))
            })
            .and_then(|_| {
                ::std::thread::sleep(::std::time::Duration::from_millis(1_000));

                println!("2");
                c.execute(
                    r#"

let marker = document.querySelector('.edit-text div[data-tag=h1] span');

// marker.style.cssText = `
// background: red;
// width: 10px;
// height: 10px;
// display: inline-block;
// `;
//h1.appendChild(marker);

let clientX = marker.getBoundingClientRect().right;
let clientY = marker.getBoundingClientRect().top;

// h1.removeChild(marker);

var evt = new MouseEvent("mousedown", {
    bubbles: true,
    cancelable: true,
    clientX: clientX - 3,
    clientY: clientY + 3,
});
console.log('x', clientX);
console.log('y', clientY);
document.querySelector('.edit-text').dispatchEvent(evt);

                "#,
                    vec![],
                )
            })
            .and_then(|_| {
                ::std::thread::sleep(::std::time::Duration::from_millis(1_000));

                println!("2a");
                c.execute(
                    r#"

// let charCode = 35;
let charCode = 0x1f47b;
var evt = new KeyboardEvent("keypress", {
    bubbles: true,
    cancelable: true,
    charCode: charCode,
});
document.dispatchEvent(evt);

                "#,
                    vec![],
                )
            })
            .and_then(|_| {
                // Enough time for both clients to sync up.
                ok(::std::thread::sleep(::std::time::Duration::from_millis(
                    4000,
                )))
            })
            .and_then(|_| {
                println!("3");

                c.execute(
                    r#"

let h1 = document.querySelector('.edit-text div[data-tag=h1]');
return h1.innerText;

                "#,
                    vec![],
                )
            })
            .and_then(move |out| {
                println!("4");
                println!("OUT {:?}", out);
                // println!("TITLE {:?}", url);
                // assert_eq!(url.as_ref(), "https://en.wikipedia.org/wiki/Foobar");
                // click "Foo (disambiguation)"
                // c.wait_for_find(Locator::Css(r#"div[data-tag="cccc"]"#))
                // })
                // .and_then(|_e| {
                // assert_eq!(url.as_ref(), "https://en.wikipedia.org/wiki/Foo_Lake");
                Ok(out)
            });

        // and set the browser off to do those things
        core.run(f).unwrap()
    };

    // drop the client to delete the browser session
    if let Some(fin) = c.close() {
        // and wait for cleanup to finish
        core.run(fin).unwrap();
    }

    let h1_string = ret_value.as_string().unwrap();
    eprintln!("done: {:?}", h1_string);

    // drop(child);

    Ok(h1_string.ends_with("👻👻"))
}
