use std::ffi::OsStr;
use std::process::Command;

use anyhow::{anyhow, Result};

use crate::core::models::{PackageDetails, PackageSource, PackageSummary};
use crate::core::providers::PacmanProvider;

#[derive(Debug, Default)]
pub struct Pacman;

impl Pacman {
    fn run_capture<I, S>(args: I) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = Command::new("pacman")
            .args(args)
            .env("LC_ALL", "C")
            .output()?;
        if !output.status.success() {
            return Err(anyhow!(
                "pacman failed with status {}",
                output.status
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn parse_search_output(output: &str) -> Vec<PackageSummary> {
        let mut results = Vec::new();
        let mut lines = output.lines();
        while let Some(line) = lines.next() {
            if line.trim().is_empty() {
                continue;
            }
            let header = line.trim();
            let summary = lines.next().unwrap_or("").trim().to_string();
            let mut parts = header.split_whitespace();
            let repo_pkg = parts.next().unwrap_or("");
            let version = parts.next().unwrap_or("").to_string();
            let name = repo_pkg.split('/').nth(1).unwrap_or(repo_pkg).to_string();
            results.push(PackageSummary {
                name,
                summary,
                version,
                source: PackageSource::Repo,
                installed: false,
                origin: None,
            });
        }
        results
    }

    fn parse_info(output: &str, source: PackageSource) -> PackageDetails {
        let mut name = String::new();
        let mut version = String::new();
        let mut desc = String::new();
        let mut summary = String::new();
        let mut size = None;
        let mut home = None;

        for line in output.lines() {
            if let Some((k, v)) = line.split_once(':') {
                let key = k.trim();
                let value = v.trim();
                match key {
                    "Name" => name = value.to_string(),
                    "Version" => version = value.to_string(),
                    "Description" => {
                        desc = value.to_string();
                        summary = value.to_string();
                    }
                    "Installed Size" | "Download Size" => size = Some(value.to_string()),
                    "URL" => home = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        PackageDetails {
            name,
            summary,
            description: desc,
            version,
            source,
            installed: false,
            size,
            home,
            screenshots: Vec::new(),
            icon_name: None,
        }
    }
}

impl PacmanProvider for Pacman {
    fn search(&self, query: &str) -> Result<Vec<PackageSummary>> {
        let mut args = vec!["-Ss".to_string()];
        let terms: Vec<String> = query
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        if terms.is_empty() {
            return Ok(Vec::new());
        }
        args.extend(terms);
        let output = Self::run_capture(args)?;
        Ok(Self::parse_search_output(&output))
    }

    fn info_repo(&self, name: &str) -> Result<PackageDetails> {
        let output = Self::run_capture(["-Si", name])?;
        Ok(Self::parse_info(&output, PackageSource::Repo))
    }

    fn info_installed(&self, name: &str) -> Result<PackageDetails> {
        let output = Self::run_capture(["-Qi", name])?;
        let mut details = Self::parse_info(&output, PackageSource::Repo);
        details.installed = true;
        Ok(details)
    }

    fn list_installed(&self) -> Result<Vec<PackageSummary>> {
        let output = Self::run_capture(["-Q"])?;
        let mut results = Vec::new();
        for line in output.lines() {
            let mut parts = line.split_whitespace();
            let name = parts.next().unwrap_or("").to_string();
            let version = parts.next().unwrap_or("").to_string();
            if name.is_empty() {
                continue;
            }
            results.push(PackageSummary {
                name,
                summary: String::from(""),
                version,
                source: PackageSource::Repo,
                installed: true,
                origin: None,
            });
        }
        Ok(results)
    }
}
