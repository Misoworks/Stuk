use stuk_core::Element;
use stuk_settings::{SettingDefinition, SettingKind, SettingValue, SettingsSchema, SettingsStore};

use crate::{Button, HStack, Text, TextField, Toggle, VStack};

#[derive(Clone, Debug)]
pub struct SettingsPage {
    schema: SettingsSchema,
    values: SettingsStore,
    title: String,
    action_prefix: Option<String>,
}

impl SettingsPage {
    pub fn from_schema(schema: SettingsSchema) -> Self {
        Self {
            values: schema.defaults(),
            schema,
            title: "Settings".to_string(),
            action_prefix: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn action_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.action_prefix = Some(prefix.into());
        self
    }

    pub fn values(mut self, values: SettingsStore) -> Self {
        self.values = values;
        self
    }

    fn action_id(&self, setting_id: &str, value: Option<&str>) -> Option<String> {
        let prefix = self.action_prefix.as_ref()?;
        Some(match value {
            Some(value) => format!("{prefix}.{setting_id}.{value}"),
            None => format!("{prefix}.{setting_id}"),
        })
    }

    fn value<'a>(&'a self, definition: &'a SettingDefinition) -> &'a SettingValue {
        self.values
            .get(&definition.id)
            .unwrap_or(&definition.default)
    }
}

impl From<SettingsPage> for Element {
    fn from(page: SettingsPage) -> Self {
        let mut content = VStack::new()
            .padding(24.0)
            .spacing(12.0)
            .child(Text::title(page.title.clone()));

        for definition in page.schema.definitions() {
            let value = page.value(definition);
            let row: Element = match &definition.kind {
                SettingKind::Boolean => {
                    let mut toggle = Toggle::new(
                        definition.label.as_str(),
                        value.as_bool().unwrap_or_default(),
                    );
                    if let Some(action_id) = page.action_id(&definition.id, None) {
                        toggle = toggle.action(action_id);
                    }
                    toggle.into()
                }
                SettingKind::Number { .. } | SettingKind::Text => TextField::new(value.to_string())
                    .label(definition.label.as_str())
                    .into(),
                SettingKind::Enum { values } => {
                    let mut row = HStack::new()
                        .spacing(8.0)
                        .child(Text::new(definition.label.as_str()).muted());
                    let selected = value.as_text();
                    for value in values {
                        let mut button = if selected == Some(value.as_str()) {
                            Button::primary(value.as_str())
                        } else {
                            Button::secondary(value.as_str())
                        };
                        if let Some(action_id) =
                            page.action_id(&definition.id, Some(value.as_str()))
                        {
                            button = button.action(action_id);
                        }
                        row = row.child(button);
                    }
                    row.into()
                }
            };
            content = content.child(row);
        }

        if page.schema.is_empty() {
            content = content.child(Text::new("No settings defined.").muted());
        }

        content.into()
    }
}
