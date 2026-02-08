use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;

use anyhow::{anyhow, Result};
use serde_json::Value;
use url::Url;

use crate::core::cache::{ensure_cache_dirs, screenshots_dir};

#[derive(Debug, Clone)]
pub struct AppStreamComponent {
    pub id: String,
    pub summary: Option<String>,
    pub icon_name: Option<String>,
    pub screenshots: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AppStreamClient;

impl AppStreamClient {
    pub fn search_component(&self, name: &str) -> Option<AppStreamComponent> {
        let output = Command::new("appstreamcli")
            .args(["search", name, "--format=json"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout).to_string();
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            return Self::component_from_json(&v);
        }
        Self::component_from_text(&text)
    }

    pub fn get_component(&self, id: &str) -> Option<AppStreamComponent> {
        let output = Command::new("appstreamcli")
            .args(["get", id, "--format=json"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout).to_string();
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            return Self::component_from_json(&v);
        }
        None
    }

    fn component_from_json(v: &Value) -> Option<AppStreamComponent> {
        let comps = v.get("components").and_then(|c| c.as_array())?;
        let comp = comps.first()?;
        let id = comp.get("id").and_then(|v| v.as_str())?.to_string();
        let summary = comp.get("summary").and_then(|v| v.as_str()).map(|s| s.to_string());

        let mut icon_name = None;
        if let Some(icons) = comp.get("icons").and_then(|v| v.as_array()) {
            for icon in icons {
                if let Some(name) = icon.get("name").and_then(|v| v.as_str()) {
                    icon_name = Some(name.to_string());
                    break;
                }
            }
        }

        let mut screenshots = Vec::new();
        if let Some(shots) = comp.get("screenshots").and_then(|v| v.as_array()) {
            for shot in shots {
                if let Some(images) = shot.get("images").and_then(|v| v.as_array()) {
                    if let Some(url) = images
                        .iter()
                        .filter_map(|img| img.get("url").and_then(|v| v.as_str()))
                        .next()
                    {
                        screenshots.push(url.to_string());
                    }
                }
            }
        }

        Some(AppStreamComponent {
            id,
            summary,
            icon_name,
            screenshots,
        })
    }

    fn component_from_text(text: &str) -> Option<AppStreamComponent> {
        for line in text.lines() {
            let line = line.trim();
            if line.contains('.') && (line.contains(".desktop") || line.contains(".metainfo")) {
                return Some(AppStreamComponent {
                    id: line.to_string(),
                    summary: None,
                    icon_name: None,
                    screenshots: Vec::new(),
                });
            }
        }
        None
    }

    pub fn download_screenshots_async(&self, urls: Vec<String>) {
        let _ = ensure_cache_dirs();
        thread::spawn(move || {
            for url in urls {
                let _ = Self::download_one(&url);
            }
        });
    }

    pub fn cached_path_for_url(url: &str) -> Option<PathBuf> {
        let parsed = Url::parse(url).ok()?;
        let filename = parsed.path_segments()?.last()?.to_string();
        Some(screenshots_dir().join(filename))
    }

    pub fn ensure_cached(url: &str) -> Option<PathBuf> {
        let _ = ensure_cache_dirs();
        let path = Self::cached_path_for_url(url)?;
        if path.exists() {
            return Some(path);
        }
        if Self::download_one(url).is_ok() && path.exists() {
            return Some(path);
        }
        None
    }

    fn download_one(url: &str) -> Result<()> {
        let path = Self::cached_path_for_url(url).ok_or_else(|| anyhow!("invalid url"))?;
        if path.exists() {
            return Ok(());
        }
        let response = ureq::get(url).call()?;
        let mut reader = response.into_reader();
        let mut file = fs::File::create(path)?;
        let _ = std::io::copy(&mut reader, &mut file)?;
        Ok(())
    }
}
