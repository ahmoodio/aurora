use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageSource {
    Repo,
    Aur,
    Flatpak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSummary {
    pub name: String,
    pub summary: String,
    pub version: String,
    pub source: PackageSource,
    pub installed: bool,
    pub origin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDetails {
    pub name: String,
    pub summary: String,
    pub description: String,
    pub version: String,
    pub source: PackageSource,
    pub installed: bool,
    pub size: Option<String>,
    pub home: Option<String>,
    pub screenshots: Vec<String>,
    pub icon_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    Install,
    Remove,
    Upgrade,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAction {
    pub name: String,
    pub source: PackageSource,
    pub kind: ActionKind,
    pub origin: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TransactionQueue {
    pub actions: Vec<TransactionAction>,
}

impl TransactionQueue {
    pub fn push(&mut self, action: TransactionAction) {
        self.actions.push(action);
    }

    pub fn clear(&mut self) {
        self.actions.clear();
    }

    pub fn len(&self) -> usize {
        self.actions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AurHelperKind {
    Yay,
    Paru,
}

impl AurHelperKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            AurHelperKind::Yay => "yay",
            AurHelperKind::Paru => "paru",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub aur_helper: AurHelperKind,
    pub allow_noconfirm: bool,
    pub theme: ThemeMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            aur_helper: AurHelperKind::Yay,
            allow_noconfirm: false,
            theme: ThemeMode::System,
        }
    }
}
