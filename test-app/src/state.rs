use crate::actions::Action;

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
