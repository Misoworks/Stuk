use std::collections::BTreeMap;

use crate::{SettingDefinition, SettingsDiagnostic, SettingsSchema, SettingsSchemaError};

const SCHEMA_KEYS: &[&str] = &["type", "label", "default", "min", "max", "values"];

impl SettingsSchema {
    pub fn from_toml(
        settings: &BTreeMap<String, toml::Value>,
    ) -> Result<Self, Vec<SettingsDiagnostic>> {
        let mut parser = TomlSettingsParser {
            schema: Self::new(),
            diagnostics: Vec::new(),
        };

        for (name, value) in settings {
            parser.visit(name, &format!("settings.{name}"), value);
        }

        if parser.diagnostics.is_empty() {
            Ok(parser.schema)
        } else {
            Err(parser.diagnostics)
        }
    }
}

struct TomlSettingsParser {
    schema: SettingsSchema,
    diagnostics: Vec<SettingsDiagnostic>,
}

impl TomlSettingsParser {
    fn visit(&mut self, id: &str, path: &str, value: &toml::Value) {
        let Some(table) = value.as_table() else {
            self.diagnostics.push(SettingsDiagnostic::new(
                path,
                "Setting entry must be a table.",
            ));
            return;
        };

        if table.keys().any(|key| SCHEMA_KEYS.contains(&key.as_str())) {
            self.parse_definition(id, path, table);
            return;
        }

        for (child_name, child_value) in table {
            self.visit(
                &format!("{id}.{child_name}"),
                &format!("{path}.{child_name}"),
                child_value,
            );
        }
    }

    fn parse_definition(&mut self, id: &str, path: &str, table: &toml::Table) {
        let Some(kind) = self.string_field(path, table, "type") else {
            return;
        };
        let Some(label) = self.string_field(path, table, "label") else {
            return;
        };

        let definition = match kind {
            "boolean" | "bool" => self.boolean_definition(id, path, label, table),
            "number" => self.number_definition(id, path, label, table),
            "enum" => self.enum_definition(id, path, label, table),
            "text" | "string" => self.text_definition(id, path, label, table),
            other => {
                self.diagnostics.push(
                    SettingsDiagnostic::new(
                        format!("{path}.type"),
                        format!("Unsupported setting type `{other}`."),
                    )
                    .fix_hint("Use boolean, number, enum, or text."),
                );
                None
            }
        };

        let Some(definition) = definition else {
            return;
        };

        if let Err(error) = self.schema.insert(definition) {
            self.diagnostics.push(diagnostic_from_error(path, error));
        }
    }

    fn boolean_definition(
        &mut self,
        id: &str,
        path: &str,
        label: &str,
        table: &toml::Table,
    ) -> Option<SettingDefinition> {
        let Some(default) = table.get("default") else {
            self.missing_default(path);
            return None;
        };
        let Some(default) = default.as_bool() else {
            self.type_error(format!("{path}.default"), "boolean");
            return None;
        };
        Some(SettingDefinition::boolean(id, label, default))
    }

    fn number_definition(
        &mut self,
        id: &str,
        path: &str,
        label: &str,
        table: &toml::Table,
    ) -> Option<SettingDefinition> {
        let Some(default) = table.get("default") else {
            self.missing_default(path);
            return None;
        };
        let Some(default) = number_value(default) else {
            self.type_error(format!("{path}.default"), "number");
            return None;
        };
        let min = self.optional_number(path, table, "min");
        let max = self.optional_number(path, table, "max");
        Some(SettingDefinition::number(id, label, default, min, max))
    }

    fn enum_definition(
        &mut self,
        id: &str,
        path: &str,
        label: &str,
        table: &toml::Table,
    ) -> Option<SettingDefinition> {
        let values = self.string_array_field(path, table, "values")?;
        let default = self.string_field(path, table, "default")?;
        Some(SettingDefinition::enumeration(id, label, values, default))
    }

    fn text_definition(
        &mut self,
        id: &str,
        path: &str,
        label: &str,
        table: &toml::Table,
    ) -> Option<SettingDefinition> {
        let default = self.string_field(path, table, "default")?;
        Some(SettingDefinition::text(id, label, default))
    }

    fn string_field<'a>(
        &mut self,
        path: &str,
        table: &'a toml::Table,
        field: &str,
    ) -> Option<&'a str> {
        let Some(value) = table.get(field) else {
            self.diagnostics.push(SettingsDiagnostic::new(
                format!("{path}.{field}"),
                format!("Setting {field} is required."),
            ));
            return None;
        };
        let Some(value) = value.as_str() else {
            self.type_error(format!("{path}.{field}"), "string");
            return None;
        };
        Some(value)
    }

    fn string_array_field(
        &mut self,
        path: &str,
        table: &toml::Table,
        field: &str,
    ) -> Option<Vec<String>> {
        let Some(value) = table.get(field) else {
            self.diagnostics.push(SettingsDiagnostic::new(
                format!("{path}.{field}"),
                format!("Setting {field} is required."),
            ));
            return None;
        };
        let Some(values) = value.as_array() else {
            self.type_error(format!("{path}.{field}"), "array of strings");
            return None;
        };
        let mut strings = Vec::new();
        for (index, value) in values.iter().enumerate() {
            let Some(value) = value.as_str() else {
                self.type_error(format!("{path}.{field}.{index}"), "string");
                return None;
            };
            strings.push(value.to_string());
        }
        Some(strings)
    }

    fn optional_number(&mut self, path: &str, table: &toml::Table, field: &str) -> Option<f64> {
        let Some(value) = table.get(field) else {
            return None;
        };
        let Some(value) = number_value(value) else {
            self.type_error(format!("{path}.{field}"), "number");
            return None;
        };
        Some(value)
    }

    fn missing_default(&mut self, path: &str) {
        self.diagnostics.push(SettingsDiagnostic::new(
            format!("{path}.default"),
            "Setting default is required.",
        ));
    }

    fn type_error(&mut self, path: impl Into<String>, expected: &str) {
        self.diagnostics.push(SettingsDiagnostic::new(
            path,
            format!("Setting value must be a {expected}."),
        ));
    }
}

fn number_value(value: &toml::Value) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
}

fn diagnostic_from_error(path: &str, error: SettingsSchemaError) -> SettingsDiagnostic {
    let diagnostic = SettingsDiagnostic::new(path, error.to_string());
    match error {
        SettingsSchemaError::InvalidSettingId(_) => {
            diagnostic.fix_hint("Use an ID such as editor.font_size.")
        }
        _ => diagnostic,
    }
}
