use stuk_core::Element;

use crate::{Button, Sidebar, Spacer, SplitView, Text, Toolbar, VStack};

#[derive(Clone, Debug)]
pub struct SidebarLayout {
    sidebar: Element,
    content: Element,
    ratio: f32,
    resizable: bool,
}

impl SidebarLayout {
    pub fn new(sidebar: impl Into<Element>, content: impl Into<Element>) -> Self {
        Self {
            sidebar: sidebar.into(),
            content: content.into(),
            ratio: 0.28,
            resizable: false,
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

impl From<SidebarLayout> for Element {
    fn from(layout: SidebarLayout) -> Self {
        SplitView::new(layout.sidebar, layout.content)
            .initial_ratio(layout.ratio)
            .resizable(layout.resizable)
            .into()
    }
}

#[derive(Clone, Debug)]
pub struct NavigationItem {
    label: String,
    action: String,
    selected: bool,
    disabled: bool,
}

impl NavigationItem {
    pub fn new(label: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            action: action.into(),
            selected: false,
            disabled: false,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl From<NavigationItem> for Element {
    fn from(item: NavigationItem) -> Self {
        let button = if item.selected {
            Button::primary(item.label)
        } else {
            Button::ghost(item.label)
        };
        button.action(item.action).disabled(item.disabled).into()
    }
}

#[derive(Clone, Debug)]
pub struct NavigationView {
    title: String,
    items: Vec<Element>,
    content: Element,
    footer: Vec<Element>,
    toolbar_actions: Vec<Element>,
    shows_toolbar: bool,
    ratio: f32,
    resizable: bool,
}

impl NavigationView {
    pub fn new(title: impl Into<String>, content: impl Into<Element>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
            content: content.into(),
            footer: Vec::new(),
            toolbar_actions: Vec::new(),
            shows_toolbar: false,
            ratio: 0.28,
            resizable: false,
        }
    }

    pub fn item(mut self, item: impl Into<Element>) -> Self {
        self.items.push(item.into());
        self
    }

    pub fn footer(mut self, footer: impl Into<Element>) -> Self {
        self.footer.push(footer.into());
        self
    }

    pub fn toolbar_action(mut self, action: impl Into<Element>) -> Self {
        self.toolbar_actions.push(action.into());
        self.shows_toolbar = true;
        self
    }

    pub fn toolbar(mut self, visible: bool) -> Self {
        self.shows_toolbar = visible;
        self
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

impl From<NavigationView> for Element {
    fn from(view: NavigationView) -> Self {
        let mut sidebar = Sidebar::new().child(Text::title(view.title.clone()));
        for item in view.items {
            sidebar = sidebar.child(item);
        }
        if !view.footer.is_empty() {
            sidebar = sidebar.child(Spacer::new());
            for footer in view.footer {
                sidebar = sidebar.child(footer);
            }
        }

        let content = if view.shows_toolbar {
            let mut toolbar = Toolbar::new(view.title);
            for action in view.toolbar_actions {
                toolbar = toolbar.child(action);
            }
            VStack::new()
                .padding(24.0)
                .spacing(14.0)
                .child(toolbar)
                .child(view.content)
                .into()
        } else {
            view.content
        };

        SidebarLayout::new(sidebar, content)
            .initial_ratio(view.ratio)
            .resizable(view.resizable)
            .into()
    }
}
