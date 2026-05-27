use std::collections::BTreeMap;

use thiserror::Error;

use crate::{SettingValue, SettingsSchema, SettingsSchemaError};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SettingsStore {
    values: BTreeMap<String, SettingValue>,
}

impl SettingsStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_schema(schema: &SettingsSchema) -> Self {
        let values = schema
            .definitions()
            .map(|definition| (definition.id.clone(), definition.default.clone()))
            .collect();
        Self { values }
    }

    pub fn get(&self, id: &str) -> Option<&SettingValue> {
        self.values.get(id)
    }

    pub fn set(
        &mut self,
        schema: &SettingsSchema,
        id: &str,
        value: impl Into<SettingValue>,
    ) -> Result<(), SettingsStoreError> {
        let definition = schema
            .get(id)
            .ok_or_else(|| SettingsStoreError::UnknownSetting(id.to_string()))?;
        let value = value.into();
        definition.validate_value(&value)?;
        self.values.insert(id.to_string(), value);
        Ok(())
    }

    pub fn get_bool(&self, id: &str) -> Option<bool> {
        self.get(id).and_then(SettingValue::as_bool)
    }

    pub fn get_number(&self, id: &str) -> Option<f64> {
        self.get(id).and_then(SettingValue::as_number)
    }

    pub fn get_text(&self, id: &str) -> Option<&str> {
        self.get(id).and_then(SettingValue::as_text)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &SettingValue)> {
        self.values.iter().map(|(id, value)| (id.as_str(), value))
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum SettingsStoreError {
    #[error("setting `{0}` is not defined")]
    UnknownSetting(String),
    #[error(transparent)]
    InvalidValue(#[from] SettingsSchemaError),
}
