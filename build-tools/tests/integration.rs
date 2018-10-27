// The nightly features that are commonly needed with async / await
#![recursion_limit="128"]
#![feature(await_macro, async_await, futures_api)]

#![feature(integer_atomics)]
#![allow(unused)]

#[macro_use]
extern crate tokio;

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
use taken::*;
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
    client: &'a mut Client,
    value: String,
}

fn code<'a>(client: &'a mut Client) -> JsCode<'a> {
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

    fn execute(self) -> impl Future<Item = serde_json::value::Value, Error = error::CmdError> {
        self.client.execute(&self.value, vec![])
    }

    fn debug_end_of_line(self) -> impl Future<Item = serde_json::value::Value, Error = error::CmdError> {
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

// Sync barrier, optional sequential barrier.
struct Checkpoint(Arc<Barrier>, Option<Arc<Barrier>>);

impl Checkpoint {
    fn sync(self) {
        if let Some(seq_barrier) = self.1 {
            seq_barrier.wait();
        }
        // Then synchronize both clients.
        self.0.wait();
    }

    // // Sequential until next .sync()
    // fn sequential() {
    //      // TODO
    // }
}








async fn with_webdriver<T: ::std::future::Future<Output = U> + Send + 'static, U>(
    callback: impl FnOnce(Client) -> T + Send + 'static,
) -> U {
    // Launch child.
    let (port, _webdriver_guard) = webdriver().unwrap();

    // Wait for webdriver startup.
    await!(sleep_ms(3_000));

    // Connect to the browser driver from Rust.
    let client = await!(Client::new(
        &format!("http://0.0.0.0:{}/", port),
    )).unwrap();

    eprintln!("Connected...");
    await!(callback(client))
}

fn parallel<T: ::std::future::Future<Output = Result<bool, Error>> + Send + 'static>(
    runner_test: fn(Client, String, Checkpoint) -> T,
) {
    commandspec::cleanup_on_ctrlc();

    let test_id1 = format!("test{}", random_id());
    let test_id2 = test_id1.clone();

    let both_barrier = Arc::new(Barrier::new(2));
    let seq_barrier = Arc::new(Barrier::new(2));

    let j1 = ::std::thread::spawn({
        take!(=both_barrier, =seq_barrier);
        move || -> Result<bool, ()> {
            let checkpoint = Checkpoint(both_barrier.clone(), Some(seq_barrier.clone()));
            tokio::run_async(async move {
                await!(with_webdriver(async move |c| {
                    await!(runner_test(c, test_id1.clone(), checkpoint));
                }))
            });
            Ok(true)
        }
    });

    let j2 = ::std::thread::spawn({
        take!(=both_barrier, =seq_barrier);
        move || -> Result<bool, ()> {
            seq_barrier.wait();
            let checkpoint = Checkpoint(both_barrier.clone(), None);
            tokio::run_async(async move {
                await!(with_webdriver(async move |c| {
                    await!(runner_test(c, test_id2.clone(), checkpoint));
                }))
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

fn webdriver() -> Result<(u16, SpawnGuard), Error> {
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
            cmd.env("MOZ_HEADLESS", "1");
            cmd
        }
    };

    // Launch child.
    Ok((port, cmd
        // .stdout(Stdio::inherit())
        // .stderr(Stdio::inherit())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .scoped_spawn()?))
}


// Individual tests

#[cfg(feature = "integration")]
#[test]
fn integration_spooky_test() {
    parallel(async move |
        mut c: Client,
        test_id: String,
        checkpoint: Checkpoint,
    | {
        // Navigate to the test URL.
        let test_url = format!("http://0.0.0.0:8000/{}", test_id);
        c = await!(c.goto(&test_url))?;

        // Wait for the page to load.
        c = await!(c.wait_for_find(Locator::Css(".edit-text")))?.client();

        // Ensure all browsers have loaded before proceeding. Loading
        // can be deferred or load sequentially, but this checkpoint
        // will ensure all browsers are in the same editor state.
        checkpoint.sync();
        eprintln!("Synchronized.");

        // Now wait until carets show up on the page.
        c = await!(c.wait_for_find(Locator::Css(r#"div[data-tag="caret"]"#)))?.client();

        // Position the caret.
        await!(sleep_ms(1_000));
        await!(code(&mut c).debug_end_of_line());

        // Type the ghost character.
        await!(sleep_ms(1_000));
        await!(code(&mut c).keypress("0x1f47b").execute());
        
        // DEBUG.keypress();

        // Wait up 4s for both clients to synchronize.
        await!(sleep_ms(4000));
        
        // Get the innerText of the header element.
        let heading = await!(code(&mut c)
            .js(r#"

// DEBUG.asMarkdown().match(/\S.*$/m);

let h1 = document.querySelector('.edit-text div[data-tag=h1]');
return h1.innerText;

            "#)
            .execute())?
            .to_string();

        eprintln!("done: {:?}", heading);
        Ok(heading.ends_with("👻👻"))
    });
}
