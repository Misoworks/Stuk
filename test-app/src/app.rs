use crate::{
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
