use crate::{
    Client,
    ClientImpl,
    Task,
    button_handlers,
};

use extern::{
    failure::Error,
    std::thread,
    crossbeam_channel::Sender,
    edit_common::commands::*,
    rand::{
        self,
        Rng,
    },
    std::sync::atomic::AtomicBool,
    std::sync::atomic::Ordering,
    std::sync::Arc,
    std::time::Duration,
};

#[cfg(not(target_arch = "wasm32"))]
pub struct ProxyClient {
    pub state: Client,
    pub tx_client: Sender<UserToFrontendCommand>,
    pub tx_sync: Sender<UserToSyncCommand>,
}

#[cfg(not(target_arch = "wasm32"))]
impl ClientImpl for ProxyClient {
    fn state(&mut self) -> &mut Client {
        &mut self.state
    }

    fn send_client(&self, req: &UserToFrontendCommand) -> Result<(), Error> {
        log_wasm!(SendClient(req.clone()));
        self.tx_client.send(req.clone())?;
        Ok(())
    }

    fn send_sync(&self, req: UserToSyncCommand) -> Result<(), Error> {
        log_wasm!(SendSync(req.clone()));
        self.tx_sync.send(req)?;
        Ok(())
    }
}

macro_rules! spawn_monkey_task {
    ($alive:expr, $monkey:expr, $tx:expr, $wait_params:expr, $task:expr) => {{
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
                    tx.send(Task::FrontendToUserCommand($task))?;
                }
            }
            Ok(())
        })
    }};
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
pub fn setup_monkey<C: ClientImpl + Sized>(
    alive: Arc<AtomicBool>,
    monkey: Arc<AtomicBool>,
    tx: Sender<Task>,
) {
    spawn_monkey_task!(alive, monkey, tx, MONKEY_BUTTON, {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0, button_handlers::<C>(None).len() as u32);
        FrontendToUserCommand::Button(index)
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
        FrontendToUserCommand::Character(c)
    });

    spawn_monkey_task!(alive, monkey, tx, MONKEY_ARROW, {
        let mut rng = rand::thread_rng();
        let key = *rng.choose(&[37, 39, 37, 39, 37, 39, 38, 40]).unwrap();
        FrontendToUserCommand::Keypress(key, false, false, false)
    });

    spawn_monkey_task!(alive, monkey, tx, MONKEY_BACKSPACE, {
        FrontendToUserCommand::Keypress(8, false, false, false)
    });

    spawn_monkey_task!(alive, monkey, tx, MONKEY_ENTER, {
        FrontendToUserCommand::Keypress(13, false, false, false)
    });

    spawn_monkey_task!(alive, monkey, tx, MONKEY_CLICK, {
        let mut rng = rand::thread_rng();
        FrontendToUserCommand::RandomTarget(rng.gen::<f64>())
    });
}
