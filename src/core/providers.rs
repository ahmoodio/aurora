use anyhow::Result;

use crate::core::models::{PackageDetails, PackageSummary};

pub trait PacmanProvider: Send + Sync {
    fn search(&self, query: &str) -> Result<Vec<PackageSummary>>;
    fn info_repo(&self, name: &str) -> Result<PackageDetails>;
    fn info_installed(&self, name: &str) -> Result<PackageDetails>;
    fn list_installed(&self) -> Result<Vec<PackageSummary>>;
}

pub trait AurProvider: Send + Sync {
    fn search(&self, query: &str) -> Result<Vec<PackageSummary>>;
    fn info(&self, name: &str) -> Result<PackageDetails>;
}

pub trait FlatpakProvider: Send + Sync {
    fn search(&self, query: &str) -> Result<Vec<PackageSummary>>;
    fn info(&self, name: &str) -> Result<PackageDetails>;
    fn list_installed(&self) -> Result<Vec<PackageSummary>>;
}

pub mod pacman;
pub mod aur;
pub mod flatpak;
