use stuk_manifest::{parse, validate};

#[test]
fn manifest_round_trip_with_all_sections() {
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
camera = false

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

[settings.appearance.density]
type = "enum"
label = "Density"
values = ["compact", "regular", "touch"]
default = "regular"

[settings.sync.enabled]
type = "boolean"
label = "Enable sync"
default = false
"#;
    let manifest = parse(source).expect("manifest should parse");
    assert_eq!(manifest.app.id, "com.example.notes");
    assert_eq!(manifest.app.name, "Notes");
    assert_eq!(manifest.window.len(), 1);
    assert!(manifest.window.contains_key("main"));
    assert!(manifest.permissions.contains_key("network"));
    assert!(manifest.permissions.contains_key("filesystem"));
    assert!(manifest.actions.contains_key("notes.new"));

    let diagnostics = validate(&manifest);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn manifest_webview_config_parses_and_validates() {
    let source = r#"
[app]
id = "net.aveid.klarkey"
name = "Klarkey"
version = "0.1.0"

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
    assert_eq!(manifest.webview.runtime.as_deref(), Some("shared-preferred"));
    assert_eq!(manifest.webview.entry.as_deref(), Some("ui/dist/index.html"));
    assert_eq!(
        manifest.webview.security.devtools.as_deref(),
        Some("dev-only")
    );

    let diagnostics = validate(&manifest);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn manifest_rejects_invalid_webview_fields() {
    let source = r#"
[app]
id = "dev.example.bad"
name = "Bad"
version = "0.1.0"

[webview]
engine = "webkit"
runtime = "optional"

[webview.security]
devtools = "always"
remote_content = true
"#;
    let manifest = parse(source).expect("manifest should parse");
    let diagnostics = validate(&manifest);
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == stuk_manifest::DiagnosticLevel::Error)
        .collect();
    assert!(!errors.is_empty(), "should have validation errors");
}