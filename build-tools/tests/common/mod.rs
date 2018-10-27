// rustfmt-edition: 2018

use fantoccini::{
    error,
    Client,
    Locator,
};
use commandspec::*;
use failure::Error;
use futures::future::{
    Future,
};
use rand::thread_rng;
use std::process::Stdio;
use std::{
    sync::atomic::{
        AtomicU16,
        Ordering,
    },
    thread::JoinHandle,
};
use std::sync::{
    atomic::AtomicBool,
    Arc,
    Barrier,
};
use taken::*;

static DRIVER_PORT_COUNTER: AtomicU16 = AtomicU16::new(4445);

#[must_use]
pub async fn sleep_ms(val: u64) -> Result<(), Error> {
    await!(tokio_timer::sleep(::std::time::Duration::from_millis(val)))?;
    Ok(())
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

pub struct JsCode<'a> {
    client: &'a mut Client,
    value: String,
}

pub fn code<'a>(client: &'a mut Client) -> JsCode<'a> {
    JsCode {
        client: client,
        value: "".to_string(),
    }
}

impl<'a> JsCode<'a> {
    pub fn js(mut self, input: &str) -> JsCode<'a> {
        self.value.push_str(input);
        self
    }

    pub fn keypress(self, key: &str) -> JsCode<'a> {
        self.js(&format!(
            r#"
var event = new KeyboardEvent("keypress", {{
    bubbles: true,
    cancelable: true,
    charCode: {},
}});
document.dispatchEvent(event);
            "#,
            key
        ))
    }

    pub fn mousedown(self, x: &str, y: &str) -> JsCode<'a> {
        self.js(&format!(
            r#"
var evt = new MouseEvent("mousedown", {{
    bubbles: true,
    cancelable: true,
    clientX: {},
    clientY: {},
}});
document.querySelector('.edit-text').dispatchEvent(evt);
            "#,
            x, y
        ))
    }

    pub fn execute(self) -> impl Future<Item = serde_json::value::Value, Error = error::CmdError> {
        self.client.execute(&self.value, vec![])
    }

    pub fn debug_end_of_line(
        self,
    ) -> impl Future<Item = serde_json::value::Value, Error = error::CmdError> {
        self.js(r#"

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
#[derive(Clone)]
pub struct Checkpoint(Arc<Barrier>, (Arc<Barrier>, bool));

impl Checkpoint {
    pub fn sync(&self) {
        if !(self.1).1 {
            (self.1).0.wait();
        }
        // Then synchronize both clients.
        self.0.wait();
    }

    // Sequential until next .sync()
    pub fn sequential(&self) {
        if (self.1).1 {
            (self.1).0.wait();
        }
    }
}

async fn synchronize_clients(
    mut c: Client,
    test_id: String,
    checkpoint: Checkpoint,
) -> Result<Client, Error> {
    // Navigate to the test URL and wait for the page to load.
    let test_url = format!("http://0.0.0.0:8000/{}", test_id);
    c = await!(c.goto(&test_url))?;
    c = await!(c.wait_for_find(Locator::Css(".edit-text")))?.client();

    // Ensure all browsers have loaded before proceeding. Loading
    // can be deferred or load sequentially, but this checkpoint
    // will ensure all browsers are in the same editor state.
    checkpoint.sync();
    eprintln!("[{}] synchronized.", test_id);

    // Wait until carets are rendered.
    c = await!(c.wait_for_find(Locator::Css(r#"div[data-tag="caret"]"#)))?.client();

    Ok(c)
}

async fn with_webdriver<T, U>(callback: impl FnOnce(Client) -> T + Send + 'static) -> U
where
    T: std::future::Future<Output = U> + Send + 'static,
{
    // Launch child.
    let (port, _webdriver_guard) = webdriver().unwrap();

    // Wait for webdriver startup.
    await!(sleep_ms(3_000));

    // Connect to the browser driver from Rust.
    let client = await!(Client::new(&format!("http://0.0.0.0:{}/", port),)).unwrap();

    eprintln!("Connected...");
    await!(callback(client))
}


fn spawn_test_thread<T>(
    test_id: String,
    checkpoint: Checkpoint,
    runner_test: fn(Client, String, Checkpoint) -> T,
) -> JoinHandle<Result<bool, ()>>
where
    T: std::future::Future<Output = Result<bool, Error>> + Send + 'static,
{
    std::thread::spawn(move || -> Result<bool, ()> {
        checkpoint.sequential();
        let result = Arc::new(AtomicBool::new(false));
        {
            take!(=result);
            tokio::run_async(async move {
                take!(=result);
                let success =
                    await!(with_webdriver(async move |mut c| {
                        c = await!(synchronize_clients(
                            c,
                            test_id.clone(),
                            checkpoint.clone()
                        ))?;
                        await!(runner_test(c, test_id.clone(), checkpoint))
                    }))
                    .unwrap();
                result.store(success, Ordering::Relaxed);
            });
        }
        Ok(result.load(Ordering::Relaxed))
    })
}

pub fn concurrent_editing<T>(runner_test: fn(Client, String, Checkpoint) -> T)
where
    T: std::future::Future<Output = Result<bool, Error>> + Send + 'static,
{
    commandspec::cleanup_on_ctrlc();

    let test_id1 = format!("test{}", random_id());
    let test_id2 = test_id1.clone();

    let both_barrier = Arc::new(Barrier::new(2));
    let seq_barrier = Arc::new(Barrier::new(2));

    let checkpoint1 = Checkpoint(both_barrier.clone(), (seq_barrier.clone(), false));
    let checkpoint2 = Checkpoint(both_barrier.clone(), (seq_barrier.clone(), true));

    let j1 = spawn_test_thread(test_id1, checkpoint1, runner_test);
    let j2 = spawn_test_thread(test_id2, checkpoint2, runner_test);

    let ret1 = j1.join().unwrap().expect("Program failed:");
    let ret2 = j2.join().unwrap().expect("Program failed:");

    assert!(ret1, "client 1 failed test");
    assert!(ret2, "client 2 failed test");

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
    Ok((
        port,
        cmd
            // .stdout(Stdio::inherit())
            // .stderr(Stdio::inherit())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .scoped_spawn()?,
    ))
}