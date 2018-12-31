mod checkpoint;
mod debug_bindings;

pub use self::{
    checkpoint::*,
    debug_bindings::*,
};
use commandspec::*;
use failure::Error;
use fantoccini::{
    Client,
    Locator,
};
use serde_json::json;
use std::process::Stdio;
use std::sync::Arc;
use std::{
    sync::atomic::{
        AtomicBool,
        AtomicUsize,
        Ordering,
    },
    thread::JoinHandle,
};
use taken::*;

static DRIVER_PORT_COUNTER: AtomicUsize = AtomicUsize::new(4445);

#[must_use]
pub async fn sleep_ms(val: u64) -> Result<(), Error> {
    await!(tokio_timer::sleep(::std::time::Duration::from_millis(val)))?;
    Ok(())
}

#[allow(unused)]
#[derive(Debug)]
enum Driver {
    Chrome,
    Gecko,
}

fn launch_webdriver() -> Result<(u16, SpawnGuard), Error> {
    // TODO accept port ID and alternative drivers.
    let port = DRIVER_PORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let driver = Driver::Gecko; // TODO do not hardcode this

    println!("---> Connecting to driver {:?} on port {:?}", driver, port);

    let mut cmd = match driver {
        Driver::Chrome => {
            let mut cmd = command!("chromedriver")?;
            cmd.arg(format!("--port={}", port));
            cmd
        }
        Driver::Gecko => {
            let mut cmd = command!("geckodriver")?;
            cmd.arg("-p").arg(port.to_string());
            cmd
        }
    };

    cmd.stdout(Stdio::null()).stderr(Stdio::null());

    // Launch child.
    Ok((
        port as u16,
        cmd
            // .stdout(Stdio::inherit())
            // .stderr(Stdio::inherit())
            .scoped_spawn()?,
    ))
}

async fn with_webdriver<T, U>(callback: impl FnOnce(Client) -> T + Send + 'static) -> U
where
    T: std::future::Future<Output = U> + Send + 'static,
{
    // Launch child.
    let (port, _webdriver_guard) = launch_webdriver().unwrap();

    // Wait for webdriver startup.
    await!(sleep_ms(3_000));

    // Connect to the browser driver from Rust.
    let client = await!(Client::with_capabilities(
        &format!("http://0.0.0.0:{}/", port),
        json!({
            "moz:firefoxOptions": {"args": ["--headless"]},
            "goog:chromeOptions": {"args": ["--headless"]}, // TODO this doesn't seem to work.
        })
        .as_object()
        .unwrap()
        .to_owned(),
    ))
    .unwrap();

    eprintln!("Connected...");
    await!(callback(client))
}

async fn synchronize_clients(
    mut c: Client,
    test_id: String,
    mut checkpoint: Checkpoint,
) -> Result<(Client, Checkpoint), Error> {
    // We enter this function with threads running sequentially.

    // Navigate to the test URL and wait for the page to load.
    c = await!(c.goto(&test_id))?;
    c = await!(c.wait_for_find(Locator::Css(r#".edit-text"#)))?.client();

    // Ensure all browsers have reached this step before proceeding.
    // From here on out, the threads run in parallel.
    checkpoint.sync();

    // Wait until controls and carets are rendered.
    c = await!(c.wait_for_find(Locator::Css(r#"div[data-tag="caret"]"#)))?.client();
    c = await!(c.wait_for_find(Locator::Css(r#"#native-buttons .menu-buttongroup"#)))?.client();

    // Ensure all browsers have reached this step before proceeding.
    checkpoint.sync();

    eprintln!("starting test: {:?}", test_id);
    Ok((c, checkpoint))
}

fn spawn_test_thread<T>(
    test_id: String,
    mut checkpoint: Checkpoint,
    runner_test: fn(DebugClient, String, Checkpoint) -> T,
) -> JoinHandle<Result<bool, ()>>
where
    T: std::future::Future<Output = Result<bool, Error>> + Send + 'static,
{
    std::thread::spawn(move || -> Result<bool, ()> {
        // This value gets shared with the tokio::run_async closure, which is
        // run sequentially, and then we can read the value out after it's done.
        // This is just a workaround for not being able to return values from
        // tokio::run_async directly (for some reason)
        let result = Arc::new(AtomicBool::new(false));
        tokio::run_async({
            take!(=result);
            async move {
                checkpoint.sequential();
                result.store(
                    await!(with_webdriver(async move |mut c| {
                        let (c, checkpoint) =
                            await!(synchronize_clients(c, test_id.clone(), checkpoint,))?;
                        let mut debug = DebugClient::from(c);
                        await!(runner_test(debug, test_id.clone(), checkpoint))
                    }))
                    .unwrap(),
                    Ordering::Relaxed,
                );
            }
        });
        Ok(result.load(Ordering::Relaxed))
    })
}

pub fn individual_editing<T>(markdown: &str, runner_test: fn(DebugClient, String, Checkpoint) -> T)
where
    T: std::future::Future<Output = Result<bool, Error>> + Send + 'static,
{
    commandspec::cleanup_on_ctrlc();

    // Make an HTTP request to load the document.
    let client = reqwest::Client::new();
    let response = client
        .get("http://0.0.0.0:8000/")
        .query(&[("from", markdown)])
        .send()
        .unwrap();
    let target_url = response.url().to_string();

    let j1 = spawn_test_thread(
        target_url.clone(),
        Checkpoint::generate(1).remove(0),
        runner_test,
    );

    let ret1 = j1.join().unwrap().expect("Program failed:");

    assert!(ret1, "client 1 failed test");

    eprintln!("test successful.");
}

pub fn concurrent_editing<T>(markdown: &str, runner_test: fn(DebugClient, String, Checkpoint) -> T)
where
    T: std::future::Future<Output = Result<bool, Error>> + Send + 'static,
{
    commandspec::cleanup_on_ctrlc();

    // Make an HTTP request to load the document.
    let client = reqwest::Client::new();
    let response = client
        .get("http://0.0.0.0:8000/")
        .query(&[("from", markdown)])
        .send()
        .unwrap();
    let target_url = response.url().to_string();

    let (checkpoint1, checkpoint2) = Checkpoint::new_pair();

    let j1 = spawn_test_thread(target_url.clone(), checkpoint1, runner_test);
    let j2 = spawn_test_thread(target_url.clone(), checkpoint2, runner_test);

    let ret1 = j1.join().unwrap().expect("Program failed:");
    let ret2 = j2.join().unwrap().expect("Program failed:");

    assert!(ret1, "client 1 failed test");
    assert!(ret2, "client 2 failed test");

    eprintln!("test successful.");
}
