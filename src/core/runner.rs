use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use std::sync::mpsc::Sender;

use crate::core::models::TerminalEmulator;

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

    pub fn display_line(&self) -> String {
        let mut parts = Vec::new();
        for (k, v) in &self.env {
            parts.push(format!("{k}={}", shell_quote(v)));
        }
        parts.push(shell_quote(&self.program));
        for arg in &self.args {
            parts.push(shell_quote(arg));
        }
        parts.join(" ")
    }

    fn shell_command(&self) -> String {
        let mut parts = vec!["env".to_string()];
        for (k, v) in &self.env {
            parts.push(format!("{k}={}", shell_quote(v)));
        }
        parts.push(shell_quote(&self.program));
        for arg in &self.args {
            parts.push(shell_quote(arg));
        }
        parts.join(" ")
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

    pub fn run_external_terminal(
        &self,
        spec: CommandSpec,
        preferred_terminal: TerminalEmulator,
        sender: Sender<LogEvent>,
    ) -> Result<()> {
        let Some(terminal) = resolve_terminal(preferred_terminal) else {
            return Err(anyhow!(
                "No supported terminal found. Install kitty, konsole, or alacritty."
            ));
        };

        thread::spawn(move || {
            let display_line = spec.display_line();
            let shell_command = spec.shell_command();
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let exit_file = std::env::temp_dir().join(format!("aurora-exit-{stamp}.code"));
            let exit_file_text = exit_file.to_string_lossy().to_string();

            let script = format!(
                "printf '%s\\n' {trace}\n{command}\ncode=$?\nprintf '%s\\n' \"$code\" > {exit_file}\nexit \"$code\"\n",
                trace = shell_quote(&format!("[aurora] Running: {display_line}")),
                command = shell_command,
                exit_file = shell_quote(&exit_file_text),
            );

            let mut cmd = Command::new(terminal.binary());
            cmd.args(terminal.launch_args(&script));
            let status = match cmd.status() {
                Ok(status) => status.code().unwrap_or(1),
                Err(err) => {
                    let _ = sender.send(LogEvent::Line(format!(
                        "Failed to launch terminal {}: {err}",
                        terminal.label()
                    )));
                    let _ = sender.send(LogEvent::Finished(1));
                    return;
                }
            };

            let final_code = std::fs::read_to_string(&exit_file)
                .ok()
                .and_then(|value| value.trim().parse::<i32>().ok())
                .unwrap_or(status);
            let _ = std::fs::remove_file(&exit_file);
            let _ = sender.send(LogEvent::Line(format!(
                "External terminal finished with exit code {final_code}"
            )));
            let _ = sender.send(LogEvent::Finished(final_code));
        });

        Ok(())
    }
}

fn shell_quote(input: &str) -> String {
    if input.is_empty() {
        return "''".to_string();
    }
    format!("'{}'", input.replace('\'', "'\"'\"'"))
}

fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .arg("-lc")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn resolve_terminal(preferred: TerminalEmulator) -> Option<TerminalEmulator> {
    match preferred {
        TerminalEmulator::Auto => [TerminalEmulator::Kitty, TerminalEmulator::Konsole, TerminalEmulator::Alacritty]
            .into_iter()
            .find(|terminal| command_exists(terminal.binary())),
        _ => {
            if command_exists(preferred.binary()) {
                Some(preferred)
            } else {
                None
            }
        }
    }
}

impl TerminalEmulator {
    fn binary(self) -> &'static str {
        match self {
            TerminalEmulator::Auto => "",
            TerminalEmulator::Kitty => "kitty",
            TerminalEmulator::Konsole => "konsole",
            TerminalEmulator::Alacritty => "alacritty",
        }
    }

    fn launch_args(self, script: &str) -> Vec<String> {
        match self {
            TerminalEmulator::Kitty => vec![
                "sh".to_string(),
                "-lc".to_string(),
                script.to_string(),
            ],
            TerminalEmulator::Konsole => vec![
                "-e".to_string(),
                "sh".to_string(),
                "-lc".to_string(),
                script.to_string(),
            ],
            TerminalEmulator::Alacritty => vec![
                "-e".to_string(),
                "sh".to_string(),
                "-lc".to_string(),
                script.to_string(),
            ],
            TerminalEmulator::Auto => vec![
                "sh".to_string(),
                "-lc".to_string(),
                script.to_string(),
            ],
        }
    }
}
