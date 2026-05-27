use stuk_actions::ActionDescriptor;
use stuk_core::Element;

use crate::{Button, Dialog, EmptyState, List, SearchField, VStack};

#[derive(Clone, Debug)]
pub struct CommandPalette {
    title: String,
    query: String,
    actions: Vec<ActionDescriptor>,
    empty_title: String,
}

impl CommandPalette {
    pub fn new() -> Self {
        Self {
            title: "Command Palette".to_string(),
            query: String::new(),
            actions: Vec::new(),
            empty_title: "No commands".to_string(),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }

    pub fn empty_title(mut self, title: impl Into<String>) -> Self {
        self.empty_title = title.into();
        self
    }

    pub fn action(mut self, action: ActionDescriptor) -> Self {
        self.actions.push(action);
        self
    }

    pub fn actions(mut self, actions: impl IntoIterator<Item = ActionDescriptor>) -> Self {
        self.actions.extend(actions);
        self
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl From<CommandPalette> for Element {
    fn from(palette: CommandPalette) -> Self {
        let mut list = List::new().spacing(6.0);
        let mut visible_actions = 0;

        for action in palette
            .actions
            .iter()
            .filter(|action| action.visible && matches_query(action, &palette.query))
        {
            visible_actions += 1;
            list = list.child(
                Button::ghost(action_label(action))
                    .action(action.id.clone())
                    .disabled(!action.enabled),
            );
        }

        let results: Element = if visible_actions == 0 {
            EmptyState::new(palette.empty_title).into()
        } else {
            list.into()
        };

        Dialog::new(
            palette.title,
            VStack::new()
                .spacing(10.0)
                .child(SearchField::new(palette.query).placeholder("Search commands"))
                .child(results),
        )
        .into()
    }
}

fn matches_query(action: &ActionDescriptor, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty()
        || action.label.to_lowercase().contains(&query)
        || action.id.to_lowercase().contains(&query)
        || action
            .description
            .as_deref()
            .is_some_and(|description| description.to_lowercase().contains(&query))
}

fn action_label(action: &ActionDescriptor) -> String {
    match &action.shortcut {
        Some(shortcut) => format!("{}  {}", action.label, shortcut),
        None => action.label.clone(),
    }
}
