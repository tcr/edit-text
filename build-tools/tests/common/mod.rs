// rustfmt-edition: 2018

mod checkpoint;
mod debug;

pub use self::{
    checkpoint::*,
    debug::*,
};
use fantoccini::{
    Client,
    Locator,
};
use commandspec::*;
use failure::Error;
use rand::thread_rng;
use std::process::Stdio;
use std::{
    sync::atomic::{
        AtomicBool,
        AtomicUsize,
        Ordering,
    },
    thread::JoinHandle,
};
use std::sync::{
    Arc,
};
use taken::*;

static DRIVER_PORT_COUNTER: AtomicUsize = AtomicUsize::new(4445);

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

fn launch_webdriver() -> Result<(u16, SpawnGuard), Error> {
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
        port as u16,
        cmd
            // .stdout(Stdio::inherit())
            // .stderr(Stdio::inherit())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
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
    let client = await!(Client::new(&format!("http://0.0.0.0:{}/", port),)).unwrap();

    eprintln!("Connected...");
    await!(callback(client))
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

    // Ensure all browsers have reached this step before proceeding.
    checkpoint.sync();

    // Wait until carets are rendered.
    c = await!(c.wait_for_find(Locator::Css(r#"div[data-tag="caret"]"#)))?.client();

    // Notify user.
    eprintln!("starting test: {:?}", test_id);

    Ok(c)
}

fn spawn_test_thread<T>(
    test_id: String,
    checkpoint: Checkpoint,
    runner_test: fn(DebugClient, String, Checkpoint) -> T,
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
                        let mut debug = DebugClient::from(c);
                        await!(runner_test(debug, test_id.clone(), checkpoint))
                    }))
                    .unwrap();
                result.store(success, Ordering::Relaxed);
            });
        }
        Ok(result.load(Ordering::Relaxed))
    })
}

pub fn concurrent_editing<T>(
    runner_test: fn(DebugClient, String, Checkpoint) -> T,
)
where
    T: std::future::Future<Output = Result<bool, Error>> + Send + 'static,
{
    commandspec::cleanup_on_ctrlc();

    let test_id1 = format!("test{}", random_id());
    let test_id2 = test_id1.clone();

    let (checkpoint1, checkpoint2) = Checkpoint::new_pair();

    let j1 = spawn_test_thread(test_id1, checkpoint1, runner_test);
    let j2 = spawn_test_thread(test_id2, checkpoint2, runner_test);

    let ret1 = j1.join().unwrap().expect("Program failed:");
    let ret2 = j2.join().unwrap().expect("Program failed:");

    assert!(ret1, "client 1 failed test");
    assert!(ret2, "client 2 failed test");

    eprintln!("test successful.");
}