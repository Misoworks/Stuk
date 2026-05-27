use std::{collections::BTreeMap, fmt, str::FromStr};

use stuk_layout::Rect;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionDescriptor {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub shortcut: Option<Shortcut>,
    pub category: Option<String>,
    pub enabled: bool,
    pub visible: bool,
}

impl ActionDescriptor {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            shortcut: None,
            category: None,
            enabled: true,
            visible: true,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn shortcut(mut self, shortcut: Shortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn shortcut_str(mut self, shortcut: &str) -> Result<Self, ShortcutParseError> {
        self.shortcut = Some(shortcut.parse()?);
        Ok(self)
    }

    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct ActionRegistry {
    actions: BTreeMap<String, ActionDescriptor>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, action: ActionDescriptor) -> Result<(), ActionRegistryError> {
        validate_action_id(&action.id)?;
        if self.actions.contains_key(&action.id) {
            return Err(ActionRegistryError::DuplicateAction(action.id));
        }
        if let Some(shortcut) = &action.shortcut {
            if let Some(existing) = self.find_by_shortcut(shortcut) {
                return Err(ActionRegistryError::ShortcutConflict {
                    shortcut: shortcut.clone(),
                    first_action: existing.id.clone(),
                    second_action: action.id.clone(),
                });
            }
        }

        self.actions.insert(action.id.clone(), action);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&ActionDescriptor> {
        self.actions.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.actions.contains_key(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ActionDescriptor> {
        self.actions.values()
    }

    pub fn from_actions(
        actions: impl IntoIterator<Item = ActionDescriptor>,
    ) -> Result<Self, ActionRegistryError> {
        let mut registry = ActionRegistry::new();
        for action in actions {
            registry.register(action)?;
        }
        Ok(registry)
    }

    fn find_by_shortcut(&self, shortcut: &Shortcut) -> Option<&ActionDescriptor> {
        self.actions
            .values()
            .find(|action| action.shortcut.as_ref() == Some(shortcut))
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ActionRegistryError {
    #[error("action `{0}` has an invalid ID")]
    InvalidActionId(String),
    #[error("action `{0}` is already registered")]
    DuplicateAction(String),
    #[error(
        "shortcut `{shortcut}` is already used by `{first_action}` and cannot be used by `{second_action}`"
    )]
    ShortcutConflict {
        shortcut: Shortcut,
        first_action: String,
        second_action: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Shortcut {
    pub modifiers: Modifiers,
    pub key: String,
}

impl Shortcut {
    pub fn new(modifiers: Modifiers, key: impl Into<String>) -> Self {
        let key = key.into();
        Self {
            modifiers,
            key: normalize_key(&key),
        }
    }

    pub fn parse(value: &str) -> Result<Self, ShortcutParseError> {
        value.parse()
    }
}

impl FromStr for Shortcut {
    type Err = ShortcutParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut modifiers = Modifiers::default();
        let mut key = None;

        for part in value
            .split('+')
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => modifiers.ctrl = true,
                "alt" | "option" => modifiers.alt = true,
                "shift" => modifiers.shift = true,
                "cmd" | "command" | "meta" | "super" => modifiers.meta = true,
                _ if key.is_none() => key = Some(normalize_key(part)),
                _ => return Err(ShortcutParseError::Invalid(value.to_string())),
            }
        }

        let Some(key) = key else {
            return Err(ShortcutParseError::MissingKey(value.to_string()));
        };

        Ok(Self { modifiers, key })
    }
}

impl fmt::Display for Shortcut {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.modifiers.ctrl {
            parts.push("Ctrl");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }
        if self.modifiers.meta {
            parts.push("Meta");
        }
        parts.push(&self.key);
        write!(formatter, "{}", parts.join("+"))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActionHitRegion {
    pub rect: Rect,
    pub action_id: String,
    pub enabled: bool,
}

impl ActionHitRegion {
    pub fn new(rect: Rect, action_id: impl Into<String>) -> Self {
        Self {
            rect,
            action_id: action_id.into(),
            enabled: true,
        }
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.rect.x
            && x <= self.rect.x + self.rect.width
            && y >= self.rect.y
            && y <= self.rect.y + self.rect.height
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ShortcutParseError {
    #[error("shortcut `{0}` is missing a key")]
    MissingKey(String),
    #[error("shortcut `{0}` is invalid")]
    Invalid(String),
}

pub fn validate_action_id(id: &str) -> Result<(), ActionRegistryError> {
    let parts = id.split('.').collect::<Vec<_>>();
    if parts.len() < 2
        || parts.iter().any(|part| {
            part.is_empty()
                || !part.chars().all(|ch| {
                    ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-'
                })
        })
    {
        return Err(ActionRegistryError::InvalidActionId(id.to_string()));
    }
    Ok(())
}

pub fn is_valid_action_id(id: &str) -> bool {
    validate_action_id(id).is_ok()
}

fn normalize_key(key: &str) -> String {
    let key = key.trim();
    match key.to_ascii_lowercase().as_str() {
        "return" | "enter" => "Enter".to_string(),
        "esc" | "escape" => "Escape".to_string(),
        "space" | "spacebar" | " " => "Space".to_string(),
        "tab" => "Tab".to_string(),
        "backspace" => "Backspace".to_string(),
        "delete" | "del" => "Delete".to_string(),
        "insert" | "ins" => "Insert".to_string(),
        "home" => "Home".to_string(),
        "end" => "End".to_string(),
        "pageup" | "page_up" | "page-up" => "PageUp".to_string(),
        "pagedown" | "page_down" | "page-down" => "PageDown".to_string(),
        "arrowup" | "up" => "ArrowUp".to_string(),
        "arrowdown" | "down" => "ArrowDown".to_string(),
        "arrowleft" | "left" => "ArrowLeft".to_string(),
        "arrowright" | "right" => "ArrowRight".to_string(),
        value if is_function_key(value) => value.to_ascii_uppercase(),
        _ if key.chars().count() == 1 => key.to_ascii_uppercase(),
        _ => key.to_string(),
    }
}

fn is_function_key(key: &str) -> bool {
    let Some(number) = key.strip_prefix('f') else {
        return false;
    };

    number
        .parse::<u8>()
        .is_ok_and(|number| (1..=24).contains(&number))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_shortcuts() {
        let shortcut: Shortcut = "Ctrl+Shift+N".parse().unwrap();

        assert!(shortcut.modifiers.ctrl);
        assert!(shortcut.modifiers.shift);
        assert_eq!(shortcut.key, "N");
        assert_eq!(shortcut.to_string(), "Ctrl+Shift+N");
    }

    #[test]
    fn normalizes_named_keys() {
        assert_eq!(
            Shortcut::parse("Ctrl+Return").unwrap(),
            Shortcut::parse("ctrl+enter").unwrap()
        );
        assert_eq!(Shortcut::parse("Shift+f1").unwrap().to_string(), "Shift+F1");
        assert_eq!(
            Shortcut::parse("Alt+page-down").unwrap().to_string(),
            "Alt+PageDown"
        );
    }

    #[test]
    fn rejects_shortcut_conflicts() {
        let mut registry = ActionRegistry::new();
        registry
            .register(ActionDescriptor::new("notes.new", "New").shortcut("Ctrl+N".parse().unwrap()))
            .unwrap();

        assert!(matches!(
            registry.register(
                ActionDescriptor::new("files.new", "New File").shortcut("Ctrl+N".parse().unwrap())
            ),
            Err(ActionRegistryError::ShortcutConflict { .. })
        ));
    }
}
