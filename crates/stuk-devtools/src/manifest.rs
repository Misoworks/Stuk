use std::path::Path;

use stuk_manifest::{Diagnostic, DiagnosticLevel, Manifest, validate, validate_with_base_dir};

use crate::PreviewDescriptor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestInspection {
    pub ok: bool,
    pub app: AppInspection,
    pub windows: Vec<WindowInspection>,
    pub actions: Vec<ActionInspection>,
    pub settings_count: usize,
    pub permissions_count: usize,
    pub permissions: Vec<PermissionInspection>,
    pub diagnostics: Vec<DiagnosticInspection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppInspection {
    pub id: String,
    pub name: String,
    pub version: String,
    pub icon: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowInspection {
    pub name: String,
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub min_width: Option<u32>,
    pub min_height: Option<u32>,
    pub material: Option<String>,
    pub chrome: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionInspection {
    pub id: String,
    pub label: Option<String>,
    pub shortcut: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PermissionInspection {
    pub name: String,
    pub value: String,
    pub value_kind: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticInspection {
    pub level: String,
    pub path: String,
    pub message: String,
    pub fix_hint: Option<String>,
}

impl ManifestInspection {
    pub fn from_manifest(manifest: &Manifest, diagnostics: &[Diagnostic]) -> Self {
        let ok = diagnostics
            .iter()
            .all(|diagnostic| diagnostic.level != DiagnosticLevel::Error);

        Self {
            ok,
            app: AppInspection {
                id: manifest.app.id.clone(),
                name: manifest.app.name.clone(),
                version: manifest.app.version.clone(),
                icon: manifest.app.icon.clone(),
            },
            windows: manifest
                .window
                .iter()
                .map(|(name, window)| WindowInspection {
                    name: name.clone(),
                    title: window.title.clone(),
                    width: window.width,
                    height: window.height,
                    min_width: window.min_width,
                    min_height: window.min_height,
                    material: window.material.clone(),
                    chrome: window.chrome.clone(),
                })
                .collect(),
            actions: action_inspections(manifest),
            settings_count: manifest
                .settings_schema()
                .map(|schema| schema.len())
                .unwrap_or(0),
            permissions_count: manifest.permissions.len(),
            permissions: permission_inspections(manifest),
            diagnostics: diagnostics
                .iter()
                .cloned()
                .map(diagnostic_inspection)
                .collect(),
        }
    }

    pub fn to_text(&self) -> String {
        let mut output = String::new();
        output.push_str("App\n");
        output.push_str(&format!("  id: {}\n", self.app.id));
        output.push_str(&format!("  name: {}\n", self.app.name));
        output.push_str(&format!("  version: {}\n", self.app.version));
        if let Some(icon) = &self.app.icon {
            output.push_str(&format!("  icon: {icon}\n"));
        }
        output.push_str(&format!("Windows: {}\n", self.windows.len()));
        for window in &self.windows {
            output.push_str(&format!("  {}: {}\n", window.name, window.summary()));
        }
        output.push_str(&format!("Actions: {}\n", self.actions.len()));
        for action in &self.actions {
            let label = action.label.as_deref().unwrap_or("unlabeled");
            match &action.shortcut {
                Some(shortcut) => {
                    output.push_str(&format!("  {}: {label} ({shortcut})\n", action.id));
                }
                None => output.push_str(&format!("  {}: {label}\n", action.id)),
            }
        }
        output.push_str(&format!("Settings: {}\n", self.settings_count));
        output.push_str(&format!("Permissions: {}\n", self.permissions_count));
        for permission in &self.permissions {
            output.push_str(&format!(
                "  {}: {} ({})\n",
                permission.name, permission.value, permission.value_kind
            ));
        }
        if self.diagnostics.is_empty() {
            output.push_str("Diagnostics: 0\n");
        } else {
            output.push_str(&format!("Diagnostics: {}\n", self.diagnostics.len()));
            for diagnostic in &self.diagnostics {
                output.push_str(&format!(
                    "  {}: {}: {}\n",
                    diagnostic.level, diagnostic.path, diagnostic.message
                ));
            }
        }
        output
    }

    pub fn to_json(&self) -> String {
        let mut output = format!(
            "{{\"ok\":{},\"app\":{{\"id\":\"{}\",\"name\":\"{}\",\"version\":\"{}\",\"icon\":{}}},",
            self.ok,
            escape_json(&self.app.id),
            escape_json(&self.app.name),
            escape_json(&self.app.version),
            optional_json_string(self.app.icon.as_deref())
        );
        output.push_str("\"windows\":[");
        for (index, window) in self.windows.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            output.push_str(&window.to_json());
        }
        output.push_str("],\"actions\":[");
        for (index, action) in self.actions.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            output.push_str(&action.to_json());
        }
        output.push_str(&format!(
            "],\"settings\":{},\"permissions\":{},\"permission_details\":{},\"diagnostics\":{}}}",
            self.settings_count,
            self.permissions_count,
            permissions_json(&self.permissions),
            diagnostics_json(&self.diagnostics)
        ));
        output
    }

    pub fn preview_descriptors(
        &self,
        theme: Option<&str>,
        density: Option<&str>,
    ) -> Vec<PreviewDescriptor> {
        let mut previews = self
            .windows
            .iter()
            .map(|window| {
                let label = window.title.as_deref().unwrap_or(&window.name);
                preview_with_options(
                    PreviewDescriptor::new(format!("window.{}", window.name), label)
                        .size(window.width.unwrap_or(980), window.height.unwrap_or(680)),
                    theme,
                    density,
                )
            })
            .collect::<Vec<_>>();

        if self.settings_count > 0 {
            previews.push(preview_with_options(
                PreviewDescriptor::new("settings", "Settings").size(720, 680),
                theme,
                density,
            ));
        }

        previews
    }
}

impl WindowInspection {
    fn summary(&self) -> String {
        let mut parts = Vec::new();
        match (self.width, self.height) {
            (Some(width), Some(height)) => parts.push(format!("{width}x{height}")),
            _ => parts.push("size unspecified".to_string()),
        }
        if let (Some(min_width), Some(min_height)) = (self.min_width, self.min_height) {
            parts.push(format!("min {min_width}x{min_height}"));
        }
        if let Some(material) = &self.material {
            parts.push(format!("material={material}"));
        }
        if let Some(chrome) = &self.chrome {
            parts.push(format!("chrome={chrome}"));
        }
        parts.join(" ")
    }

    fn to_json(&self) -> String {
        format!(
            "{{\"name\":\"{}\",\"width\":{},\"height\":{},\"min_width\":{},\"min_height\":{},\"material\":{},\"chrome\":{}}}",
            escape_json(&self.name),
            optional_json_u32(self.width),
            optional_json_u32(self.height),
            optional_json_u32(self.min_width),
            optional_json_u32(self.min_height),
            optional_json_string(self.material.as_deref()),
            optional_json_string(self.chrome.as_deref())
        )
    }
}

impl ActionInspection {
    fn to_json(&self) -> String {
        let mut output = format!("{{\"id\":\"{}\"", escape_json(&self.id));
        if let Some(label) = &self.label {
            output.push_str(&format!(",\"label\":\"{}\"", escape_json(label)));
        }
        if let Some(shortcut) = &self.shortcut {
            output.push_str(&format!(",\"shortcut\":\"{}\"", escape_json(shortcut)));
        }
        output.push('}');
        output
    }
}

impl PermissionInspection {
    pub(crate) fn to_json(&self) -> String {
        format!(
            "{{\"name\":\"{}\",\"value\":\"{}\",\"kind\":\"{}\"}}",
            escape_json(&self.name),
            escape_json(&self.value),
            escape_json(&self.value_kind)
        )
    }
}

pub fn inspect_manifest(manifest: &Manifest) -> ManifestInspection {
    inspect_manifest_with_base_dir(manifest, None)
}

pub fn inspect_manifest_with_base_dir(
    manifest: &Manifest,
    base_dir: Option<&Path>,
) -> ManifestInspection {
    let diagnostics = match base_dir {
        Some(base_dir) => validate_with_base_dir(manifest, base_dir),
        None => validate(manifest),
    };
    ManifestInspection::from_manifest(manifest, &diagnostics)
}

fn action_inspections(manifest: &Manifest) -> Vec<ActionInspection> {
    let mut actions = Vec::new();
    for (name, value) in &manifest.actions {
        collect_action_inspections(name, value, &mut actions);
    }
    actions
}

fn permission_inspections(manifest: &Manifest) -> Vec<PermissionInspection> {
    manifest
        .permissions
        .iter()
        .map(|(name, value)| {
            let (value, value_kind) = permission_value(value);
            PermissionInspection {
                name: name.clone(),
                value,
                value_kind,
            }
        })
        .collect()
}

fn permission_value(value: &toml::Value) -> (String, String) {
    if let Some(value) = value.as_bool() {
        (value.to_string(), "boolean".to_string())
    } else if let Some(value) = value.as_str() {
        (value.to_string(), "string".to_string())
    } else {
        (value.to_string(), "unsupported".to_string())
    }
}

fn collect_action_inspections(id: &str, value: &toml::Value, actions: &mut Vec<ActionInspection>) {
    let Some(table) = value.as_table() else {
        return;
    };

    if table.contains_key("label") || table.contains_key("shortcut") {
        actions.push(ActionInspection {
            id: id.to_string(),
            label: table
                .get("label")
                .and_then(toml::Value::as_str)
                .map(str::to_string),
            shortcut: table
                .get("shortcut")
                .and_then(toml::Value::as_str)
                .map(str::to_string),
        });
        return;
    }

    for (child_name, child_value) in table {
        collect_action_inspections(&format!("{id}.{child_name}"), child_value, actions);
    }
}

fn diagnostic_inspection(diagnostic: Diagnostic) -> DiagnosticInspection {
    DiagnosticInspection {
        level: match diagnostic.level {
            DiagnosticLevel::Error => "error".to_string(),
            DiagnosticLevel::Warning => "warning".to_string(),
        },
        path: diagnostic.path,
        message: diagnostic.message,
        fix_hint: diagnostic.fix_hint,
    }
}

fn diagnostics_json(diagnostics: &[DiagnosticInspection]) -> String {
    let mut output = String::from("[");
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"level\":\"{}\",\"path\":\"{}\",\"message\":\"{}\"",
            diagnostic.level,
            escape_json(&diagnostic.path),
            escape_json(&diagnostic.message)
        ));
        if let Some(fix_hint) = &diagnostic.fix_hint {
            output.push_str(&format!(",\"fix_hint\":\"{}\"", escape_json(fix_hint)));
        }
        output.push('}');
    }
    output.push(']');
    output
}

fn permissions_json(permissions: &[PermissionInspection]) -> String {
    let mut output = String::from("[");
    for (index, permission) in permissions.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&permission.to_json());
    }
    output.push(']');
    output
}

fn optional_json_u32(value: Option<u32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
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

fn preview_with_options(
    mut preview: PreviewDescriptor,
    theme: Option<&str>,
    density: Option<&str>,
) -> PreviewDescriptor {
    if let Some(theme) = theme {
        preview = preview.theme(theme);
    }
    if let Some(density) = density {
        preview = preview.density(density);
    }
    preview
}
