use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::Receiver;
use std::thread;

use anyhow::{anyhow, Result};
use std::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

impl CommandSpec {
    pub fn new(program: &str, args: Vec<String>) -> Self {
        Self {
            program: program.to_string(),
            args,
            env: vec![(String::from("LC_ALL"), String::from("C"))],
        }
    }
}

#[derive(Debug, Clone)]
pub enum LogEvent {
    Line(String),
    Finished(i32),
}

#[derive(Debug, Clone)]
pub struct CommandRunner {
    pub log_limit: usize,
}

impl Default for CommandRunner {
    fn default() -> Self {
        Self { log_limit: 1000 }
    }
}

impl CommandRunner {
    pub fn run_capture(&self, spec: &CommandSpec) -> Result<String> {
        let mut cmd = Command::new(&spec.program);
        cmd.args(&spec.args);
        for (k, v) in &spec.env {
            cmd.env(k, v);
        }
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(anyhow!("command failed with status {}", output.status));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn run_streaming(
        &self,
        spec: CommandSpec,
        sender: Sender<LogEvent>,
        input_rx: Option<Receiver<String>>,
    ) -> Result<()> {
        thread::spawn(move || {
            let mut cmd = Command::new(&spec.program);
            cmd.args(&spec.args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            for (k, v) in &spec.env {
                cmd.env(k, v);
            }

            let mut child = match cmd.spawn() {
                Ok(child) => child,
                Err(err) => {
                    let _ = sender.send(LogEvent::Line(format!("Failed to spawn: {err}")));
                    let _ = sender.send(LogEvent::Finished(1));
                    return;
                }
            };

            if let Some(mut stdin) = child.stdin.take() {
                if let Some(rx) = input_rx {
                    thread::spawn(move || {
                        for line in rx {
                            let mut input = line;
                            if !input.ends_with('\n') {
                                input.push('\n');
                            }
                            let _ = stdin.write_all(input.as_bytes());
                            let _ = stdin.flush();
                        }
                    });
                }
            }

            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            if let Some(out) = stdout {
                let tx = sender.clone();
                thread::spawn(move || {
                    let reader = BufReader::new(out);
                    for line in reader.lines().flatten() {
                        let _ = tx.send(LogEvent::Line(line));
                    }
                });
            }

            if let Some(err) = stderr {
                let tx = sender.clone();
                thread::spawn(move || {
                    let reader = BufReader::new(err);
                    for line in reader.lines().flatten() {
                        let _ = tx.send(LogEvent::Line(line));
                    }
                });
            }

            let status = match child.wait() {
                Ok(status) => status.code().unwrap_or(1),
                Err(_) => 1,
            };
            let _ = sender.send(LogEvent::Finished(status));
        });

        Ok(())
    }
}
