#![allow(unused_imports)]

use crossbeam_channel;

use crate::{
    button_handlers,
    ClientController,
    Task,
};

use self::crossbeam_channel::Sender;
use edit_common::commands::*;
use serde_json;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wbg_rand::Rng;







#[cfg(target_arch = "wasm32")]
use crate::wasm::WasmClientController;

#[cfg(target_arch = "wasm32")]
pub struct Scheduler {
    controller: WasmClientController,
    alive: Arc<AtomicBool>,
    monkey: Arc<AtomicBool>,
}


#[cfg(target_arch = "wasm32")]
impl Scheduler {
    pub fn new(
        controller: WasmClientController,
        alive: Arc<AtomicBool>,
        monkey: Arc<AtomicBool>,
    ) -> Self {
        Self {
            controller,
            alive,
            monkey,
        }
    }

    pub fn schedule_random<F>(&mut self, bounds: (u64, u64), task: F)
    where
        F: Fn() -> ControllerCommand + 'static,
    {
        use crate::wasm::setTimeout;

        use ::wbg_rand::{
            wasm_rng,
            Rng,
        };

        let controller = self.controller.clone();
        let alive = self.alive.clone();
        let monkey = self.monkey.clone();

        let task = Rc::new(task);
        let load_it: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));
        let load_it_clone = load_it.clone();
        *load_it.borrow_mut() = Some(Box::new(move || {
            let mut controller = controller.clone();
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
                        let task_str =
                            serde_json::to_string(&Task::ControllerCommand(task_object)).unwrap();

                        controller.command(&task_str);
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
        F: Fn() -> ControllerCommand + 'static + Send,
    {
        use ::failure::Error;
        use ::std::thread;
        use ::std::time::Duration;
        use rand;

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
                    tx.send(Task::ControllerCommand(task_object));
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
    use ::wbg_rand::{
        wasm_rng,
        Rng,
    };
    wasm_rng()
}

#[cfg(not(target_arch = "wasm32"))]
fn local_rng() -> impl Rng {
    use rand;
    rand::thread_rng()
}

#[allow(unused)]
pub fn setup_monkey<C: ClientController + Sized>(client: Rc<RefCell<crate::client::Client>>, mut scheduler: Scheduler) {
    scheduler.schedule_random(MONKEY_BUTTON, || {
        let mut rng = local_rng();
        let index = rng.gen_range(0, button_handlers::<C>(None).0.len() as u32);
        ControllerCommand::Button { button: index }
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
        ControllerCommand::Character { char_code: c }
    });

    // scheduler.schedule_random(MONKEY_ARROW, || {
    //     let mut rng = local_rng();
    //     let key_code = *rng.choose(&[37, 39, 37, 39, 37, 39, 38, 40]).unwrap();
    //     ControllerCommand::Keypress {
    //         key_code,
    //         meta_key: false,
    //         shift_key: false,
    //         alt_key: false,
    //     }
    // });

    scheduler.schedule_random(MONKEY_BACKSPACE, || ControllerCommand::Keypress {
        key_code: 8,
        meta_key: false,
        shift_key: false,
        alt_key: false,
    });

    scheduler.schedule_random(MONKEY_ENTER, || ControllerCommand::Keypress {
        key_code: 13,
        meta_key: false,
        shift_key: false,
        alt_key: false,
    });

    scheduler.schedule_random(MONKEY_CLICK, || {
        let mut rng = local_rng();
        ControllerCommand::RandomTarget {
            position: rng.gen::<f64>(),
        }
    });
}
