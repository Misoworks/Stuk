use stuk_core::Element;

use crate::{Text, TextField};

#[derive(Clone, Debug)]
pub struct Label {
    text: String,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl From<Label> for Element {
    fn from(label: Label) -> Self {
        Text::new(label.text).into()
    }
}

#[derive(Clone, Debug)]
pub struct SelectableText {
    text: String,
}

impl SelectableText {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl From<SelectableText> for Element {
    fn from(text: SelectableText) -> Self {
        Text::new(text.text).into()
    }
}

#[derive(Clone, Debug)]
pub struct SearchField {
    field: TextField,
}

impl SearchField {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            field: TextField::new(value).label("Search"),
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.field = self.field.placeholder(placeholder);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.field = self.field.disabled(disabled);
        self
    }
}

impl From<SearchField> for Element {
    fn from(search: SearchField) -> Self {
        search.field.into()
    }
}

#[derive(Clone, Debug)]
pub struct TextArea {
    field: TextField,
}

impl TextArea {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            field: TextField::new(value),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.field = self.field.label(label);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.field = self.field.placeholder(placeholder);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.field = self.field.disabled(disabled);
        self
    }
}

impl From<TextArea> for Element {
    fn from(area: TextArea) -> Self {
        area.field.into()
    }
}

#[derive(Clone, Debug)]
pub struct PasswordField {
    value: String,
    label: Option<String>,
    placeholder: String,
    disabled: bool,
}

impl PasswordField {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: None,
            placeholder: String::new(),
            disabled: false,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl From<PasswordField> for Element {
    fn from(field: PasswordField) -> Self {
        let masked = "*".repeat(field.value.chars().count());
        let mut text_field = TextField::new(masked)
            .placeholder(field.placeholder)
            .disabled(field.disabled);
        if let Some(label) = field.label {
            text_field = text_field.label(label);
        }
        text_field.into()
    }
}

#[derive(Clone, Debug)]
pub struct TextEditorLite {
    area: TextArea,
}

impl TextEditorLite {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            area: TextArea::new(value).label("Editor"),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.area = self.area.label(label);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.area = self.area.placeholder(placeholder);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.area = self.area.disabled(disabled);
        self
    }
}

impl From<TextEditorLite> for Element {
    fn from(editor: TextEditorLite) -> Self {
        editor.area.into()
    }
}
