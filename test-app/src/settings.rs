use stuk::prelude::*;

pub fn app_settings_schema() -> SettingsSchema {
    let mut schema = SettingsSchema::new();
    schema
        .insert(SettingDefinition::enumeration(
            "appearance.theme",
            "Theme",
            vec!["system".to_string(), "light".to_string(), "dark".to_string()],
            "system",
        ))
        .expect("settings schema should be valid");
    schema
        .insert(SettingDefinition::enumeration(
            "appearance.density",
            "Density",
            vec!["compact".to_string(), "regular".to_string(), "touch".to_string()],
            "regular",
        ))
        .expect("settings schema should be valid");
    schema
        .insert(SettingDefinition::boolean("sync.enabled", "Enable sync", false))
        .expect("settings schema should be valid");
    schema
}
