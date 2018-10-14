// The nightly features that are commonly needed with async / await
#![recursion_limit="128"]
#![feature(await_macro, async_await, futures_api)]
#![feature(integer_atomics)]
#![allow(unused)]

extern crate fantoccini;
extern crate futures;
#[macro_use]
extern crate commandspec;
#[macro_use]
extern crate taken;
extern crate rand;
#[macro_use]
extern crate failure;
extern crate rustc_serialize;
#[macro_use]
extern crate tokio;
extern crate tokio_timer;

use fantoccini::{
    Client,
    Locator,
    error,
};
// use futures::prelude::*;
use commandspec::*;
use failure::Error;
use futures::future::{
    ok,
    Future,
};
use rand::thread_rng;
use std::process::Stdio;
use std::sync::atomic::{
    AtomicU16,
    Ordering,
};
use std::sync::{
    Arc,
    Barrier,
};

static DRIVER_PORT_COUNTER: AtomicU16 = AtomicU16::new(4445);

#[must_use]
async fn sleep_ms(val: u64) -> Result<(), Error> {
    await!(tokio_timer::sleep(::std::time::Duration::from_millis(val)))?;
    Ok(())
}

fn in_ci() -> bool {
    ::std::env::var("CI")
        .ok()
        .map(|x| x == "true")
        .unwrap_or(false)
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


struct JsCode<'a> {
    client: &'a Client,
    value: String,
}

fn code<'a>(client: &'a Client) -> JsCode<'a> {
    JsCode {
        client: client,
        value: "".to_string(),
    }
}

impl<'a> JsCode<'a> {
    fn js(mut self, input: &str) -> JsCode<'a> {
        self.value.push_str(input);
        self
    }

    fn keypress(self, key: &str) -> JsCode<'a> {
        self.js(&format!(r#"
var event = new KeyboardEvent("keypress", {{
    bubbles: true,
    cancelable: true,
    charCode: {},
}});
document.dispatchEvent(event);
            "#, key))
    }

    fn mousedown(self, x: &str, y: &str) -> JsCode<'a> {
        self.js(&format!(r#"
var evt = new MouseEvent("mousedown", {{
    bubbles: true,
    cancelable: true,
    clientX: {},
    clientY: {},
}});
document.querySelector('.edit-text').dispatchEvent(evt);
            "#, x, y))
    }

    fn execute(self) -> impl Future<Item = ::rustc_serialize::json::Json, Error = error::CmdError> {
        self.client.execute(&self.value, vec![])
    }

    fn debug_end_of_line(self) -> impl Future<Item = ::rustc_serialize::json::Json, Error = error::CmdError> {
        self
                .js(r#"

// DEBUG.endOfLine();

let marker = document.querySelector('.edit-text div[data-tag=h1] span');
let clientX = marker.getBoundingClientRect().right;
let clientY = marker.getBoundingClientRect().top;


            "#)
                .mousedown("clientX - 3", "clientY + 3")
                .execute()
    }
}

struct Checkpoint(Arc<Barrier>, Option<Arc<Barrier>>);

impl Checkpoint {
    fn sync(self) {
        if let Some(seq_barrier) = self.1 {
            seq_barrier.wait();
        }
        // Then synchronize both clients.
        self.0.wait();
    }
}



fn main() {
    commandspec::forward_ctrlc();

    let test_id1 = format!("test{}", random_id());
    let test_id2 = test_id1.clone();

    let both_barrier = Arc::new(Barrier::new(2));
    let seq_barrier = Arc::new(Barrier::new(2));

    let j1 = ::std::thread::spawn({
        take!(=both_barrier, =seq_barrier);
        move || -> Result<bool, ()> {
            tokio::run_async(async move {
                await!(bootstrap(&test_id1, Checkpoint(both_barrier, Some(seq_barrier))));
            });
            Ok(true)
        }
    });
    let j2 = ::std::thread::spawn({
        take!(=both_barrier, =seq_barrier);
        move || -> Result<bool, ()> {
            seq_barrier.wait();
            println!("ok...");
            tokio::run_async(async move {
                await!(bootstrap(&test_id2, Checkpoint(both_barrier, None)));
            });
            Ok(true)
        }
    });

    let ret1 = j1.join().unwrap().expect("Program failed:");
    let ret2 = j2.join().unwrap().expect("Program failed:");

    assert!(ret1, "client 1 failed to see ghosts");
    assert!(ret2, "client 2 failed to see ghosts");

    eprintln!("test successful.");
}

async fn bootstrap(
    test_id: &str,
    checkpoint: Checkpoint,
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
    await!(sleep_ms(3_000));

    // Connect to the browser driver from Rust.
    // TODO Pass in the current executor from the current runtime
    // instead of creating one here.
    let mut core = tokio::runtime::Runtime::new().unwrap();
    let client = await!(Client::new(
        &format!("http://0.0.0.0:{}/", port),
        core.executor(),
    ))?;

    eprintln!("Connected...");

    await!(spooky_test(client, test_id.to_owned(), checkpoint))
}




// Individual tests

async fn spooky_test<'a>(
    c: Client,
    test_id: String,
    checkpoint: Checkpoint,
) -> Result<bool, Error> {
    // Navigate to the test URL.
    let test_url = format!("http://0.0.0.0:8000/{}", test_id);
    await!(c.goto(&test_url));

    // Wait for the page to load.
    await!(c.wait_for_find(Locator::Css(".edit-text")));

    // Ensure all browsers have loaded before proceeding. Loading
    // can be deferred or load sequentially, but this checkpoint
    // will ensure all browsers are in the same editor state.
    checkpoint.sync();
    eprintln!("Synchronized.");

    // Now wait until carets show up on the page.
    await!(c.wait_for_find(Locator::Css(r#"div[data-tag="caret"]"#)));

    // Position the caret.
    await!(sleep_ms(1_000));
    await!(code(&c).debug_end_of_line());

    // Type the ghost character.
    await!(sleep_ms(1_000));
    await!(code(&c).keypress("0x1f47b").execute());
    
    // DEBUG.keypress();

    // Wait up 4s for both clients to synchronize.
    await!(sleep_ms(4000));
    
    // Get the innerText of the header element.
    let heading = await!(code(&c)
        .js(r#"
    
    // DEBUG.asMarkdown().match(/\S.*$/m);

let h1 = document.querySelector('.edit-text div[data-tag=h1]');
return h1.innerText;

        "#)
        .execute())?
        .as_string()
        .unwrap()
        .to_owned();

    eprintln!("done: {:?}", heading);
    Ok(heading.ends_with("👻👻"))
}
