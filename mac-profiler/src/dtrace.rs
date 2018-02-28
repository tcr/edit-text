use failure::Error;
use nix::sys::signal::Signal;
use std::io::BufReader;
use std::io::prelude::*;
use std::process::{Child, ChildStdout, Command, Stdio};
use toml::Value;
use nix;
use shlex;

pub struct DtraceProbe {
    command: Child,
    buffer: Vec<String>,
    reader: ::std::io::Lines<BufReader<ChildStdout>>,
}

impl DtraceProbe {
    pub fn stop(&self) -> Result<(), Error> {
        let kill_id = self.command.id();
        nix::sys::signal::kill(nix::unistd::Pid::from_raw(kill_id as i32), Signal::SIGINT)?;
        Ok(())
    }
}

impl Iterator for DtraceProbe {
    type Item = Value;

    fn next(&mut self) -> Option<Value> {
        while let Some(line) = self.reader.next() {
            if let Ok(line) = line {
                if line == "[[entry]]" {
                    if !self.buffer.is_empty() {
                        if let Ok(toml) = self.buffer.join("\n").parse::<Value>() {
                            self.buffer.split_off(0);
                            return Some(toml);
                        }
                    }

                    // Move forward
                    self.buffer.split_off(0);
                } else {
                    if line.len() > 0 {
                        self.buffer.push(line);
                    }
                }
            }
        }

        if !self.buffer.is_empty() {
            if let Ok(toml) = self.buffer.join("\n").parse::<Value>() {
                self.buffer.split_off(0);
                return Some(toml);
            }
        }

        None
    }
}

pub fn dtrace_probe(target: &str, script: &str) -> Result<DtraceProbe, Error> {
    let dtrace_script = shlex::quote(script);
    let len = dtrace_script.chars().count();
    let dtrace_arg = dtrace_script
        .chars()
        .skip(1)
        .take(len - 2)
        .collect::<String>();

    // Using `sh` is an inelegant hack to do child process file descriptor redirection
    // Is there a way to read a child's file descriptor in Rust? Other than 0, 1, and 2
    let mut child = Command::new("sh")
        .args(
            vec![
                "-c".to_string(),
                format!(
                    r#"/usr/sbin/dtrace -n "{}" '-c {}' -o '/dev/fd/3' 3>&1 1>/dev/null 2>/dev/null"#,
                    dtrace_arg, target
                ),
            ].into_iter(),
        )
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();

    Ok(DtraceProbe {
        command: child,
        buffer: vec![],
        reader: BufReader::new(stdout).lines(),
    })
}
