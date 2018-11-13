// From https://raw.githubusercontent.com/watchexec/watchexec/master/src/signal.rs
#![allow(unused)]

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

lazy_static! {
    static ref CLEANUP: Mutex<Option<Box<Fn(self::Signal) + Send>>> = Mutex::new(None);
}

#[cfg(unix)]
pub use nix::sys::signal::Signal;

#[cfg(windows)]
use winapi;

// This is a dummy enum for Windows
#[cfg(windows)]
#[derive(Debug, Copy, Clone)]
pub enum Signal {
    SIGKILL,
    SIGTERM,
    SIGINT,
    SIGHUP,
    SIGSTOP,
    SIGCONT,
    SIGCHLD,
    SIGUSR1,
    SIGUSR2,
}

#[cfg(unix)]
use nix::libc::*;

#[cfg(unix)]
pub trait ConvertToLibc {
    fn convert_to_libc(self) -> c_int;
}

#[cfg(unix)]
impl ConvertToLibc for Signal {
    fn convert_to_libc(self) -> c_int {
        // Convert from signal::Signal enum to libc::* c_int constants
        match self {
            Signal::SIGKILL => SIGKILL,
            Signal::SIGTERM => SIGTERM,
            Signal::SIGINT => SIGINT,
            Signal::SIGHUP => SIGHUP,
            Signal::SIGSTOP => SIGSTOP,
            Signal::SIGCONT => SIGCONT,
            Signal::SIGCHLD => SIGCHLD,
            Signal::SIGUSR1 => SIGUSR1,
            Signal::SIGUSR2 => SIGUSR2,
            _ => panic!("unsupported signal: {:?}", self),
        }
    }
}

pub fn new(signal_name: Option<String>) -> Option<Signal> {
    if let Some(signame) = signal_name {
        let signal = match signame.as_ref() {
            "SIGKILL" | "KILL" => Signal::SIGKILL,
            "SIGTERM" | "TERM" => Signal::SIGTERM,
            "SIGINT" | "INT" => Signal::SIGINT,
            "SIGHUP" | "HUP" => Signal::SIGHUP,
            "SIGSTOP" | "STOP" => Signal::SIGSTOP,
            "SIGCONT" | "CONT" => Signal::SIGCONT,
            "SIGCHLD" | "CHLD" => Signal::SIGCHLD,
            "SIGUSR1" | "USR1" => Signal::SIGUSR1,
            "SIGUSR2" | "USR2" => Signal::SIGUSR2,
            _ => panic!("unsupported signal: {}", signame),
        };

        Some(signal)
    } else {
        None
    }
}

static GLOBAL_HANDLER_ID: AtomicUsize = ATOMIC_USIZE_INIT;

#[cfg(unix)]
pub fn uninstall_handler() {
    GLOBAL_HANDLER_ID.fetch_add(1, Ordering::Relaxed) + 1;

    use nix::libc::c_int;
    use nix::sys::signal::*;
    use nix::unistd::Pid;

    // Interrupt self to interrupt handler.
    kill(Pid::this(), Signal::SIGUSR2);
}

#[cfg(unix)]
pub fn install_handler<F>(handler: F)
where
    F: Fn(self::Signal) + 'static + Send + Sync,
{
    use nix::libc::c_int;
    use nix::sys::signal::*;
    use std::thread;

    // Mask all signals interesting to us. The mask propagates
    // to all threads started after this point.
    let mut mask = SigSet::empty();
    mask.add(SIGKILL);
    mask.add(SIGTERM);
    mask.add(SIGINT);
    mask.add(SIGHUP);
    mask.add(SIGSTOP);
    mask.add(SIGCONT);
    mask.add(SIGCHLD);
    mask.add(SIGUSR1);
    mask.add(SIGUSR2);
    mask.thread_swap_mask(SigmaskHow::SIG_SETMASK).expect("unable to set signal mask");

    set_handler(handler);

    // Indicate interest in SIGCHLD by setting a dummy handler
    pub extern "C" fn sigchld_handler(_: c_int) {}

    unsafe {
        let _ = sigaction(
            SIGCHLD,
            &SigAction::new(
                SigHandler::Handler(sigchld_handler),
                SaFlags::empty(),
                SigSet::empty(),
            ),
        );
    }

    // Spawn a thread to catch these signals
    let id = GLOBAL_HANDLER_ID.fetch_add(1, Ordering::Relaxed) + 1;
    thread::spawn(move || {
        let mut is_current = true;
        while is_current {
            let signal = mask.wait().expect("Unable to sigwait");
            debug!("Received {:?}", signal);

            if id != GLOBAL_HANDLER_ID.load(Ordering::Relaxed) {
                return;
            }
            // Invoke closure
            invoke(signal);

            // Restore default behavior for received signal and unmask it
            if signal != SIGCHLD {
                let default_action =
                    SigAction::new(SigHandler::SigDfl, SaFlags::empty(), SigSet::empty());

                unsafe {
                    let _ = sigaction(signal, &default_action);
                }
            }

            let mut new_mask = SigSet::empty();
            new_mask.add(signal);

            // Re-raise with signal unmasked
            let _ = new_mask.thread_unblock();
            let _ = raise(signal);
            let _ = new_mask.thread_block();
        }
    });
}

#[cfg(windows)]
pub fn uninstall_handler() {
    use kernel32::SetConsoleCtrlHandler;
    use winapi::{BOOL, DWORD, FALSE, TRUE};

    unsafe {
        SetConsoleCtrlHandler(Some(ctrl_handler), FALSE);
    }
    debug!("Removed ConsoleCtrlHandler.");
}

#[cfg(windows)]
pub unsafe extern "system" fn ctrl_handler(_: winapi::DWORD) -> winapi::BOOL {
    invoke(self::Signal::SIGTERM);

    winapi::FALSE
}

#[cfg(windows)]
pub fn install_handler<F>(handler: F)
where
    F: Fn(self::Signal) + 'static + Send + Sync,
{
    use kernel32::SetConsoleCtrlHandler;
    use winapi::{BOOL, DWORD, FALSE, TRUE};

    set_handler(handler);

    unsafe {
        SetConsoleCtrlHandler(Some(ctrl_handler), TRUE);
    }
}

fn invoke(sig: self::Signal) {
    if let Some(ref handler) = *CLEANUP.lock().unwrap() {
        handler(sig)
    }
}

fn set_handler<F>(handler: F)
where
    F: Fn(self::Signal) + 'static + Send + Sync,
{
    *CLEANUP.lock().unwrap() = Some(Box::new(handler));
}
