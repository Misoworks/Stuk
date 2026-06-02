use stuk_core::Element;
use stuk_layout::FlexAlign;

use crate::{Button, Divider, Flex, Frame, HStack, Spacer, Text, VStack};

#[derive(Clone, Debug)]
pub struct AppShell {
    sidebar: Option<Element>,
    content: Element,
    titlebar: Option<Element>,
    sidebar_width: f32,
}

impl AppShell {
    pub fn new(content: impl Into<Element>) -> Self {
        Self {
            sidebar: None,
            content: content.into(),
            titlebar: None,
            sidebar_width: 260.0,
        }
    }

    pub fn sidebar(mut self, sidebar: impl Into<Element>) -> Self {
        self.sidebar = Some(sidebar.into());
        self
    }

    pub fn titlebar(mut self, titlebar: impl Into<Element>) -> Self {
        self.titlebar = Some(titlebar.into());
        self
    }

    pub fn sidebar_width(mut self, width: f32) -> Self {
        self.sidebar_width = width.max(0.0);
        self
    }
}

impl From<AppShell> for Element {
    fn from(shell: AppShell) -> Self {
        let body: Element = if let Some(sidebar) = shell.sidebar {
            Flex::row()
                .fill_width()
                .fill_height()
                .flex_child(
                    stuk_core::FlexChildElement::new(
                        Frame::new(sidebar).width(shell.sidebar_width).fill_height(),
                    )
                    .basis(shell.sidebar_width)
                    .shrink(0.0),
                )
                .child(Divider::vertical())
                .child(Frame::new(shell.content).fill_width().fill_height())
                .into()
        } else {
            Frame::new(shell.content).fill_width().fill_height().into()
        };

        let mut stack = VStack::new().child(Frame::new(body).fill_width().fill_height());
        if let Some(titlebar) = shell.titlebar {
            stack = VStack::new()
                .child(Frame::new(titlebar).fill_width())
                .child(Frame::new(stack).fill_width().fill_height());
        }
        Frame::new(stack).fill_width().fill_height().into()
    }
}

#[derive(Clone, Debug)]
pub struct PageShell {
    title: Option<String>,
    toolbar: Option<Element>,
    content: Element,
    padding: f32,
}

impl PageShell {
    pub fn new(content: impl Into<Element>) -> Self {
        Self {
            title: None,
            toolbar: None,
            content: content.into(),
            padding: 24.0,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn toolbar(mut self, toolbar: impl Into<Element>) -> Self {
        self.toolbar = Some(toolbar.into());
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding.max(0.0);
        self
    }
}

impl From<PageShell> for Element {
    fn from(page: PageShell) -> Self {
        let mut content = VStack::new().spacing(16.0).padding(page.padding);
        if page.title.is_some() || page.toolbar.is_some() {
            let mut header = HStack::new().spacing(12.0);
            if let Some(title) = page.title {
                header = header.child(Frame::new(Text::title(title)).fill_width());
            } else {
                header = header.child(Frame::new(Spacer::new()).fill_width());
            }
            if let Some(toolbar) = page.toolbar {
                header = header.child(toolbar);
            }
            content = content.child(header);
        }
        content
            .child(Frame::new(page.content).fill_width().fill_height())
            .into()
    }
}

#[derive(Clone, Debug)]
pub struct Pane {
    child: Element,
    padding: f32,
}

impl Pane {
    pub fn new(child: impl Into<Element>) -> Self {
        Self {
            child: child.into(),
            padding: 16.0,
        }
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding.max(0.0);
        self
    }
}

impl From<Pane> for Element {
    fn from(pane: Pane) -> Self {
        Frame::new(VStack::new().padding(pane.padding).child(pane.child))
            .fill_width()
            .fill_height()
            .into()
    }
}

#[derive(Clone, Debug)]
pub struct CommandBar {
    leading: Vec<Element>,
    trailing: Vec<Element>,
}

impl CommandBar {
    pub fn new() -> Self {
        Self {
            leading: Vec::new(),
            trailing: Vec::new(),
        }
    }

    pub fn leading(mut self, child: impl Into<Element>) -> Self {
        self.leading.push(child.into());
        self
    }

    pub fn trailing(mut self, child: impl Into<Element>) -> Self {
        self.trailing.push(child.into());
        self
    }

    pub fn action(self, label: impl Into<String>, action: impl Into<String>) -> Self {
        self.trailing(Button::new(label).action(action))
    }
}

impl Default for CommandBar {
    fn default() -> Self {
        Self::new()
    }
}

impl From<CommandBar> for Element {
    fn from(bar: CommandBar) -> Self {
        let mut row = Flex::row().fill_width().gap(10.0).align(FlexAlign::Center);
        for child in bar.leading {
            row = row.child(child);
        }
        row = row.child(Frame::new(Spacer::new()).fill_width());
        for child in bar.trailing {
            row = row.child(child);
        }
        row.into()
    }
}

#[derive(Clone, Debug)]
pub struct ListSection {
    title: Option<String>,
    rows: Vec<Element>,
    spacing: f32,
}

impl ListSection {
    pub fn new() -> Self {
        Self {
            title: None,
            rows: Vec::new(),
            spacing: 8.0,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn row(mut self, row: impl Into<Element>) -> Self {
        self.rows.push(row.into());
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing.max(0.0);
        self
    }
}

impl Default for ListSection {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ListSection> for Element {
    fn from(section: ListSection) -> Self {
        let mut stack = VStack::new().spacing(section.spacing);
        if let Some(title) = section.title {
            stack = stack.child(Text::new(title).muted());
        }
        for row in section.rows {
            stack = stack.child(row);
        }
        stack.into()
    }
}
