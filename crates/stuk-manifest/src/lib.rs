#[cfg(test)]
mod tests;
mod validation;

use std::{collections::BTreeMap, path::Path};

use serde::Deserialize;
use stuk_settings::SettingsSchema;
use thiserror::Error;

pub use validation::{validate, validate_with_base_dir};

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub app: AppSection,
    #[serde(default)]
    pub window: BTreeMap<String, WindowSection>,
    #[serde(default)]
    pub platform: PlatformSection,
    #[serde(default)]
    pub permissions: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub actions: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub settings: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppSection {
    pub id: String,
    pub name: String,
    pub version: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WindowSection {
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub min_width: Option<u32>,
    pub min_height: Option<u32>,
    pub material: Option<String>,
    pub chrome: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PlatformSection {
    #[serde(default)]
    pub staccato: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub path: String,
    pub message: String,
    pub fix_hint: Option<String>,
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("failed to read manifest: {0}")]
    Read(#[from] std::io::Error),
    #[error("failed to parse manifest: {0}")]
    Parse(#[from] toml::de::Error),
}

pub fn parse(source: &str) -> Result<Manifest, ManifestError> {
    Ok(toml::from_str(source)?)
}

pub fn parse_file(path: impl AsRef<Path>) -> Result<Manifest, ManifestError> {
    parse(&std::fs::read_to_string(path)?)
}

impl Manifest {
    pub fn settings_schema(&self) -> Result<SettingsSchema, Vec<Diagnostic>> {
        SettingsSchema::from_toml(&self.settings).map_err(validation::settings_diagnostics)
    }
}
