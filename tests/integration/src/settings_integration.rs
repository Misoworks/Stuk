#[allow(unused_imports)]
#[allow(unused_imports)]
use stuk_settings::{SettingDefinition, SettingValue, SettingsSchema, SettingsStore};

#[test]
fn settings_schema_builds_from_definitions() {
    let mut schema = SettingsSchema::new();
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
        .expect("should insert");
    schema
        .insert(SettingDefinition::boolean(
            "sync.enabled",
            "Enable sync",
            false,
        ))
        .expect("should insert");
    assert_eq!(schema.len(), 2);
    assert!(!schema.is_empty());
}

#[test]
fn settings_store_gets_and_sets_values() {
    let mut schema = SettingsSchema::new();
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
        .expect("should insert");
    schema
        .insert(SettingDefinition::boolean(
            "sync.enabled",
            "Enable sync",
            false,
        ))
        .expect("should insert");

    let mut store = SettingsStore::from_schema(&schema);
    store
        .set(
            &schema,
            "appearance.theme",
            SettingValue::Text("dark".to_string()),
        )
        .expect("should set");
    assert_eq!(store.get_text("appearance.theme"), Some("dark"));
    store
        .set(&schema, "sync.enabled", SettingValue::Boolean(true))
        .expect("should set");
    assert_eq!(store.get_bool("sync.enabled"), Some(true));
}

#[test]
fn settings_schema_defaults_produces_store() {
    let mut schema = SettingsSchema::new();
    schema
        .insert(SettingDefinition::boolean(
            "sync.enabled",
            "Enable sync",
            false,
        ))
        .expect("should insert");
    let store = schema.defaults();
    assert_eq!(store.get_bool("sync.enabled"), Some(false));
}
