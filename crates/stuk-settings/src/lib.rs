mod schema;
mod store;
mod toml_schema;

pub use schema::{
    SettingDefinition, SettingKind, SettingValue, SettingsDiagnostic, SettingsSchema,
    SettingsSchemaError, is_valid_setting_id, validate_setting_id,
};
pub use store::{SettingsStore, SettingsStoreError};

#[cfg(test)]
mod tests;
