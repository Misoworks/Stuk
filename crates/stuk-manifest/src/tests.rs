use std::{fs, path::Path};

use crate::{DiagnosticLevel, parse, validate, validate_with_base_dir};

#[test]
fn validates_manifest_fields_from_spec() {
    let manifest = parse(
        r#"
[app]
id = "dev.example.bad"
name = "Bad"
version = "01.0"
icon = "missing.svg"

[window.main]
width = 640
height = 480
min_width = 720
min_height = 360
material = "fog"
chrome = "giant"
background_effect = "sparkles"

[platform.staccato]
command_palette = "yes"
preferred_mode = "terminal"

[permissions]
network = "yes"
filesystem = "root"
unknown = true
"#,
    )
    .unwrap();

    let diagnostics = validate_with_base_dir(&manifest, Path::new("/tmp"));
    assert_has_error(&diagnostics, "app.version");
    assert_has_error(&diagnostics, "app.icon");
    assert_has_error(&diagnostics, "window.main.width");
    assert_has_error(&diagnostics, "window.main.material");
    assert_has_error(&diagnostics, "window.main.chrome");
    assert_has_error(&diagnostics, "window.main.background_effect");
    assert_has_error(&diagnostics, "platform.staccato.command_palette");
    assert_has_error(&diagnostics, "platform.staccato.preferred_mode");
    assert_has_error(&diagnostics, "permissions.network");
    assert_has_error(&diagnostics, "permissions.filesystem");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.level == DiagnosticLevel::Warning && diagnostic.path == "permissions.unknown"
    }));
}

#[test]
fn accepts_existing_icon_and_supported_values() {
    let dir = std::env::temp_dir().join(format!("stuk-manifest-{}", std::process::id()));
    let assets = dir.join("assets");
    fs::create_dir_all(&assets).unwrap();
    fs::write(assets.join("icon.svg"), "<svg/>").unwrap();

    let manifest = parse(
        r#"
[app]
id = "dev.example.good"
name = "Good"
version = "1.2.3"
icon = "assets/icon.svg"

[window.main]
width = 900
height = 600
min_width = 420
min_height = 320
material = "maris"
chrome = "compact"
transparent = true
background_effect = "mica"

[platform.staccato]
command_palette = true
preferred_mode = "browser"
preferred_material = "luca"
preferred_chrome = "sidebar"

[permissions]
network = false
filesystem = false
notifications = true
location = false
clipboard = true
shell_integration = false
command_execution = false
screen_capture = false
input_capture = false
"#,
    )
    .unwrap();

    let diagnostics = validate_with_base_dir(&manifest, &dir);
    let _ = fs::remove_dir_all(&dir);

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn pathless_validation_keeps_icon_check_structural() {
    let manifest = parse(
        r#"
[app]
id = "dev.example.app"
name = "App"
version = "0.1.0"
icon = "assets/icon.svg"
"#,
    )
    .unwrap();

    assert!(validate(&manifest).is_empty());
}

fn assert_has_error(diagnostics: &[crate::Diagnostic], path: &str) {
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.level == DiagnosticLevel::Error
                && diagnostic.path == path),
        "missing error at {path}: {diagnostics:?}"
    );
}

#[test]
fn validates_webview_fields() {
    let manifest = parse(
        r#"
[app]
id = "dev.example.app"
name = "App"
version = "0.1.0"

[webview]
engine = "unknown"
runtime = "invalid"
entry = ""

[webview.security]
devtools = "always"
remote_content = true
"#,
    )
    .unwrap();

    let diagnostics = validate(&manifest);
    assert_has_error(&diagnostics, "webview.engine");
    assert_has_error(&diagnostics, "webview.runtime");
    assert_has_error(&diagnostics, "webview.entry");
    assert_has_error(&diagnostics, "webview.security.devtools");
    assert!(
        diagnostics
            .iter()
            .any(|d| d.path == "webview.security.remote_content")
    );
}

#[test]
fn accepts_valid_webview_config() {
    let manifest = parse(
        r#"
[app]
id = "dev.example.app"
name = "App"
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
"#,
    )
    .unwrap();

    let diagnostics = validate(&manifest);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}
