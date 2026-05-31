use stuk::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum Action {
    NewDocument,
    OpenSettings,
    SyncSettings,
    SearchNotes,
    SaveDocument,
}

impl Action {
    pub fn id(self) -> &'static str {
        match self {
            Self::NewDocument => "app.new_document",
            Self::OpenSettings => "app.settings",
            Self::SyncSettings => "settings.sync.enabled",
            Self::SearchNotes => "notes.search",
            Self::SaveDocument => "document.save",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "app.new_document" | "notes.new" => Some(Self::NewDocument),
            "app.settings" => Some(Self::OpenSettings),
            "settings.sync.enabled" => Some(Self::SyncSettings),
            "notes.search" => Some(Self::SearchNotes),
            "document.save" => Some(Self::SaveDocument),
            _ => None,
        }
    }
}

pub fn action_descriptors() -> Vec<ActionDescriptor> {
    vec![
        ActionDescriptor::new(Action::NewDocument.id(), "New Document").shortcut(Shortcut::new(
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
            "N",
        )),
        ActionDescriptor::new(Action::OpenSettings.id(), "Settings").shortcut(Shortcut::new(
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
            ",",
        )),
        ActionDescriptor::new(Action::SyncSettings.id(), "Toggle Sync"),
        ActionDescriptor::new("notes.new", "New Note").shortcut(Shortcut::new(
            Modifiers {
                ctrl: true,
                alt: true,
                ..Modifiers::default()
            },
            "N",
        )),
        ActionDescriptor::new(Action::SearchNotes.id(), "Search").shortcut(Shortcut::new(
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
            "F",
        )),
        ActionDescriptor::new(Action::SaveDocument.id(), "Save").shortcut(Shortcut::new(
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
            "S",
        )),
    ]
}
