use crate::inspect_manifest;
use stuk_manifest::parse;

#[test]
fn inspects_manifest_metadata() {
    let manifest = parse(
        r#"
[app]
id = "dev.example.preview"
name = "Preview"
version = "0.1.0"

[window.main]
title = "Preview"
width = 900
height = 600
chrome = "compact"

[actions.notes.new]
label = "New Note"
shortcut = "Ctrl+N"

[permissions]
network = false
filesystem = "documents"
"#,
    )
    .unwrap();

    let inspection = inspect_manifest(&manifest);

    assert!(inspection.ok);
    assert_eq!(inspection.app.id, "dev.example.preview");
    assert_eq!(inspection.windows[0].chrome.as_deref(), Some("compact"));
    assert_eq!(inspection.actions[0].id, "notes.new");
    assert_eq!(inspection.permissions_count, 2);
    assert!(
        inspection
            .permissions
            .iter()
            .any(|permission| permission.name == "filesystem" && permission.value == "documents")
    );
    assert!(inspection.to_json().contains("\"permission_details\""));
}

#[test]
fn derives_preview_targets_from_manifest() {
    let manifest = parse(
        r#"
[app]
id = "dev.example.preview"
name = "Preview"
version = "0.1.0"

[window.main]
title = "Preview"
width = 900
height = 600

[settings.appearance.theme]
type = "enum"
label = "Theme"
values = ["system", "dark"]
default = "system"
"#,
    )
    .unwrap();

    let inspection = inspect_manifest(&manifest);
    let previews = inspection.preview_descriptors(Some("dark"), Some("compact"));

    assert_eq!(previews.len(), 2);
    assert_eq!(previews[0].id, "window.main");
    assert_eq!(previews[0].width, 900);
    assert_eq!(previews[0].theme.as_deref(), Some("dark"));
    assert_eq!(previews[1].id, "settings");
    assert_eq!(previews[1].density.as_deref(), Some("compact"));
}
