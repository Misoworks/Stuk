use std::collections::BTreeMap;

use crate::{SettingDefinition, SettingKind, SettingsSchema};

#[test]
fn parses_nested_toml_schema() {
    let settings = nested_settings_table();
    let schema = SettingsSchema::from_toml(&settings).expect("settings schema should parse");

    assert_eq!(schema.len(), 3);
    assert_eq!(
        schema
            .get("appearance.theme")
            .map(|definition| &definition.kind),
        Some(&SettingKind::Enum {
            values: vec![
                "system".to_string(),
                "light".to_string(),
                "dark".to_string()
            ]
        })
    );
    assert_eq!(schema.defaults().get_number("editor.font_size"), Some(15.0));
    assert_eq!(schema.defaults().get_bool("sync.enabled"), Some(false));
}

#[test]
fn validates_runtime_store_values_against_schema() {
    let mut schema = SettingsSchema::new();
    schema
        .insert(SettingDefinition::number(
            "editor.font_size",
            "Editor font size",
            15.0,
            Some(10.0),
            Some(30.0),
        ))
        .unwrap();
    schema
        .insert(SettingDefinition::enumeration(
            "appearance.theme",
            "Theme",
            vec![
                "system".to_string(),
                "light".to_string(),
                "dark".to_string(),
            ],
            "system",
        ))
        .unwrap();

    let mut store = schema.defaults();
    store.set(&schema, "editor.font_size", 18.0).unwrap();
    store.set(&schema, "appearance.theme", "dark").unwrap();

    assert_eq!(store.get_number("editor.font_size"), Some(18.0));
    assert_eq!(store.get_text("appearance.theme"), Some("dark"));
    assert!(store.set(&schema, "editor.font_size", 48.0).is_err());
    assert!(store.set(&schema, "appearance.theme", "sepia").is_err());
}

#[test]
fn reports_invalid_toml_schema_paths() {
    let mut theme = toml::Table::new();
    theme.insert("type".to_string(), toml::Value::String("enum".to_string()));
    theme.insert(
        "label".to_string(),
        toml::Value::String("Theme".to_string()),
    );
    theme.insert(
        "values".to_string(),
        toml::Value::Array(vec![toml::Value::String("system".to_string())]),
    );
    theme.insert(
        "default".to_string(),
        toml::Value::String("dark".to_string()),
    );

    let mut appearance = toml::Table::new();
    appearance.insert("theme".to_string(), toml::Value::Table(theme));

    let mut settings = BTreeMap::new();
    settings.insert("appearance".to_string(), toml::Value::Table(appearance));

    let diagnostics = SettingsSchema::from_toml(&settings).unwrap_err();
    assert_eq!(diagnostics[0].path, "settings.appearance.theme");
    assert!(diagnostics[0].message.contains("default `dark`"));
}

fn nested_settings_table() -> BTreeMap<String, toml::Value> {
    let mut font_size = toml::Table::new();
    font_size.insert(
        "type".to_string(),
        toml::Value::String("number".to_string()),
    );
    font_size.insert(
        "label".to_string(),
        toml::Value::String("Editor font size".to_string()),
    );
    font_size.insert("default".to_string(), toml::Value::Integer(15));
    font_size.insert("min".to_string(), toml::Value::Integer(10));
    font_size.insert("max".to_string(), toml::Value::Integer(30));

    let mut editor = toml::Table::new();
    editor.insert("font_size".to_string(), toml::Value::Table(font_size));

    let mut theme = toml::Table::new();
    theme.insert("type".to_string(), toml::Value::String("enum".to_string()));
    theme.insert(
        "label".to_string(),
        toml::Value::String("Theme".to_string()),
    );
    theme.insert(
        "values".to_string(),
        toml::Value::Array(vec![
            toml::Value::String("system".to_string()),
            toml::Value::String("light".to_string()),
            toml::Value::String("dark".to_string()),
        ]),
    );
    theme.insert(
        "default".to_string(),
        toml::Value::String("system".to_string()),
    );

    let mut appearance = toml::Table::new();
    appearance.insert("theme".to_string(), toml::Value::Table(theme));

    let mut sync = toml::Table::new();
    sync.insert(
        "type".to_string(),
        toml::Value::String("boolean".to_string()),
    );
    sync.insert(
        "label".to_string(),
        toml::Value::String("Enable sync".to_string()),
    );
    sync.insert("default".to_string(), toml::Value::Boolean(false));

    let mut sync_group = toml::Table::new();
    sync_group.insert("enabled".to_string(), toml::Value::Table(sync));

    let mut settings = BTreeMap::new();
    settings.insert("appearance".to_string(), toml::Value::Table(appearance));
    settings.insert("editor".to_string(), toml::Value::Table(editor));
    settings.insert("sync".to_string(), toml::Value::Table(sync_group));
    settings
}
