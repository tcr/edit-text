// cargo-deps: tempdir="0.3", rand="0.4"

extern crate tempdir;
extern crate rand;

use std::fs::File;
use std::fs;
use std::process::{Stdio, Command};
use std::io::prelude::*;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};
use rand::{thread_rng, Rng};
use std::sync::Arc;
use std::thread;
use tempdir::TempDir;

fn launch(counter: Arc<AtomicUsize>, thread_count: usize) {
    let dir = TempDir::new("failgen").unwrap();

    let sync_port = 4000 + (thread_count * 10);

    let mut rng = thread_rng();
    let rnd_monkies = rng.gen_range::<usize>(1, 8);
    let rnd_period = rng.gen_range::<usize>(100, 5000);
    println!();
    println!("THREAD: {:?}", thread_count);
    println!("MONIES: {:?}", rnd_monkies);
    println!("PERIOD: {:?}", rnd_period);
    println!();
    println!();

    let mut child1 = Command::new("/Users/trim/tcr/edit-text/target/release/edit-sync")
        .current_dir(dir.path())
        .env("RUST_BACKTRACE", "1")
        .arg("--port")
        .arg(sync_port.to_string())
        .arg("--period")
        .arg(rnd_period.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let mut child2 = Command::new("/Users/trim/tcr/edit-text/target/release/edit-wasm")
        .current_dir(dir.path())
        .env("RUST_BACKTRACE", "1")
        .arg("--port")
        .arg((sync_port + 2).to_string())
        .arg("--monkies")
        .arg(rnd_monkies.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    
    let now = Instant::now();

    let success = loop {
        if now.elapsed().as_secs() > 30 {
            child1.kill();
            child2.kill();
            break false;
        }

        match child1.try_wait() {
            Ok(Some(status)) => {
                println!("1 status {:?}", status);
                child1.kill();
                child2.kill();
                break true;
            }
            Ok(None) => {
            }
            Err(e) => {
                println!("e {:?}", e);
            }
        }

        match child2.try_wait() {
            Ok(Some(status)) => {
                println!("2 status {:?}", status);
                child1.kill();
                child2.kill();
                break false;
            }
            Ok(None) => {
            }
            Err(e) => {
                println!("e {:?}", e);
            }
        }

        ::std::thread::sleep(::std::time::Duration::from_millis(100));
    };

    if success {
        if let Ok(mut f) = File::open(dir.path().join("test.txt")) {
            let mut contents = String::new();
            f.read_to_string(&mut contents);

            let cur_value = counter.load(Ordering::Relaxed);
            counter.store(cur_value + 1, Ordering::Relaxed);

            let mut f = File::create(format!("in/{}", cur_value)).expect("file not created");
            f.write_all(contents.as_bytes());

            println!("{} ...next", cur_value);
        } else {
            println!("failed without creating a test.txt! did the server run successfully?");
        }
    } else {
        println!("No errors found in time duration");
    }
}

fn main() {
    let mut high = 0;
    for entry in fs::read_dir("in/").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        // println!("path {:?}", path);
        let p = path.file_name().unwrap().to_string_lossy().parse::<u32>();
        if let Ok(p) = p {
            if p > high {
                high = p;
            }
        }
    }

    let mut counter = Arc::new(AtomicUsize::new((high as usize) + 1));
    println!("Start with in/{:?}", counter.load(Ordering::Relaxed));
    println!();

    // panic!("done");

    let THREAD_MAX = 16;

    let mut obj = vec![];
    for thread_count in 0..THREAD_MAX {
        let counter = counter.clone();
        let t = thread::spawn(move || {
            launch(counter.clone(), thread_count);
        });
        obj.push(t);
    }

    for i in obj {
        i.join();
    }

    println!("(done)");
}