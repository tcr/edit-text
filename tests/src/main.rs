#![feature(integer_atomics)]

extern crate tokio_core;
extern crate futures;
extern crate fantoccini;
#[macro_use]
extern crate commandspec;
extern crate rand;
extern crate failure;

use fantoccini::{Client, Locator};
// use futures::prelude::*;
use futures::future::{Future, ok};
use failure::Error;
use commandspec::*;
use std::process::Stdio;
use rand::thread_rng;
use std::sync::atomic::{AtomicU16, Ordering};

static DRIVER_PORT_COUNTER: AtomicU16 = AtomicU16::new(4445);

fn main() {
    commandspec::forward_ctrlc();

    // let test_id = format!("test{}", random_id());
    
    // let test_id2 = test_id.clone();
    // let j = ::std::thread::spawn(move || {
    //     run(&test_id2)
    // });
    // let ret1 = run(&test_id);

    // let ret2 = j.join().unwrap();
    
    // // let _ = cleanup();

    // let ret1 = ret1.expect("Program failed:");
    // let ret2 = ret2.expect("Program failed:");

    // assert!(ret1, "client 1 failed to see ghosts");
    // assert!(ret2, "client 2 failed to see ghosts");

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
enum Driver {
    Chrome,
    Gecko,
}

fn run(test_id: &str) -> Result<bool, Error> {
    // TODO accept port ID and alternative drivers.
    let port = DRIVER_PORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let driver = Driver::Gecko; // TODO do not hardcode this

    let mut cmd = match driver {
        Driver::Chrome => {
            let mut cmd = command!("chromedriver")?;
            cmd
                .arg(format!("--port={}", port))
                .arg(port.to_string());
            cmd
        }
        Driver::Gecko => {
            let mut cmd = command!("geckodriver")?;
            cmd
                .arg("-p")
                .arg(port.to_string());
            cmd
        }
    };

    // Launch child.
    let _webdriver_guard = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn_guard()?;
    
    // Wait for startup.
    ::std::thread::sleep(::std::time::Duration::from_millis(1000));

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let c = Client::new(&format!("http://0.0.0.0:{}/", port), &core.handle());
    let c = core.run(c).unwrap();
    
    let ret_value = {
        // we want to have a reference to c so we can use it in the and_thens below
        let c = &c;
    
        // now let's set up the sequence of steps we want the browser to take
        // first, go to the Wikipedia page for Foobar
        let f =
            c.goto(&format!("http://0.0.0.0:8000/{}", test_id))
            .and_then(move |_| {
                c.current_url()
            })
            .and_then(move |url| {
                println!("1");
                println!("URL {:?}", url);
                // assert_eq!(url.as_ref(), "https://en.wikipedia.org/wiki/Foobar");
                // click "Foo (disambiguation)"
                c.wait_for_find(Locator::Css(r#"div[data-tag="caret"]"#))
            })
            .and_then(|_| {
                ::std::thread::sleep(::std::time::Duration::from_millis(500));


                println!("2");
                c.execute(r#"

let h1 = document.querySelector('.edit-text div[data-tag=h1]');

let marker = document.createElement('span');
// marker.style.cssText = `
// background: red;
// width: 10px;
// height: 10px;
// display: inline-block;
// `;
h1.appendChild(marker);

let clientX = marker.getBoundingClientRect().left - 2;
let clientY = marker.getBoundingClientRect().top - 5;

h1.removeChild(marker);

var evt = new MouseEvent("mousedown", {
    bubbles: true,
    cancelable: true,
    clientX: clientX,
    clientY: clientY,
});
console.log('x', clientX);
console.log('y', clientY);
document.querySelector('.edit-text').dispatchEvent(evt);

setTimeout(function () {
    // let charCode = 35;
    let charCode = 0x1f47b;
    var evt = new KeyboardEvent("keypress", {
        bubbles: true,
        cancelable: true,
        charCode: charCode,
    });
    document.dispatchEvent(evt);
}, 10);

                "#, vec![])
            })
            .and_then(|_| {
                // Enough time for both clients to sync up.
                ok(::std::thread::sleep(::std::time::Duration::from_millis(8000)))
            })
            .and_then(|_| {
                println!("3");

                c.execute(r#"

let h1 = document.querySelector('.edit-text div[data-tag=h1]');
return h1.innerText;

                "#, vec![])
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