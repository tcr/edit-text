// cargo-deps: rayon="*"

extern crate rayon;

use std::fs::File;
use std::fs;
use std::process::{Stdio, Command};
use std::io::prelude::*;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use rayon::prelude::*;

fn launch(input: &str) -> i32 {
    let mut child = Command::new("/Users/trim/tcr/edit-text/target/release/oatie-transform")
        // .current_dir(dir.path())
        .env("RUST_BACKTRACE", "1")
        // .arg("--port")
        // .arg(sync_port.to_string())
        // .arg("--period")
        // .arg(rnd_period.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    
    {
        let mut stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(input.as_bytes()).expect("Failed to write to stdin");
    }
    
    match child.wait() {
        Ok(status) => {
            // println!("1 status {:?}", status);
            return status.code().unwrap_or(255);
        }
        Err(e) => {
            println!("e {:?}", e);
            return 255;
        }
    }
}

//     let mut child2 = Command::new("/Users/trim/github/edit-text//target/release/edit-wasm")
//         .current_dir(dir.path())
//         .env("RUST_BACKTRACE", "1")
//         .arg("--port")
//         .arg((sync_port + 1).to_string())
//         .arg("--monkies")
//         .arg(rnd_monkies.to_string())
//         .stdin(Stdio::null())
//         .stdout(Stdio::null())
//         .stderr(Stdio::null())
//         .spawn()
//         .unwrap();
    
//     let now = Instant::now();

//     let success = loop {
//         if now.elapsed().as_secs() > 10 {
//             child1.kill();
//             child2.kill();
//             break false;
//         }

//         match child1.try_wait() {
//             Ok(Some(status)) => {
//                 println!("1 status {:?}", status);
//                 child1.kill();
//                 child2.kill();
//                 break true;
//             }
//             Ok(None) => {
//             }
//             Err(e) => {
//                 println!("e {:?}", e);
//             }
//         }

//         match child2.try_wait() {
//             Ok(Some(status)) => {
//                 println!("2 status {:?}", status);
//                 child1.kill();
//                 child2.kill();
//                 break false;
//             }
//             Ok(None) => {
//             }
//             Err(e) => {
//                 println!("e {:?}", e);
//             }
//         }

//         ::std::thread::sleep(::std::time::Duration::from_millis(100));
//     };

//     if success {
//         if let Ok(mut f) = File::open(dir.path().join("test.txt")) {
//             let mut contents = String::new();
//             f.read_to_string(&mut contents);

//             let cur_value = counter.load(Ordering::Relaxed);
//             counter.store(cur_value + 1, Ordering::Relaxed);

//             let mut f = File::create(format!("out/{}", cur_value)).expect("file not created");
//             f.write_all(contents.as_bytes());

//             println!("{} ...next", cur_value);
//         } else {
//             println!("odd, next");
//         }
//     } else {
//         println!("No errors found in time duration");
//     }
// }

#[derive(PartialEq)]
enum Status {
    Success,
    Failed,
    Skipped,
}

fn main() {
    println!("Running tests...");
    println!();

    let mut files = vec![];
    for entry in fs::read_dir("in/").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        // println!("path {:?}", path);
        files.push(path);
    }

    let res = files.par_iter()
        .map(|file| {
            let mut f = File::open(file).unwrap();
            let mut contents = String::new();
            f.read_to_string(&mut contents);
            contents.push_str("\n\n\n");

            // Skip these
            // if contents.find("DelGroupAll").is_some() || contents.find("DelMany").is_some() {
            //     return Status::Skipped;
            // }

            println!("-----> {:?}", file);
            if launch(&contents) == 0 {
                println!("success");
                Status::Success
            } else {
                println!("failed");
                Status::Failed
            }
        })
        .collect::<Vec<_>>();
    
    println!("total ran: {:?}", res.len());
    println!();
    println!("total success: {:?}", res.iter().filter(|x| **x == Status::Success).count());
    println!("total failed: {:?}", res.iter().filter(|x| **x == Status::Failed).count());
    println!();
    println!("total skipped: {:?}", res.iter().filter(|x| **x == Status::Skipped).count());

    // let mut counter = Arc::new(AtomicUsize::new((high as usize) + 1));
    // println!("Start with out/{:?}", counter.load(Ordering::Relaxed));
    // println!();

    // // panic!("done");

    // let THREAD_MAX = 16;

    // let mut obj = vec![];
    // for thread_count in 0..THREAD_MAX {
    //     let counter = counter.clone();
    //     let t = thread::spawn(move || {
    //         launch(counter.clone(), thread_count);
    //     });
    //     obj.push(t);
    // }

    // for i in obj {
    //     i.join();
    // }

    // println!("(done)");

    // println!("files {:?}", files);
}