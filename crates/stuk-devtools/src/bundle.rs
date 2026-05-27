use std::path::Path;

use stuk_manifest::{Diagnostic, Manifest};

use crate::{AppInspection, ManifestInspection};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BundleTarget {
    Staccato,
    Flatpak,
    AppImage,
    Windows,
    Macos,
}

impl BundleTarget {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "staccato" => Some(Self::Staccato),
            "flatpak" => Some(Self::Flatpak),
            "appimage" => Some(Self::AppImage),
            "windows" => Some(Self::Windows),
            "macos" => Some(Self::Macos),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Staccato => "staccato",
            Self::Flatpak => "flatpak",
            Self::AppImage => "appimage",
            Self::Windows => "windows",
            Self::Macos => "macos",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BundlePlan {
    pub ok: bool,
    pub target: BundleTarget,
    pub app: AppInspection,
    pub binary_name: String,
    pub manifest_path: String,
    pub icon: Option<String>,
    pub includes: Vec<String>,
    pub actions_count: usize,
    pub settings_count: usize,
    pub permissions_count: usize,
    pub permissions: Vec<crate::PermissionInspection>,
    pub staccato: StaccatoBundleMetadata,
    pub diagnostics: Vec<crate::DiagnosticInspection>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StaccatoBundleMetadata {
    pub command_palette: bool,
    pub workspace_sessions: bool,
    pub shell_tabs: bool,
    pub preferred_mode: Option<String>,
    pub preferred_material: Option<String>,
    pub preferred_chrome: Option<String>,
}

impl BundlePlan {
    pub fn from_manifest(
        manifest: &Manifest,
        diagnostics: &[Diagnostic],
        target: BundleTarget,
        manifest_path: &Path,
    ) -> Self {
        let inspection = ManifestInspection::from_manifest(manifest, diagnostics);
        Self {
            ok: inspection.ok,
            target,
            app: inspection.app.clone(),
            binary_name: binary_name_from_app_id(&inspection.app.id),
            manifest_path: manifest_path.display().to_string(),
            icon: inspection.app.icon.clone(),
            includes: bundle_includes(target, &inspection),
            actions_count: inspection.actions.len(),
            settings_count: inspection.settings_count,
            permissions_count: inspection.permissions_count,
            permissions: inspection.permissions.clone(),
            staccato: staccato_metadata(manifest),
            diagnostics: inspection.diagnostics,
        }
    }

    pub fn to_text(&self) -> String {
        let mut output = String::new();
        output.push_str("Bundle\n");
        output.push_str(&format!("  target: {}\n", self.target.as_str()));
        output.push_str(&format!("  app: {} ({})\n", self.app.name, self.app.id));
        output.push_str(&format!("  binary: {}\n", self.binary_name));
        output.push_str(&format!("  manifest: {}\n", self.manifest_path));
        if let Some(icon) = &self.icon {
            output.push_str(&format!("  icon: {icon}\n"));
        }
        output.push_str("Includes\n");
        for item in &self.includes {
            output.push_str(&format!("  {item}\n"));
        }
        output.push_str("Metadata\n");
        output.push_str(&format!("  actions: {}\n", self.actions_count));
        output.push_str(&format!("  settings: {}\n", self.settings_count));
        output.push_str(&format!("  permissions: {}\n", self.permissions_count));
        for permission in &self.permissions {
            output.push_str(&format!(
                "    {}: {} ({})\n",
                permission.name, permission.value, permission.value_kind
            ));
        }
        output.push_str(&format!(
            "  staccato.command_palette: {}\n",
            self.staccato.command_palette
        ));
        output.push_str(&format!(
            "  staccato.workspace_sessions: {}\n",
            self.staccato.workspace_sessions
        ));
        output.push_str(&format!(
            "  staccato.shell_tabs: {}\n",
            self.staccato.shell_tabs
        ));
        output.push_str(&format!("Diagnostics: {}\n", self.diagnostics.len()));
        for diagnostic in &self.diagnostics {
            output.push_str(&format!(
                "  {}: {}: {}\n",
                diagnostic.level, diagnostic.path, diagnostic.message
            ));
        }
        output
    }

    pub fn to_json(&self) -> String {
        let includes = self
            .includes
            .iter()
            .map(|item| format!("\"{}\"", escape_json(item)))
            .collect::<Vec<_>>()
            .join(",");
        let diagnostics = self
            .diagnostics
            .iter()
            .map(|diagnostic| {
                format!(
                    "{{\"level\":\"{}\",\"path\":\"{}\",\"message\":\"{}\"{}}}",
                    escape_json(&diagnostic.level),
                    escape_json(&diagnostic.path),
                    escape_json(&diagnostic.message),
                    diagnostic
                        .fix_hint
                        .as_deref()
                        .map(|hint| format!(",\"fix_hint\":\"{}\"", escape_json(hint)))
                        .unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        let permissions = self
            .permissions
            .iter()
            .map(crate::PermissionInspection::to_json)
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "{{\"ok\":{},\"target\":\"{}\",\"app\":{{\"id\":\"{}\",\"name\":\"{}\",\"version\":\"{}\"}},\"binary\":\"{}\",\"manifest\":\"{}\",\"icon\":{},\"includes\":[{}],\"metadata\":{{\"actions\":{},\"settings\":{},\"permissions\":{},\"permission_details\":[{}],\"staccato\":{}}},\"diagnostics\":[{}]}}",
            self.ok,
            self.target.as_str(),
            escape_json(&self.app.id),
            escape_json(&self.app.name),
            escape_json(&self.app.version),
            escape_json(&self.binary_name),
            escape_json(&self.manifest_path),
            optional_json_string(self.icon.as_deref()),
            includes,
            self.actions_count,
            self.settings_count,
            self.permissions_count,
            permissions,
            self.staccato.to_json(),
            diagnostics
        )
    }
}

impl StaccatoBundleMetadata {
    fn to_json(&self) -> String {
        format!(
            "{{\"command_palette\":{},\"workspace_sessions\":{},\"shell_tabs\":{},\"preferred_mode\":{},\"preferred_material\":{},\"preferred_chrome\":{}}}",
            self.command_palette,
            self.workspace_sessions,
            self.shell_tabs,
            optional_json_string(self.preferred_mode.as_deref()),
            optional_json_string(self.preferred_material.as_deref()),
            optional_json_string(self.preferred_chrome.as_deref())
        )
    }
}

fn bundle_includes(target: BundleTarget, inspection: &ManifestInspection) -> Vec<String> {
    let mut includes = vec![
        "binary".to_string(),
        "manifest".to_string(),
        "actions metadata".to_string(),
        "settings schema".to_string(),
        "permission metadata".to_string(),
        "desktop launcher metadata".to_string(),
    ];
    if inspection.app.icon.is_some() {
        includes.push("icon".to_string());
    }
    if target == BundleTarget::Staccato {
        includes.push("Staccato integration metadata".to_string());
    }
    includes
}

fn staccato_metadata(manifest: &Manifest) -> StaccatoBundleMetadata {
    StaccatoBundleMetadata {
        command_palette: bool_field(manifest, "command_palette"),
        workspace_sessions: bool_field(manifest, "workspace_sessions"),
        shell_tabs: bool_field(manifest, "shell_tabs"),
        preferred_mode: string_field(manifest, "preferred_mode"),
        preferred_material: string_field(manifest, "preferred_material"),
        preferred_chrome: string_field(manifest, "preferred_chrome"),
    }
}

fn bool_field(manifest: &Manifest, name: &str) -> bool {
    manifest
        .platform
        .staccato
        .get(name)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn string_field(manifest: &Manifest, name: &str) -> Option<String> {
    manifest
        .platform
        .staccato
        .get(name)
        .and_then(toml::Value::as_str)
        .map(str::to_string)
}

fn binary_name_from_app_id(app_id: &str) -> String {
    app_id
        .rsplit('.')
        .next()
        .unwrap_or(app_id)
        .replace('-', "_")
}

fn optional_json_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", escape_json(value)))
        .unwrap_or_else(|| "null".to_string())
}

fn escape_json(value: &str) -> String {
    let mut output = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch.is_control() => output.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => output.push(ch),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use stuk_manifest::{parse, validate};

    #[test]
    fn builds_bundle_plan_from_manifest() {
        let manifest = parse(
            r#"
[app]
id = "dev.example.notes"
name = "Notes"
version = "0.1.0"
icon = "assets/icon.svg"

[platform.staccato]
command_palette = true
workspace_sessions = true
shell_tabs = false
preferred_mode = "browser"
preferred_material = "maris"
preferred_chrome = "compact"

[actions.notes.new]
label = "New Note"
shortcut = "Ctrl+N"

[settings.appearance.theme]
type = "enum"
label = "Theme"
values = ["system", "dark"]
default = "system"

[permissions]
network = false
filesystem = "documents"
"#,
        )
        .unwrap();

        let diagnostics = validate(&manifest);
        let plan = BundlePlan::from_manifest(
            &manifest,
            &diagnostics,
            BundleTarget::Staccato,
            Path::new("Stuk.toml"),
        );

        assert!(plan.ok);
        assert_eq!(plan.binary_name, "notes");
        assert!(plan.includes.iter().any(|item| item == "icon"));
        assert!(plan.staccato.command_palette);
        assert_eq!(plan.staccato.preferred_chrome.as_deref(), Some("compact"));
        assert_eq!(plan.permissions_count, 2);
        assert!(plan.to_json().contains("\"permission_details\""));
    }
}
