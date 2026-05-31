#[allow(unused_imports)]
use stuk_actions::{ActionDescriptor, ActionRegistry, Modifiers, Shortcut, is_valid_action_id};

#[test]
fn shortcut_parses_known_combinations() {
    let shortcut = Shortcut::new(
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
        "N",
    );
    assert_eq!(shortcut.key, "N");
    assert!(shortcut.modifiers.ctrl);
}

#[test]
fn action_registry_detects_conflicts() {
    let mut registry = ActionRegistry::new();
    let shortcut = Shortcut::new(
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
        "N",
    );
    registry
        .register(ActionDescriptor::new("app.new", "New").shortcut(shortcut.clone()))
        .unwrap();
    let result =
        registry.register(ActionDescriptor::new("notes.new", "New Note").shortcut(shortcut));
    assert!(result.is_err());
}

#[test]
fn action_descriptor_builder() {
    let desc = ActionDescriptor::new("notes.new", "New Note")
        .shortcut(Shortcut::new(
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
            "N",
        ))
        .category("Notes");
    assert_eq!(desc.id, "notes.new");
    assert_eq!(desc.label, "New Note");
    assert_eq!(desc.category, Some("Notes".to_string()));
}

#[test]
fn action_id_validation() {
    assert!(is_valid_action_id("notes.new"));
    assert!(is_valid_action_id("app.settings"));
    assert!(!is_valid_action_id("INVALID"));
}
