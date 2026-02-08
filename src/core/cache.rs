use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use directories::ProjectDirs;

use crate::core::models::Settings;

pub fn project_dirs() -> ProjectDirs {
    ProjectDirs::from("io", "github.ahmoodio", "Aurora").expect("Project dirs")
}

pub fn cache_dir() -> PathBuf {
    project_dirs().cache_dir().to_path_buf()
}

pub fn screenshots_dir() -> PathBuf {
    cache_dir().join("screenshots")
}

pub fn config_dir() -> PathBuf {
    project_dirs().config_dir().to_path_buf()
}

pub fn load_settings() -> Settings {
    let path = config_dir().join("settings.json");
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(settings) = serde_json::from_str(&data) {
            return settings;
        }
    }
    Settings::default()
}

pub fn save_settings(settings: &Settings) -> Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join("settings.json");
    let data = serde_json::to_string_pretty(settings)?;
    fs::write(path, data)?;
    Ok(())
}

pub fn ensure_cache_dirs() -> Result<()> {
    fs::create_dir_all(screenshots_dir())?;
    Ok(())
}

pub fn clear_screenshots_cache() -> Result<()> {
    let dir = screenshots_dir();
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(&dir)?;
    Ok(())
}

pub fn find_logo_path() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("assets/logo.svg"),
        PathBuf::from("assets/logo.png"),
        PathBuf::from("/usr/share/aurora/assets/logo.svg"),
        PathBuf::from("/usr/share/aurora/assets/logo.png"),
        PathBuf::from("/usr/share/icons/hicolor/scalable/apps/io.github.ahmoodio.aurora.svg"),
        PathBuf::from("/usr/share/icons/hicolor/scalable/apps/io.github.ahmoodio.aurora.png"),
        PathBuf::from("/usr/share/icons/hicolor/256x256/apps/io.github.ahmoodio.aurora.png"),
    ];

    for path in candidates {
        if path.exists() {
            return Some(path);
        }
    }
    None
}
