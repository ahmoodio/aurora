use std::env;
use std::path::Path;
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
    let allowed_ops = ["-S", "-Syu", "-Rns", "-U"];
    if !allowed_ops.contains(&op.as_str()) {
        return Err(anyhow!("operation not allowed: {op}"));
    }

    let mut allow_flags = true;
    let mut pkgs = Vec::new();
    let mut pkg_files = Vec::new();

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
            if op == "-U" {
                if !is_safe_pkgfile(arg) {
                    return Err(anyhow!("invalid package file: {arg}"));
                }
                pkg_files.push(arg);
            } else {
                if !is_safe_pkg(arg) {
                    return Err(anyhow!("invalid package name: {arg}"));
                }
                pkgs.push(arg);
            }
        }
    }

    if op == "-S" || op == "-Rns" {
        if pkgs.is_empty() {
            return Err(anyhow!("no packages supplied"));
        }
    }

    if op == "-U" && pkg_files.is_empty() {
        return Err(anyhow!("no package files supplied"));
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

fn is_safe_pkgfile(path: &str) -> bool {
    if path.is_empty() || path.len() > 512 {
        return false;
    }

    let mut candidate = Path::new(path).to_path_buf();
    if !candidate.is_absolute() {
        if let Ok(cwd) = env::current_dir() {
            candidate = cwd.join(candidate);
        }
    }

    let canon = match candidate.canonicalize() {
        Ok(p) => p,
        Err(_) => return false,
    };

    if !canon.is_file() {
        return false;
    }

    let allowed_prefixes = ["/home/", "/tmp/", "/var/cache/"];
    let canon_str = canon.to_string_lossy();
    if !allowed_prefixes.iter().any(|p| canon_str.starts_with(p)) {
        return false;
    }

    let name = canon.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let allowed_exts = [
        ".pkg.tar.zst",
        ".pkg.tar.xz",
        ".pkg.tar.gz",
        ".pkg.tar",
    ];
    allowed_exts.iter().any(|ext| name.ends_with(ext))
}
