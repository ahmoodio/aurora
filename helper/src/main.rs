use std::env;
use std::process::Command;

use anyhow::{anyhow, Result};

fn main() {
    if let Err(err) = run() {
        eprintln!("aurora-helper error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    ensure_root()?;

    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        return Err(anyhow!("no command supplied"));
    }

    let target = args.remove(0);
    if target != "pacman" {
        return Err(anyhow!("unsupported target: {target}"));
    }

    validate_pacman(&args)?;

    let status = Command::new("pacman")
        .args(&args)
        .env("LC_ALL", "C")
        .status()?;

    std::process::exit(status.code().unwrap_or(1));
}

fn ensure_root() -> Result<()> {
    let uid = unsafe { libc::geteuid() };
    if uid != 0 {
        return Err(anyhow!("must be run as root via pkexec"));
    }
    Ok(())
}

fn validate_pacman(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("missing pacman args"));
    }

    let op = &args[0];
    let allowed_ops = ["-S", "-Syu", "-Rns"]; 
    if !allowed_ops.contains(&op.as_str()) {
        return Err(anyhow!("operation not allowed: {op}"));
    }

    let mut allow_flags = true;
    let mut pkgs = Vec::new();

    for arg in args.iter().skip(1) {
        if arg.starts_with('-') {
            if !allow_flags {
                return Err(anyhow!("flags after packages are not allowed"));
            }
            if !is_allowed_flag(arg) {
                return Err(anyhow!("flag not allowed: {arg}"));
            }
        } else {
            allow_flags = false;
            if !is_safe_pkg(arg) {
                return Err(anyhow!("invalid package name: {arg}"));
            }
            pkgs.push(arg);
        }
    }

    if op == "-S" || op == "-Rns" {
        if pkgs.is_empty() {
            return Err(anyhow!("no packages supplied"));
        }
    }

    if op == "-Syu" && !pkgs.is_empty() {
        return Err(anyhow!("-Syu does not accept packages"));
    }

    if pkgs.len() > 200 {
        return Err(anyhow!("too many packages"));
    }

    Ok(())
}

fn is_allowed_flag(flag: &str) -> bool {
    matches!(flag, "--noconfirm" | "--needed" | "--noprogressbar")
}

fn is_safe_pkg(name: &str) -> bool {
    if name.is_empty() || name.len() > 128 {
        return false;
    }
    name.chars().all(|c| c.is_ascii_alphanumeric() || "+-._@".contains(c))
}
