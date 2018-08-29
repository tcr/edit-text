#![allow(unused_imports)]

use crate::{
    button_handlers,
    ClientImpl,
    Task,
};

use extern::crossbeam_channel::Sender;
use edit_common::commands::*;
use serde_json;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;
use wbg_rand::Rng;

#[cfg(target_arch = "wasm32")]
pub struct Scheduler {
    // tx: Sender<Task>,
    alive: Arc<AtomicBool>,
    monkey: Arc<AtomicBool>,
}

#[cfg(target_arch = "wasm32")]
impl Scheduler {
    pub fn new(
        // tx: Sender<Task>,
        alive: Arc<AtomicBool>,
        monkey: Arc<AtomicBool>,
    ) -> Self {
        Self {
            // tx,
            alive,
            monkey,
        }
    }

    pub fn schedule_random<F>(&mut self, bounds: (u64, u64), task: F)
    where
        F: Fn() -> FrontendToUserCommand + 'static,
    {
        use crate::wasm::{
            forwardWasmTask,
            setTimeout,
        };

        use extern::wbg_rand::{
            wasm_rng,
            Rng,
        };

        // let tx = self.tx.clone();
        let alive = self.alive.clone();
        let monkey = self.monkey.clone();

        let task = Rc::new(task);
        let load_it: Rc<RefCell<Option<Box<Fn()>>>> = Rc::new(RefCell::new(None));
        let load_it_clone = load_it.clone();
        *load_it.borrow_mut() = Some(Box::new(move || {
            // let tx = tx.clone();
            let alive = alive.clone();
            let monkey = monkey.clone();
            let task = task.clone();
            let load_it_clone = load_it_clone.clone();

            let outer = Rc::new(RefCell::new(Box::new(None)));

            let mut rng = wasm_rng();
            let delay = rng.gen_range(bounds.0, bounds.1);
            // console_log!(" - new delay: {:?}", delay);
            let inner = {
                let outer = outer.clone();
                Closure::new(move || {
                    (load_it_clone.borrow_mut().as_ref().unwrap())();

                    outer.borrow_mut().take(); // drop it

                    if alive.load(Ordering::Relaxed) && monkey.load(Ordering::Relaxed) {
                        let task_object = task();
                        let task_str = serde_json::to_string(&Task::FrontendToUserCommand(
                            task_object,
                        )).unwrap();
                        forwardWasmTask(&task_str);
                    }
                })
            };

            // TODO not drop inner
            setTimeout(&inner, delay as u32);

            // todo return inner
            **outer.borrow_mut() = Some(inner);
        }));

        (load_it.borrow_mut().as_ref().unwrap())();

        ::std::mem::forget(load_it);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Scheduler {
    tx: Sender<Task>,
    alive: Arc<AtomicBool>,
    monkey: Arc<AtomicBool>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Scheduler {
    pub fn new(tx: Sender<Task>, alive: Arc<AtomicBool>, monkey: Arc<AtomicBool>) -> Self {
        Self { tx, alive, monkey }
    }

    pub fn schedule_random<F>(&mut self, bounds: (u64, u64), task: F)
    where
        F: Fn() -> FrontendToUserCommand + 'static + Send,
    {
        use extern::{
            failure::Error,
            rand,
            std::thread,
            std::time::Duration,
        };

        // Proxy impl
        let tx = self.tx.clone();
        let alive = self.alive.clone();
        let monkey = self.monkey.clone();
        thread::spawn::<_, Result<(), Error>>(move || {
            let mut rng = rand::thread_rng();
            while alive.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(rng.gen_range(bounds.0, bounds.1)));
                if monkey.load(Ordering::Relaxed) {
                    let task_object = task();
                    tx.send(Task::FrontendToUserCommand(task_object))?;
                }
            }
            Ok(())
        });
    }
}

pub type MonkeyParam = (u64, u64);

// "Human-like"
pub const MONKEY_BUTTON: MonkeyParam = (0, 1500);
pub const MONKEY_LETTER: MonkeyParam = (0, 200);
pub const MONKEY_ARROW: MonkeyParam = (0, 500);
pub const MONKEY_BACKSPACE: MonkeyParam = (0, 250);
pub const MONKEY_ENTER: MonkeyParam = (6_000, 10_000);
pub const MONKEY_CLICK: MonkeyParam = (400, 1000);

// Race
// const MONKEY_BUTTON: MonkeyParam = (0, 0, 100);
// const MONKEY_LETTER: MonkeyParam = (0, 0, 100);
// const MONKEY_ARROW: MonkeyParam = (0, 0, 100);
// const MONKEY_BACKSPACE: MonkeyParam = (0, 0, 100);
// const MONKEY_ENTER: MonkeyParam = (0, 0, 1_000);

#[cfg(target_arch = "wasm32")]
fn local_rng() -> impl Rng {
    use extern::wbg_rand::{
        wasm_rng,
        Rng,
    };
    wasm_rng()
}

#[cfg(not(target_arch = "wasm32"))]
fn local_rng() -> impl Rng {
    use extern::rand;
    rand::thread_rng()
}

#[allow(unused)]
pub fn setup_monkey<C: ClientImpl + Sized>(mut scheduler: Scheduler) {
    // let mut scheduler = Scheduler::new(alive, monkey);

    scheduler.schedule_random(MONKEY_BUTTON, || {
        let mut rng = local_rng();
        let index = rng.gen_range(0, button_handlers::<C>(None).0.len() as u32);
        FrontendToUserCommand::Button(index)
    });

    scheduler.schedule_random(MONKEY_LETTER, || {
        let mut rng = local_rng();
        let char_list = vec![
            rng.gen_range(b'A', b'Z'),
            rng.gen_range(b'a', b'z'),
            rng.gen_range(b'0', b'9'),
            b' ',
        ];
        let c = *rng.choose(&char_list).unwrap() as u32;
        FrontendToUserCommand::Character(c)
    });

    scheduler.schedule_random(MONKEY_ARROW, || {
        let mut rng = local_rng();
        let key = *rng.choose(&[37, 39, 37, 39, 37, 39, 38, 40]).unwrap();
        FrontendToUserCommand::Keypress(key, false, false, false)
    });

    scheduler.schedule_random(MONKEY_BACKSPACE, || {
        FrontendToUserCommand::Keypress(8, false, false, false)
    });

    scheduler.schedule_random(MONKEY_ENTER, || {
        FrontendToUserCommand::Keypress(13, false, false, false)
    });

    scheduler.schedule_random(MONKEY_CLICK, || {
        let mut rng = local_rng();
        FrontendToUserCommand::RandomTarget(rng.gen::<f64>())
    });
}
