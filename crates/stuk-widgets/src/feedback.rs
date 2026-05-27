use stuk_core::Element;

use crate::{Button, Divider, Frame, HStack, Text, VStack};

#[derive(Clone, Debug)]
pub struct List {
    children: Vec<Element>,
    spacing: f32,
}

impl List {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            spacing: 8.0,
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn child(mut self, child: impl Into<Element>) -> Self {
        self.children.push(child.into());
        self
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

impl From<List> for Element {
    fn from(list: List) -> Self {
        let mut stack = VStack::new().spacing(list.spacing);
        for child in list.children {
            stack = stack.child(child);
        }
        stack.into()
    }
}

#[derive(Clone, Debug)]
pub struct Popover {
    title: Option<String>,
    content: Element,
}

impl Popover {
    pub fn new(content: impl Into<Element>) -> Self {
        Self {
            title: None,
            content: content.into(),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

impl From<Popover> for Element {
    fn from(popover: Popover) -> Self {
        let mut content = VStack::new().padding(16.0).spacing(10.0);
        if let Some(title) = popover.title {
            content = content.child(Text::new(title).size(16.0));
        }
        Frame::new(content.child(popover.content))
            .fill_width()
            .into()
    }
}

#[derive(Clone, Debug)]
pub struct Dialog {
    title: String,
    content: Element,
    actions: Vec<Element>,
}

impl Dialog {
    pub fn new(title: impl Into<String>, content: impl Into<Element>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            actions: Vec::new(),
        }
    }

    pub fn action(mut self, action: impl Into<Element>) -> Self {
        self.actions.push(action.into());
        self
    }
}

impl From<Dialog> for Element {
    fn from(dialog: Dialog) -> Self {
        let mut action_row = HStack::new().spacing(8.0);
        for action in dialog.actions {
            action_row = action_row.child(action);
        }

        Frame::new(
            VStack::new()
                .padding(20.0)
                .spacing(12.0)
                .child(Text::title(dialog.title))
                .child(dialog.content)
                .child(Divider::horizontal())
                .child(action_row),
        )
        .fill_width()
        .into()
    }
}

#[derive(Clone, Debug)]
pub struct Spinner {
    label: String,
}

impl Spinner {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

impl From<Spinner> for Element {
    fn from(spinner: Spinner) -> Self {
        HStack::new()
            .spacing(8.0)
            .child(Text::new("...").muted())
            .child(Text::new(spinner.label).muted())
            .into()
    }
}

#[derive(Clone, Debug)]
pub struct EmptyState {
    title: String,
    message: Option<String>,
    action: Option<Element>,
}

impl EmptyState {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: None,
            action: None,
        }
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn action(mut self, action: impl Into<Element>) -> Self {
        self.action = Some(action.into());
        self
    }
}

impl From<EmptyState> for Element {
    fn from(empty: EmptyState) -> Self {
        let mut content = VStack::new()
            .padding(18.0)
            .spacing(10.0)
            .child(Text::new(empty.title).size(18.0));
        if let Some(message) = empty.message {
            content = content.child(Text::new(message).muted());
        }
        if let Some(action) = empty.action {
            content = content.child(action);
        }
        content.into()
    }
}

#[derive(Clone, Debug)]
pub struct ErrorView {
    message: String,
    retry_action: Option<String>,
}

impl ErrorView {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retry_action: None,
        }
    }

    pub fn retry_action(mut self, action: impl Into<String>) -> Self {
        self.retry_action = Some(action.into());
        self
    }
}

impl From<ErrorView> for Element {
    fn from(error: ErrorView) -> Self {
        let mut content = VStack::new()
            .padding(18.0)
            .spacing(10.0)
            .child(Text::new(error.message).color(stuk_style::Color::DANGER));
        if let Some(action) = error.retry_action {
            content = content.child(Button::secondary("Retry").action(action));
        }
        content.into()
    }
}
