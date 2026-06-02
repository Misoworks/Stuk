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
    pub targets: BTreeMap<String, bool>,
    #[serde(default)]
    pub actions: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub settings: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub webview: WebViewSection,
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
    #[serde(default)]
    pub transparent: Option<bool>,
    #[serde(default)]
    pub background_effect: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PlatformSection {
    #[serde(default)]
    pub staccato: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct WebViewSection {
    #[serde(default)]
    pub engine: Option<String>,
    #[serde(default)]
    pub runtime: Option<String>,
    #[serde(default)]
    pub entry: Option<String>,
    #[serde(default)]
    pub min_version: Option<String>,
    #[serde(default)]
    pub allow_user_install: Option<bool>,
    #[serde(default)]
    pub allow_bundled: Option<bool>,
    #[serde(default)]
    pub dev: WebViewDevSection,
    #[serde(default)]
    pub security: WebViewSecuritySection,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct WebViewDevSection {
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct WebViewSecuritySection {
    #[serde(default)]
    pub remote_content: Option<bool>,
    #[serde(default)]
    pub allowed_origins: Option<Vec<String>>,
    #[serde(default)]
    pub allowed_bridge_permissions: Option<Vec<String>>,
    #[serde(default)]
    pub devtools: Option<String>,
    #[serde(default)]
    pub allow_eval: Option<bool>,
    #[serde(default)]
    pub allow_node: Option<bool>,
    #[serde(default)]
    pub csp: Option<String>,
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
