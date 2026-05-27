use std::{collections::BTreeMap, fmt};

use thiserror::Error;

use crate::store::SettingsStore;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SettingsSchema {
    definitions: BTreeMap<String, SettingDefinition>,
}

impl SettingsSchema {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, definition: SettingDefinition) -> Result<(), SettingsSchemaError> {
        definition.validate()?;
        if self.definitions.contains_key(&definition.id) {
            return Err(SettingsSchemaError::DuplicateSetting(definition.id));
        }
        self.definitions.insert(definition.id.clone(), definition);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&SettingDefinition> {
        self.definitions.get(id)
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn definitions(&self) -> impl Iterator<Item = &SettingDefinition> {
        self.definitions.values()
    }

    pub fn defaults(&self) -> SettingsStore {
        SettingsStore::from_schema(self)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SettingDefinition {
    pub id: String,
    pub label: String,
    pub kind: SettingKind,
    pub default: SettingValue,
}

impl SettingDefinition {
    pub fn boolean(id: impl Into<String>, label: impl Into<String>, default: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: SettingKind::Boolean,
            default: SettingValue::Boolean(default),
        }
    }

    pub fn number(
        id: impl Into<String>,
        label: impl Into<String>,
        default: f64,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: SettingKind::Number { min, max },
            default: SettingValue::Number(default),
        }
    }

    pub fn enumeration(
        id: impl Into<String>,
        label: impl Into<String>,
        values: Vec<String>,
        default: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: SettingKind::Enum { values },
            default: SettingValue::Text(default.into()),
        }
    }

    pub fn text(
        id: impl Into<String>,
        label: impl Into<String>,
        default: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: SettingKind::Text,
            default: SettingValue::Text(default.into()),
        }
    }

    pub fn validate(&self) -> Result<(), SettingsSchemaError> {
        validate_setting_id(&self.id)?;
        if self.label.trim().is_empty() {
            return Err(SettingsSchemaError::EmptyLabel(self.id.clone()));
        }
        if let SettingKind::Number {
            min: Some(min),
            max: Some(max),
        } = &self.kind
        {
            if max < min {
                return Err(SettingsSchemaError::InvalidRange {
                    id: self.id.clone(),
                    min: *min,
                    max: *max,
                });
            }
        }
        if let SettingKind::Enum { values } = &self.kind {
            if values.is_empty() {
                return Err(SettingsSchemaError::EmptyEnum(self.id.clone()));
            }
            let mut seen = BTreeMap::<&str, ()>::new();
            for value in values {
                if value.trim().is_empty() {
                    return Err(SettingsSchemaError::EmptyEnumValue(self.id.clone()));
                }
                if seen.insert(value.as_str(), ()).is_some() {
                    return Err(SettingsSchemaError::DuplicateEnumValue {
                        id: self.id.clone(),
                        value: value.clone(),
                    });
                }
            }
        }
        self.validate_value(&self.default)
    }

    pub fn validate_value(&self, value: &SettingValue) -> Result<(), SettingsSchemaError> {
        match (&self.kind, value) {
            (SettingKind::Boolean, SettingValue::Boolean(_)) => Ok(()),
            (SettingKind::Text, SettingValue::Text(_)) => Ok(()),
            (SettingKind::Number { min, max }, SettingValue::Number(number)) => {
                if let Some(min) = min {
                    if number < min {
                        return Err(SettingsSchemaError::NumberBelowMin {
                            id: self.id.clone(),
                            value: *number,
                            min: *min,
                        });
                    }
                }
                if let Some(max) = max {
                    if number > max {
                        return Err(SettingsSchemaError::NumberAboveMax {
                            id: self.id.clone(),
                            value: *number,
                            max: *max,
                        });
                    }
                }
                Ok(())
            }
            (SettingKind::Enum { values }, SettingValue::Text(text)) => {
                if values.iter().any(|value| value == text) {
                    Ok(())
                } else {
                    Err(SettingsSchemaError::InvalidEnumValue {
                        id: self.id.clone(),
                        value: text.clone(),
                    })
                }
            }
            (kind, _) => Err(SettingsSchemaError::TypeMismatch {
                id: self.id.clone(),
                expected: kind.type_name(),
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SettingKind {
    Boolean,
    Number { min: Option<f64>, max: Option<f64> },
    Enum { values: Vec<String> },
    Text,
}

impl SettingKind {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Number { .. } => "number",
            Self::Enum { .. } => "enum",
            Self::Text => "text",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SettingValue {
    Boolean(bool),
    Number(f64),
    Text(String),
}

impl SettingValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            _ => None,
        }
    }
}

impl fmt::Display for SettingValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(value) => write!(formatter, "{value}"),
            Self::Number(value) => write!(formatter, "{value}"),
            Self::Text(value) => write!(formatter, "{value}"),
        }
    }
}

impl From<bool> for SettingValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<f64> for SettingValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<i32> for SettingValue {
    fn from(value: i32) -> Self {
        Self::Number(value.into())
    }
}

impl From<u32> for SettingValue {
    fn from(value: u32) -> Self {
        Self::Number(value.into())
    }
}

impl From<String> for SettingValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for SettingValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsDiagnostic {
    pub path: String,
    pub message: String,
    pub fix_hint: Option<String>,
}

impl SettingsDiagnostic {
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            fix_hint: None,
        }
    }

    pub fn fix_hint(mut self, fix_hint: impl Into<String>) -> Self {
        self.fix_hint = Some(fix_hint.into());
        self
    }
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum SettingsSchemaError {
    #[error("setting `{0}` has an invalid ID")]
    InvalidSettingId(String),
    #[error("setting `{0}` is already defined")]
    DuplicateSetting(String),
    #[error("setting `{0}` label cannot be empty")]
    EmptyLabel(String),
    #[error("setting `{id}` expects a {expected} value")]
    TypeMismatch { id: String, expected: &'static str },
    #[error("setting `{id}` value {value} is below minimum {min}")]
    NumberBelowMin { id: String, value: f64, min: f64 },
    #[error("setting `{id}` value {value} is above maximum {max}")]
    NumberAboveMax { id: String, value: f64, max: f64 },
    #[error("setting `{id}` max {max} cannot be smaller than min {min}")]
    InvalidRange { id: String, min: f64, max: f64 },
    #[error("setting `{0}` enum must define at least one value")]
    EmptyEnum(String),
    #[error("setting `{0}` enum values cannot be empty")]
    EmptyEnumValue(String),
    #[error("setting `{id}` enum value `{value}` is duplicated")]
    DuplicateEnumValue { id: String, value: String },
    #[error("setting `{id}` enum default `{value}` is not in values")]
    InvalidEnumValue { id: String, value: String },
}

pub fn validate_setting_id(id: &str) -> Result<(), SettingsSchemaError> {
    let parts = id.split('.').collect::<Vec<_>>();
    if parts.len() < 2
        || parts.iter().any(|part| {
            part.is_empty()
                || !part.chars().all(|ch| {
                    ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-'
                })
        })
    {
        return Err(SettingsSchemaError::InvalidSettingId(id.to_string()));
    }
    Ok(())
}

pub fn is_valid_setting_id(id: &str) -> bool {
    validate_setting_id(id).is_ok()
}
