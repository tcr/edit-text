use super::*;
use crossbeam_channel::{Sender};
use failure::Error;
use rand;
use rand::{Rng};
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use super::client::button_handlers;

macro_rules! spawn_monkey_task {
    ( $alive:expr, $monkey:expr, $tx:expr, $wait_params:expr, $task:expr ) => {
        {
            let tx = $tx.clone();
            let alive = $alive.clone();
            let monkey = $monkey.clone();
            thread::spawn::<_, Result<(), Error>>(move || {
                let mut rng = rand::thread_rng();
                while alive.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(
                        rng.gen_range($wait_params.0, $wait_params.1),
                    ));
                    if monkey.load(Ordering::Relaxed) {
                        tx.send(Task::NativeCommand($task))?;
                    }
                }
                Ok(())
            })
        }
    };
}

pub type MonkeyParam = (u64, u64);

// "Human-like"
pub const MONKEY_BUTTON: MonkeyParam = (0, 1500);
pub const MONKEY_LETTER: MonkeyParam = (0, 200);
pub const MONKEY_ARROW: MonkeyParam = (0, 500);
pub const MONKEY_BACKSPACE: MonkeyParam = (0, 300);
pub const MONKEY_ENTER: MonkeyParam = (6_000, 10_000);
pub const MONKEY_CLICK: MonkeyParam = (400, 1000);

// Race
// const MONKEY_BUTTON: MonkeyParam = (0, 0, 100);
// const MONKEY_LETTER: MonkeyParam = (0, 0, 100);
// const MONKEY_ARROW: MonkeyParam = (0, 0, 100);
// const MONKEY_BACKSPACE: MonkeyParam = (0, 0, 100);
// const MONKEY_ENTER: MonkeyParam = (0, 0, 1_000);

#[allow(unused)]
pub fn setup_monkey<C: ClientImpl + Sized>(alive: Arc<AtomicBool>, monkey: Arc<AtomicBool>, tx: Sender<Task>) {

    spawn_monkey_task!(alive, monkey, tx, MONKEY_BUTTON, {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0, button_handlers::<C>().len() as u32);
        NativeCommand::Button(index)
    });

    spawn_monkey_task!(alive, monkey, tx, MONKEY_LETTER, {
        let mut rng = rand::thread_rng();
        let char_list = vec![
            rng.gen_range(b'A', b'Z'),
            rng.gen_range(b'a', b'z'),
            rng.gen_range(b'0', b'9'),
            b' ',
        ];
        let c = *rng.choose(&char_list).unwrap() as u32;
        NativeCommand::Character(c)
    });

    spawn_monkey_task!(alive, monkey, tx, MONKEY_ARROW, {
        let mut rng = rand::thread_rng();
        let key = *rng.choose(&[37, 39, 37, 39, 37, 39, 38, 40]).unwrap();
        NativeCommand::Keypress(key, false, false)
    });

    spawn_monkey_task!(alive, monkey, tx, MONKEY_BACKSPACE, {
        NativeCommand::Keypress(8, false, false)
    });

    spawn_monkey_task!(alive, monkey, tx, MONKEY_ENTER, {
        NativeCommand::Keypress(13, false, false)
    });

    spawn_monkey_task!(alive, monkey, tx, MONKEY_CLICK, {
        let mut rng = rand::thread_rng();
        NativeCommand::RandomTarget(rng.gen::<f64>())
    });
}
