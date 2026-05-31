#[allow(unused_imports)]
#[allow(unused_imports)]
use stuk_manifest::{DiagnosticLevel, parse, validate};

#[test]
fn full_manifest_round_trip() {
    let source = r#"
[app]
id = "com.example.notes"
name = "Notes"
version = "0.1.0"
icon = "assets/icon.svg"

[window.main]
title = "Notes"
width = 1100
height = 760
min_width = 720
min_height = 480
material = "maris"
chrome = "compact"

[platform.staccato]
command_palette = true
workspace_sessions = true
shell_tabs = true
preferred_mode = "browser"
preferred_material = "maris"
preferred_chrome = "compact"

[permissions]
network = false
filesystem = "documents"
notifications = true

[actions.notes.new]
label = "New Note"
shortcut = "Ctrl+N"

[actions.notes.search]
label = "Search"
shortcut = "Ctrl+F"

[settings.appearance.theme]
type = "enum"
label = "Theme"
values = ["system", "light", "dark"]
default = "system"
"#;
    let manifest = parse(source).expect("manifest should parse");
    assert_eq!(manifest.app.id, "com.example.notes");
    assert_eq!(manifest.app.name, "Notes");
    assert_eq!(manifest.window.len(), 1);
    assert!(manifest.window.contains_key("main"));
    assert_eq!(manifest.window["main"].width, Some(1100));
    assert_eq!(manifest.window["main"].height, Some(760));
    assert_eq!(manifest.window["main"].material.as_deref(), Some("maris"));
    assert_eq!(manifest.window["main"].chrome.as_deref(), Some("compact"));
    assert_eq!(manifest.permissions.len(), 3);
    assert_eq!(manifest.actions.len(), 1);
    assert!(manifest.actions.contains_key("notes"));
    assert_eq!(manifest.settings.len(), 1);

    let diagnostics = validate(&manifest);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn webview_manifest_validates() {
    let source = r#"
[app]
id = "net.aveid.klarkey"
name = "Klarkey"
version = "1.0.0"

[webview]
engine = "cef"
runtime = "shared-preferred"
entry = "ui/dist/index.html"
min_version = "126"
allow_user_install = true
allow_bundled = true

[webview.dev]
command = "npm run dev"
url = "http://localhost:5173"

[webview.security]
remote_content = false
devtools = "dev-only"
allow_eval = false
allow_node = false
csp = "default-src 'self'"
"#;
    let manifest = parse(source).expect("webview manifest should parse");
    assert_eq!(manifest.webview.engine.as_deref(), Some("cef"));
    assert_eq!(
        manifest.webview.runtime.as_deref(),
        Some("shared-preferred")
    );
    assert_eq!(
        manifest.webview.entry.as_deref(),
        Some("ui/dist/index.html")
    );
    assert_eq!(
        manifest.webview.security.devtools.as_deref(),
        Some("dev-only")
    );

    let diagnostics = validate(&manifest);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn webview_manifest_rejects_invalid_fields() {
    let source = r#"
[app]
id = "dev.example.bad"
name = "Bad"
version = "0.1.0"

[webview]
engine = "webkit"
runtime = "optional"
entry = ""

[webview.security]
devtools = "always"
remote_content = true
"#;
    let manifest = parse(source).expect("manifest should parse");
    let diagnostics = validate(&manifest);
    let error_paths: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .map(|d| d.path.as_str())
        .collect();
    assert!(error_paths.contains(&"webview.engine"));
    assert!(error_paths.contains(&"webview.runtime"));
    assert!(error_paths.contains(&"webview.entry"));
    assert!(error_paths.contains(&"webview.security.devtools"));
    assert!(
        diagnostics
            .iter()
            .any(|d| d.path == "webview.security.remote_content")
    );
}

#[test]
fn manifest_rejects_bad_app_id_and_version() {
    let source = r#"
[app]
id = "bad"
name = "Bad"
version = "not-semver"
"#;
    let manifest = parse(source).expect("manifest should parse");
    let diagnostics = validate(&manifest);
    assert!(diagnostics.iter().any(|d| d.path == "app.id"));
    assert!(diagnostics.iter().any(|d| d.path == "app.version"));
}

#[test]
fn manifest_detects_shortcut_conflicts() {
    let source = r#"
[app]
id = "com.example.app"
name = "App"
version = "0.1.0"

[actions.a]
label = "A"
shortcut = "Ctrl+S"

[actions.b]
label = "B"
shortcut = "Ctrl+S"
"#;
    let manifest = parse(source).expect("manifest should parse");
    let diagnostics = validate(&manifest);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.path == "actions.b.shortcut" && d.level == DiagnosticLevel::Error),
        "should detect shortcut conflict"
    );
}

mod actions_integration;
mod core_integration;
mod layout_integration;
mod settings_integration;
mod style_integration;
mod text_integration;
