use std::ffi::OsStr;
use std::process::Command;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};

use crate::core::models::{PackageDetails, PackageSource, PackageSummary, Settings};
use crate::core::providers::AurProvider;

#[derive(Debug, Clone)]
pub struct Aur {
    settings: Arc<Mutex<Settings>>,
}

impl Aur {
    pub fn new(settings: Arc<Mutex<Settings>>) -> Self {
        Self { settings }
    }

    fn helper_bin(&self) -> String {
        self.settings.lock().unwrap().aur_helper.as_str().to_string()
    }

    fn run_capture<I, S>(&self, args: I) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let helper = self.helper_bin();
        let output = Command::new(&helper)
            .args(args)
            .env("LC_ALL", "C")
            .output()?;
        if !output.status.success() {
            return Err(anyhow!("{} failed with status {}", helper, output.status));
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
                source: PackageSource::Aur,
                installed: false,
                origin: None,
            });
        }
        results
    }

    fn parse_info(output: &str) -> PackageDetails {
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
            source: PackageSource::Aur,
            installed: false,
            size,
            home,
            screenshots: Vec::new(),
            icon_name: None,
        }
    }
}

impl AurProvider for Aur {
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
        let output = self.run_capture(args)?;
        Ok(Self::parse_search_output(&output))
    }

    fn info(&self, name: &str) -> Result<PackageDetails> {
        let output = self.run_capture(["-Si", name])?;
        Ok(Self::parse_info(&output))
    }
}
