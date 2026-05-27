use stuk_core::Element;

use crate::{SplitView, Text, Toolbar, VStack};

#[derive(Clone, Debug)]
pub struct Titlebar {
    title: String,
    subtitle: Option<String>,
    actions: Vec<Element>,
}

impl Titlebar {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            actions: Vec::new(),
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn action(mut self, action: impl Into<Element>) -> Self {
        self.actions.push(action.into());
        self
    }
}

impl From<Titlebar> for Element {
    fn from(titlebar: Titlebar) -> Self {
        let mut toolbar = Toolbar::new(titlebar.title);
        for action in titlebar.actions {
            toolbar = toolbar.child(action);
        }

        match titlebar.subtitle {
            Some(subtitle) => VStack::new()
                .spacing(4.0)
                .child(toolbar)
                .child(Text::new(subtitle).muted())
                .into(),
            None => toolbar.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResizablePane {
    leading: Element,
    content: Element,
    ratio: f32,
    resizable: bool,
}

impl ResizablePane {
    pub fn new(leading: impl Into<Element>, content: impl Into<Element>) -> Self {
        Self {
            leading: leading.into(),
            content: content.into(),
            ratio: 0.3,
            resizable: true,
        }
    }

    pub fn initial_ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
}

impl From<ResizablePane> for Element {
    fn from(pane: ResizablePane) -> Self {
        SplitView::new(pane.leading, pane.content)
            .initial_ratio(pane.ratio)
            .resizable(pane.resizable)
            .into()
    }
}
