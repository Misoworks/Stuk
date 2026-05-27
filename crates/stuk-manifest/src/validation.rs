use std::{collections::BTreeMap, path::Path};

use stuk_actions::{Shortcut, is_valid_action_id};
use stuk_settings::SettingsSchema;

use crate::{Diagnostic, DiagnosticLevel, Manifest, PlatformSection};

const VALID_MATERIALS: &[&str] = &[
    "solid",
    "surface",
    "surface_elevated",
    "window",
    "sidebar",
    "toolbar",
    "popover",
    "menu",
    "dialog",
    "maris",
    "luca",
];
const VALID_CHROMES: &[&str] = &["system", "stuk", "compact", "sidebar", "none"];
const BOOLEAN_PERMISSIONS: &[&str] = &[
    "network",
    "notifications",
    "camera",
    "microphone",
    "location",
    "clipboard",
    "background",
    "shell_integration",
    "command_execution",
    "screen_capture",
    "input_capture",
];
const FILESYSTEM_PERMISSIONS: &[&str] = &[
    "none",
    "documents",
    "downloads",
    "pictures",
    "music",
    "videos",
    "home",
    "all",
];
const STACCATO_BOOL_FIELDS: &[&str] = &["command_palette", "workspace_sessions", "shell_tabs"];
const STACCATO_MODES: &[&str] = &["app", "browser"];

pub fn validate(manifest: &Manifest) -> Vec<Diagnostic> {
    validate_inner(manifest, None)
}

pub fn validate_with_base_dir(manifest: &Manifest, base_dir: impl AsRef<Path>) -> Vec<Diagnostic> {
    validate_inner(manifest, Some(base_dir.as_ref()))
}

pub(crate) fn settings_diagnostics(
    diagnostics: Vec<stuk_settings::SettingsDiagnostic>,
) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .map(|diagnostic| Diagnostic {
            level: DiagnosticLevel::Error,
            path: diagnostic.path,
            message: diagnostic.message,
            fix_hint: diagnostic.fix_hint,
        })
        .collect()
}

fn validate_inner(manifest: &Manifest, base_dir: Option<&Path>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    if !is_reverse_dns(&manifest.app.id) {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            path: "app.id".to_string(),
            message: "App ID must use reverse DNS format.".to_string(),
            fix_hint: Some("Use an ID such as com.example.notes.".to_string()),
        });
    }

    if manifest.app.name.trim().is_empty() {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            path: "app.name".to_string(),
            message: "App name cannot be empty.".to_string(),
            fix_hint: None,
        });
    }

    if !is_semver_version(&manifest.app.version) {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            path: "app.version".to_string(),
            message: "App version must use semantic version format.".to_string(),
            fix_hint: Some("Use a version such as 0.1.0.".to_string()),
        });
    }

    if let Some(icon) = &manifest.app.icon {
        validate_icon(icon, base_dir, &mut diagnostics);
    }

    for (name, window) in &manifest.window {
        if let Some(material) = &window.material {
            validate_string_option(
                &format!("window.{name}.material"),
                material,
                VALID_MATERIALS,
                "Window material is not supported.",
                "Use maris, luca, surface, or window.",
                &mut diagnostics,
            );
        }
        if let Some(chrome) = &window.chrome {
            validate_string_option(
                &format!("window.{name}.chrome"),
                chrome,
                VALID_CHROMES,
                "Window chrome is not supported.",
                "Use system, stuk, compact, sidebar, or none.",
                &mut diagnostics,
            );
        }
        if let (Some(width), Some(min_width)) = (window.width, window.min_width) {
            if width < min_width {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Error,
                    path: format!("window.{name}.width"),
                    message: "Window width cannot be smaller than min_width.".to_string(),
                    fix_hint: None,
                });
            }
        }
        if let (Some(height), Some(min_height)) = (window.height, window.min_height) {
            if height < min_height {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Error,
                    path: format!("window.{name}.height"),
                    message: "Window height cannot be smaller than min_height.".to_string(),
                    fix_hint: None,
                });
            }
        }
    }

    validate_actions(&manifest.actions, &mut diagnostics);
    validate_settings(&manifest.settings, &mut diagnostics);
    validate_permissions(&manifest.permissions, &mut diagnostics);
    validate_platform(&manifest.platform, &mut diagnostics);

    diagnostics
}

fn validate_actions(actions: &BTreeMap<String, toml::Value>, diagnostics: &mut Vec<Diagnostic>) {
    let mut shortcuts = BTreeMap::<Shortcut, String>::new();
    for (name, value) in actions {
        validate_action_value(name, value, diagnostics, &mut shortcuts);
    }
}

fn validate_settings(settings: &BTreeMap<String, toml::Value>, diagnostics: &mut Vec<Diagnostic>) {
    if let Err(settings_diagnostics) = SettingsSchema::from_toml(settings) {
        diagnostics.extend(self::settings_diagnostics(settings_diagnostics));
    }
}

fn validate_icon(icon: &str, base_dir: Option<&Path>, diagnostics: &mut Vec<Diagnostic>) {
    if icon.trim().is_empty() {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            path: "app.icon".to_string(),
            message: "App icon path cannot be empty.".to_string(),
            fix_hint: Some("Use a path such as assets/icon.svg.".to_string()),
        });
        return;
    }

    if let Some(base_dir) = base_dir {
        let icon_path = Path::new(icon);
        let resolved = if icon_path.is_absolute() {
            icon_path.to_path_buf()
        } else {
            base_dir.join(icon_path)
        };
        if !resolved.is_file() {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Error,
                path: "app.icon".to_string(),
                message: format!("App icon `{icon}` does not exist."),
                fix_hint: Some("Add the icon file or update app.icon.".to_string()),
            });
        }
    }
}

fn validate_permissions(
    permissions: &BTreeMap<String, toml::Value>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (name, value) in permissions {
        if BOOLEAN_PERMISSIONS.contains(&name.as_str()) {
            if !value.is_bool() {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Error,
                    path: format!("permissions.{name}"),
                    message: "Permission value must be a boolean.".to_string(),
                    fix_hint: Some("Use true or false.".to_string()),
                });
            }
        } else if name == "filesystem" {
            if value.is_bool() {
                continue;
            }
            let Some(value) = value.as_str() else {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Error,
                    path: "permissions.filesystem".to_string(),
                    message: "Filesystem permission must be a boolean or string.".to_string(),
                    fix_hint: Some(
                        "Use false, true, documents, downloads, home, or all.".to_string(),
                    ),
                });
                continue;
            };
            validate_string_option(
                "permissions.filesystem",
                value,
                FILESYSTEM_PERMISSIONS,
                "Filesystem permission is not supported.",
                "Use false, true, none, documents, downloads, home, or all.",
                diagnostics,
            );
        } else {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Warning,
                path: format!("permissions.{name}"),
                message: "Permission is not recognized by this Stuk version.".to_string(),
                fix_hint: None,
            });
        }
    }
}

fn validate_platform(platform: &PlatformSection, diagnostics: &mut Vec<Diagnostic>) {
    for (name, value) in &platform.staccato {
        if STACCATO_BOOL_FIELDS.contains(&name.as_str()) {
            if !value.is_bool() {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Error,
                    path: format!("platform.staccato.{name}"),
                    message: "Staccato platform feature flag must be a boolean.".to_string(),
                    fix_hint: Some("Use true or false.".to_string()),
                });
            }
        } else if name == "preferred_mode" {
            validate_string_field(
                &format!("platform.staccato.{name}"),
                value,
                STACCATO_MODES,
                "Preferred Staccato mode is not supported.",
                "Use app or browser.",
                diagnostics,
            );
        } else if name == "preferred_material" {
            validate_string_field(
                &format!("platform.staccato.{name}"),
                value,
                VALID_MATERIALS,
                "Preferred material is not supported.",
                "Use maris, luca, surface, or window.",
                diagnostics,
            );
        } else if name == "preferred_chrome" {
            validate_string_field(
                &format!("platform.staccato.{name}"),
                value,
                VALID_CHROMES,
                "Preferred chrome is not supported.",
                "Use system, stuk, compact, sidebar, or none.",
                diagnostics,
            );
        } else {
            diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Warning,
                path: format!("platform.staccato.{name}"),
                message: "Staccato platform feature is not recognized by this Stuk version."
                    .to_string(),
                fix_hint: None,
            });
        }
    }
}

fn validate_string_field(
    path: &str,
    value: &toml::Value,
    valid: &[&str],
    message: &str,
    fix_hint: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(value) = value.as_str() else {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            path: path.to_string(),
            message: "Value must be a string.".to_string(),
            fix_hint: Some(fix_hint.to_string()),
        });
        return;
    };
    validate_string_option(path, value, valid, message, fix_hint, diagnostics);
}

fn validate_string_option(
    path: &str,
    value: &str,
    valid: &[&str],
    message: &str,
    fix_hint: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !valid.contains(&value) {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            path: path.to_string(),
            message: message.to_string(),
            fix_hint: Some(fix_hint.to_string()),
        });
    }
}

fn validate_action_value(
    id: &str,
    value: &toml::Value,
    diagnostics: &mut Vec<Diagnostic>,
    shortcuts: &mut BTreeMap<Shortcut, String>,
) {
    let Some(table) = value.as_table() else {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            path: format!("actions.{id}"),
            message: "Action entry must be a table.".to_string(),
            fix_hint: None,
        });
        return;
    };

    if table.contains_key("label") || table.contains_key("shortcut") {
        validate_action_leaf(id, table, diagnostics, shortcuts);
        return;
    }

    for (child_name, child_value) in table {
        validate_action_value(
            &format!("{id}.{child_name}"),
            child_value,
            diagnostics,
            shortcuts,
        );
    }
}

fn validate_action_leaf(
    id: &str,
    table: &toml::Table,
    diagnostics: &mut Vec<Diagnostic>,
    shortcuts: &mut BTreeMap<Shortcut, String>,
) {
    if !is_valid_action_id(id) {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            path: format!("actions.{id}"),
            message: "Action ID must be lowercase dot-separated segments.".to_string(),
            fix_hint: Some("Use an ID such as notes.new.".to_string()),
        });
    }

    match table.get("label").and_then(toml::Value::as_str) {
        Some(label) if !label.trim().is_empty() => {}
        _ => diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            path: format!("actions.{id}.label"),
            message: "Action label must be a non-empty string.".to_string(),
            fix_hint: None,
        }),
    }

    let Some(shortcut_value) = table.get("shortcut") else {
        return;
    };
    let Some(shortcut_text) = shortcut_value.as_str() else {
        diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            path: format!("actions.{id}.shortcut"),
            message: "Action shortcut must be a string.".to_string(),
            fix_hint: Some("Use a shortcut such as Ctrl+N.".to_string()),
        });
        return;
    };

    match Shortcut::parse(shortcut_text) {
        Ok(shortcut) => {
            if let Some(existing) = shortcuts.insert(shortcut.clone(), id.to_string()) {
                diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Error,
                    path: format!("actions.{id}.shortcut"),
                    message: format!("Shortcut {shortcut} conflicts with action {existing}."),
                    fix_hint: None,
                });
            }
        }
        Err(error) => diagnostics.push(Diagnostic {
            level: DiagnosticLevel::Error,
            path: format!("actions.{id}.shortcut"),
            message: format!("Invalid shortcut: {error}"),
            fix_hint: Some("Use a shortcut such as Ctrl+N.".to_string()),
        }),
    }
}

fn is_reverse_dns(id: &str) -> bool {
    let parts = id.split('.').collect::<Vec<_>>();
    parts.len() >= 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        })
}

fn is_semver_version(version: &str) -> bool {
    let core = version
        .split_once('-')
        .map(|(core, _)| core)
        .unwrap_or(version)
        .split_once('+')
        .map(|(core, _)| core)
        .unwrap_or(version);
    let parts = core.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.chars().all(|ch| ch.is_ascii_digit())
                && (part == &"0" || !part.starts_with('0'))
        })
}
