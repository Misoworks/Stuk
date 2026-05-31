#[derive(Clone, Copy)]
pub enum Template {
    Basic,
    Sidebar,
    Document,
    Settings,
    ComponentLibrary,
}

impl Template {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "basic" => Ok(Self::Basic),
            "sidebar" => Ok(Self::Sidebar),
            "document" => Ok(Self::Document),
            "settings" => Ok(Self::Settings),
            "component-library" => Ok(Self::ComponentLibrary),
            other => Err(format!(
                "unknown template `{other}`; use basic, sidebar, document, settings, or component-library"
            )),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Sidebar => "sidebar",
            Self::Document => "document",
            Self::Settings => "settings",
            Self::ComponentLibrary => "component-library",
        }
    }
}

pub struct ProjectContext {
    pub app_name: String,
    pub package_name: String,
    pub app_id: String,
    pub stuk_dependency: String,
}

pub fn cargo_toml(context: &ProjectContext) -> String {
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"
rust-version = "1.89"

[dependencies]
{}
"#,
        context.package_name, context.stuk_dependency
    )
}

pub fn stuk_toml(context: &ProjectContext) -> String {
    format!(
        r#"[app]
id = "{}"
name = "{}"
version = "0.1.0"

[window.main]
title = "{}"
width = 980
height = 680
min_width = 420
min_height = 280
material = "maris"
chrome = "system"
transparent = false
background_effect = "none"

[permissions]
network = false
notifications = false

[actions.app.new_document]
label = "New Document"
shortcut = "Ctrl+N"

[actions.app.settings]
label = "Settings"
shortcut = "Ctrl+,"

[actions.settings.sync.enabled]
label = "Toggle Sync"

[actions.notes.new]
label = "New Note"
shortcut = "Ctrl+Alt+N"

[actions.notes.search]
label = "Search"
shortcut = "Ctrl+F"

[actions.document.save]
label = "Save"
shortcut = "Ctrl+S"

[settings.appearance.theme]
type = "enum"
label = "Theme"
values = ["system", "light", "dark"]
default = "system"

[settings.appearance.density]
type = "enum"
label = "Density"
values = ["compact", "regular", "touch"]
default = "regular"

[settings.sync.enabled]
type = "boolean"
label = "Enable sync"
default = false
"#,
        context.app_id, context.app_name, context.app_name
    )
}

pub fn agents_md() -> String {
    r#"# Agent Instructions

- Views live in `src/views/`.
- Reusable UI components live in `src/components/`.
- App state lives in `src/state.rs`.
- User actions live in `src/actions.rs`.
- Runtime settings schema lives in `src/settings.rs`.
- App metadata, permissions, windows, actions, and settings schema live in `Stuk.toml`.
- Prefer existing Stuk widgets before custom drawing.
- Use semantic materials (`Maris`, `Luca`, `Surface`) instead of hardcoded blur.
- Run `stuk validate` after manifest changes.
- Run `cargo test` if logic changed.
"#
    .to_string()
}

pub fn main_rs(context: &ProjectContext) -> String {
    format!(
        r#"mod actions;
mod app;
mod components;
mod settings;
mod state;
mod views;

use app::MainWindow;
use stuk::prelude::*;

fn main() -> stuk::Result {{
    App::new()
        .id("{}")
        .name("{}")
        .window(MainWindow::default())
        .run()
}}
"#,
        context.app_id, context.app_name
    )
}

pub fn app_rs() -> String {
    r#"use crate::{
    actions::{Action, action_descriptors},
    settings::app_settings_schema,
    state::AppState,
    views::main_window::MainWindowView,
};
use stuk::prelude::*;

#[derive(Default)]
pub struct MainWindow {
    state: AppState,
}

impl View for MainWindow {
    fn view(&self, cx: &mut Cx) -> stuk::Element {
        MainWindowView::new(&self.state).view(cx)
    }

    fn actions(&self, _cx: &mut Cx) -> Vec<ActionDescriptor> {
        action_descriptors()
    }

    fn settings(&self, _cx: &mut Cx) -> SettingsSchema {
        app_settings_schema()
    }

    fn handle_action(&mut self, action_id: &str, _cx: &mut Cx) {
        self.state.last_action = Action::from_id(action_id);
    }
}
"#
    .to_string()
}

pub fn settings_rs() -> String {
    r#"use stuk::prelude::*;

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
"#
    .to_string()
}

pub fn state_rs() -> String {
    r#"use crate::actions::Action;

pub struct AppState {
    pub document_title: String,
    pub item_count: u32,
    pub last_action: Option<Action>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            document_title: "Untitled Document".to_string(),
            item_count: 3,
            last_action: None,
        }
    }
}
"#
    .to_string()
}

pub fn actions_rs() -> String {
    r#"use stuk::prelude::*;

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
"#
    .to_string()
}

pub fn main_window_rs(template: Template) -> String {
    let body = match template {
        Template::Basic => basic_template(),
        Template::Sidebar => sidebar_template(),
        Template::Document => document_template(),
        Template::Settings => settings_template(),
        Template::ComponentLibrary => component_library_template(),
    };

    format!(
        r#"use crate::state::AppState;
use stuk::prelude::*;

pub struct MainWindowView<'a> {{
    state: &'a AppState,
}}

impl<'a> MainWindowView<'a> {{
    pub fn new(state: &'a AppState) -> Self {{
        Self {{ state }}
    }}

    fn document_title(&self) -> &str {{
        if self.state.document_title.is_empty() {{
            "Untitled Document"
        }} else {{
            &self.state.document_title
        }}
    }}

    fn status_text(&self) -> String {{
        match self.state.last_action {{
            Some(action) => format!("Last action: {{action:?}}"),
            None => format!("{{}} items ready in {{}}", self.state.item_count, self.document_title()),
        }}
    }}
}}

impl View for MainWindowView<'_> {{
    fn view(&self, cx: &mut Cx) -> stuk::Element {{
        let _ = cx;
{body}
    }}
}}
"#
    )
}

fn basic_template() -> &'static str {
    r#"        Window::new()
            .title("Stuk App")
            .material(Material::Maris)
            .chrome(WindowChrome::System)
            .content(
                VStack::new()
                    .padding(32.0)
                    .spacing(14.0)
                    .child(Text::title("Stuk App"))
                    .child(Text::new(self.status_text()).muted())
                    .child(Button::primary("Create document").action("app.new_document")),
            )
            .into()"#
}

fn sidebar_template() -> &'static str {
    r#"        Window::new()
            .title("Stuk Sidebar App")
            .material(Material::Maris)
            .chrome(WindowChrome::System)
            .content(
                SplitView::new(
                    Sidebar::new()
                        .child(Text::new("All Notes"))
                        .child(Text::new("Pinned").muted())
                        .child(Toggle::new("Sync", cx.setting_bool("sync.enabled").unwrap_or_default()).action("settings.sync.enabled")),
                    VStack::new()
                        .padding(24.0)
                        .spacing(12.0)
                        .child(
                            Toolbar::new("Notes")
                                .child(Button::primary("New note").action("notes.new"))
                                .child(IconButton::new("S", "Search").action("notes.search")),
                        )
                        .child(TextField::new("").label("Search").placeholder("Find notes"))
                        .child(Text::new(self.status_text()).muted())
                        .child(ScrollView::new(Text::new("A sidebar layout using Stuk MVP widgets.")).height(96.0)),
                )
                .resizable(true),
            )
            .into()"#
}

fn document_template() -> &'static str {
    r#"        Window::new()
            .title("Stuk Document")
            .material(Material::Maris)
            .chrome(WindowChrome::System)
            .content(
                VStack::new()
                    .padding(34.0)
                    .spacing(12.0)
                    .child(Text::title(self.document_title()))
                    .child(Text::new(self.status_text()).muted())
                    .child(
                        HStack::new()
                            .spacing(10.0)
                            .child(Button::primary("Save").action("document.save"))
                            .child(Button::new("Settings").action("app.settings")),
                    ),
            )
            .into()"#
}

fn settings_template() -> &'static str {
    r#"        Window::new()
            .title("Stuk Settings")
            .material(Material::Maris)
            .chrome(WindowChrome::System)
            .content(
                VStack::new()
                    .spacing(8.0)
                    .child(
                        SettingsPage::from_schema(cx.settings_schema().clone())
                            .values(cx.settings_store())
                            .action_prefix("settings"),
                    )
                    .child(Text::new(self.status_text()).muted()),
            )
            .into()"#
}

fn component_library_template() -> &'static str {
    r#"        Window::new()
            .title("Stuk Components")
            .material(Material::Maris)
            .chrome(WindowChrome::System)
            .content(
                VStack::new()
                    .padding(32.0)
                    .spacing(14.0)
                    .child(Text::title("Components"))
                    .child(Text::new(self.status_text()).muted())
                    .child(
                        HStack::new()
                            .spacing(10.0)
                            .child(Button::primary("Primary"))
                            .child(Button::secondary("Secondary"))
                            .child(Button::ghost("Ghost")),
                    ),
            )
            .into()"#
}
