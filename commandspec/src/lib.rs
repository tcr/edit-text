extern crate shlex;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

#[cfg(windows)]
extern crate kernel32;
#[cfg(unix)]
extern crate nix;
#[cfg(windows)]
extern crate winapi;

use std::process::Command;
use std::fmt;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::path::{Path, PathBuf};

// Re-export for macros.
pub use failure::Error;

pub mod macros;
mod process;
mod signal;

use process::Process;
use signal::Signal;

lazy_static! {
    static ref PID_MAP: Arc<Mutex<HashMap<i32, Process>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub fn disable_cleanup_on_ctrlc() {
    signal::uninstall_handler();
}

pub fn cleanup_on_ctrlc() {
    signal::install_handler(move |sig: Signal| {
        match sig {
            // SIGCHLD is special, initiate reap()
            Signal::SIGCHLD => {
                for (_pid, process) in PID_MAP.lock().unwrap().iter() {
                    process.reap();
                }
            }
            Signal::SIGINT => {
                for (_pid, process) in PID_MAP.lock().unwrap().iter() {
                    process.signal(sig);
                }
                ::std::process::exit(130);
            }
            _ => {
                for (_pid, process) in PID_MAP.lock().unwrap().iter() {
                    process.signal(sig);
                }
            }
        }
    });
}

pub struct SpawnGuard(i32);

impl ::std::ops::Drop for SpawnGuard {
    fn drop(&mut self) {
        PID_MAP.lock().unwrap().remove(&self.0).map(|process| process.reap());
    }
}

//---------------

pub trait CommandSpecExt {
    fn execute(self) -> Result<(), CommandError>;

    fn scoped_spawn(self) -> Result<SpawnGuard, ::std::io::Error>;
}

#[derive(Debug, Fail)]
pub enum CommandError {
    #[fail(display = "Encountered an IO error: {:?}", _0)]
    Io(#[cause] ::std::io::Error),

    #[fail(display = "Command was interrupted.")]
    Interrupt,

    #[fail(display = "Command failed with error code {}.", _0)]
    Code(i32),
}

impl CommandError {
    /// Returns the error code this command failed with. Can panic if not a `Code`.
    pub fn error_code(&self) -> i32 {
        if let CommandError::Code(value) = *self {
            value
        } else {
            panic!("Called error_code on a value that was not a CommandError::Code")
        }
    }
}

impl CommandSpecExt for Command {
    // Executes the command, and returns a versatile error struct
    fn execute(mut self) -> Result<(), CommandError> {
        match self.spawn() {
            Ok(mut child) => {
                match child.wait() {
                    Ok(status) => {
                        let ret = if status.success() {
                            Ok(())
                        } else if let Some(code) = status.code() {
                            Err(CommandError::Code(code))
                        } else {
                            Err(CommandError::Interrupt)
                        };

                        ret
                    }
                    Err(err) => {
                        Err(CommandError::Io(err))
                    }
                }
            },
            Err(err) => Err(CommandError::Io(err)),
        }
    }

    fn scoped_spawn(self) -> Result<SpawnGuard, ::std::io::Error> {
        let process = Process::new(self)?;
        let id = process.id();
        PID_MAP.lock().unwrap().insert(id, process);
        Ok(SpawnGuard(id))
    }
}

//---------------

pub enum CommandArg {
    Empty,
    Literal(String),
    List(Vec<String>),
}

fn shell_quote(value: &str) -> String {
    shlex::quote(&format!("{}", value)).to_string()
}

impl fmt::Display for CommandArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CommandArg::*;
        match *self {
            Empty => write!(f, ""),
            Literal(ref value) => {
                write!(f, "{}", shell_quote(&format!("{}", value)))
            },
            List(ref list) => {
                write!(f, "{}", list
                    .iter()
                    .map(|x| shell_quote(&format!("{}", x)).to_string())
                    .collect::<Vec<_>>()
                    .join(" "))
            }
        }
    }
}

impl<'a, 'b> From<&'a &'b str> for CommandArg {
    fn from(value: &&str) -> Self {
        CommandArg::Literal(value.to_string())
    }
}

impl From<String> for CommandArg {
    fn from(value: String) -> Self {
        CommandArg::Literal(value)
    }
}

impl<'a> From<&'a String> for CommandArg {
    fn from(value: &String) -> Self {
        CommandArg::Literal(value.to_string())
    }
}


impl<'a> From<&'a str> for CommandArg {
    fn from(value: &str) -> Self {
        CommandArg::Literal(value.to_string())
    }
}

impl<'a> From<&'a u64> for CommandArg {
    fn from(value: &u64) -> Self {
        CommandArg::Literal(value.to_string())
    }
}

impl<'a> From<&'a f64> for CommandArg {
    fn from(value: &f64) -> Self {
        CommandArg::Literal(value.to_string())
    }
}

impl<'a> From<&'a i32> for CommandArg {
    fn from(value: &i32) -> Self {
        CommandArg::Literal(value.to_string())
    }
}

impl<'a> From<&'a i64> for CommandArg {
    fn from(value: &i64) -> Self {
        CommandArg::Literal(value.to_string())
    }
}

impl<'a, T> From<&'a [T]> for CommandArg
    where T: fmt::Display {
    fn from(list: &[T]) -> Self {
        CommandArg::List(
            list
                .iter()
                .map(|x| format!("{}", x))
                .collect()
        )
    }
}

impl<'a, T> From<&'a Vec<T>> for CommandArg
    where T: fmt::Display {
    fn from(list: &Vec<T>) -> Self {
        CommandArg::from(list.as_slice())
    }
}

impl<'a, T> From<&'a Option<T>> for CommandArg
    where T: fmt::Display {
    fn from(opt: &Option<T>) -> Self {
        if let Some(ref value) = *opt {
            CommandArg::Literal(format!("{}", value))
        } else {
            CommandArg::Empty
        }
    }
}

pub fn command_arg<'a, T>(value: &'a T) -> CommandArg
    where CommandArg: std::convert::From<&'a T> {
    CommandArg::from(value)
}

//---------------

/// Represents the invocation specification used to generate a Command.
#[derive(Debug)]
struct CommandSpec {
    binary: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    cd: Option<String>,
}

impl CommandSpec {
    fn to_command(&self) -> Command {
        let cd = if let Some(ref cd) = self.cd {
            canonicalize_path(Path::new(cd)).unwrap()
        } else {
            ::std::env::current_dir().unwrap()
        };
        let mut binary = Path::new(&self.binary).to_owned();

        // On Windows, current_dir takes place after binary name resolution.
        // If current_dir is specified and the binary is referenced by a relative path,
        // add the dir change to its relative path.
        // https://github.com/rust-lang/rust/issues/37868
        if cfg!(windows) && binary.is_relative() && binary.components().count() != 1 {
            binary = cd.join(&binary);
        }

        // On windows, we run in cmd.exe by default. (This code is a naive way
        // of accomplishing this and may contain errors.)
        if cfg!(windows) {
            let mut cmd = Command::new("cmd");
            cmd.current_dir(cd);
            let invoke_string = format!("{} {}", binary.as_path().to_string_lossy(), self.args.join(" "));
            cmd.args(&["/C", &invoke_string]);
            for (key, value) in &self.env {
                cmd.env(key, value);
            }
            return cmd;
        }

        let mut cmd = Command::new(binary);
        cmd.current_dir(cd);
        cmd.args(&self.args);
        for (key, value) in &self.env {
            cmd.env(key, value);
        }
        cmd
    }
}

// Strips UNC from canonicalized paths.
// See https://github.com/rust-lang/rust/issues/42869 for why this is needed.
#[cfg(windows)]
fn canonicalize_path<'p, P>(path: P) -> Result<PathBuf, Error>
where P: Into<&'p Path> {
    use std::ffi::OsString;
    use std::os::windows::prelude::*;

    let canonical = path.into().canonicalize()?;
    let vec_chars = canonical.as_os_str().encode_wide().collect::<Vec<u16>>();
    if vec_chars[0..4] == [92, 92, 63, 92] {
        return Ok(Path::new(&OsString::from_wide(&vec_chars[4..])).to_owned());
    }

    Ok(canonical)
}

#[cfg(not(windows))]
fn canonicalize_path<'p, P>(path: P) -> Result<PathBuf, Error>
where P: Into<&'p Path> {
    Ok(path.into().canonicalize()?)
}

//---------------

pub fn commandify(value: String) -> Result<Command, Error> {
    let lines = value.trim().split("\n").map(String::from).collect::<Vec<_>>();

    #[derive(Debug, PartialEq)]
    enum SpecState {
        Cd,
        Env,
        Cmd,
    }

    let mut env = HashMap::<String, String>::new();
    let mut cd = None;

    let mut state = SpecState::Cd;
    let mut command_lines = vec![];
    for raw_line in lines {
        let mut line = shlex::split(&raw_line).unwrap_or(vec![]);
        if state == SpecState::Cmd {
            command_lines.push(raw_line);
        } else {
            if raw_line.trim().is_empty() {
                continue;
            }

            match line.get(0).map(|x| x.as_ref()) {
                Some("cd") => {
                    if state != SpecState::Cd {
                        bail!("cd should be the first line in your command! macro.");
                    }
                    ensure!(line.len() == 2, "Too many arguments in cd; expected 1, found {}", line.len() - 1);
                    cd = Some(line.remove(1));
                    state = SpecState::Env;
                }
                Some("export") => {
                    if state != SpecState::Cd && state != SpecState::Env {
                        bail!("exports should follow cd but precede your command in the command! macro.");
                    }
                    ensure!(line.len() >= 2, "Not enough arguments in export; expected at least 1, found {}", line.len() - 1);
                    for item in &line[1..] {
                        let mut items = item.splitn(2, "=").collect::<Vec<_>>();
                        ensure!(items.len() > 0, "Expected export of the format NAME=VALUE");
                        env.insert(items[0].to_string(), items[1].to_string());
                    }
                    state = SpecState::Env;
                }
                None | Some(_) => {
                    command_lines.push(raw_line);
                    state = SpecState::Cmd;
                }
            }
        }
    }
    if state != SpecState::Cmd || command_lines.is_empty() {
        bail!("Didn't find a command in your command! macro.");
    }

    // Join the command string and split out binary / args.
    let command_string = command_lines.join("\n").replace("\\\n", "\n");
    let mut command = shlex::split(&command_string).expect("Command string couldn't be parsed by shlex");
    let binary = command.remove(0); 
    let args = command;

    // Generate the CommandSpec struct.
    let spec = CommandSpec {
        binary,
        args,
        env,
        cd,
    };

    // DEBUG
    // eprintln!("COMMAND: {:?}", spec);

    Ok(spec.to_command())
}
