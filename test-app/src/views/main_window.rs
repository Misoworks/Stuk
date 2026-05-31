use crate::state::AppState;
use stuk::prelude::*;

pub struct MainWindowView<'a> {
    state: &'a AppState,
}

impl<'a> MainWindowView<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    fn document_title(&self) -> &str {
        if self.state.document_title.is_empty() {
            "Untitled Document"
        } else {
            &self.state.document_title
        }
    }

    fn status_text(&self) -> String {
        match self.state.last_action {
            Some(action) => format!("Last action: {action:?}"),
            None => format!("{} items ready in {}", self.state.item_count, self.document_title()),
        }
    }
}

impl View for MainWindowView<'_> {
    fn view(&self, cx: &mut Cx) -> stuk::Element {
        let _ = cx;
        Window::new()
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
            .into()
    }
}
