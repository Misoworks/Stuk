use stuk_core::{
    Element, IconButtonElement, ScrollViewElement, SidebarElement, SplitViewElement,
    TextFieldElement, ToggleElement, ToolbarElement,
};
use stuk_layout::Length;

#[derive(Clone, Debug)]
pub struct IconButton {
    element: IconButtonElement,
}

impl IconButton {
    pub fn new(icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            element: IconButtonElement {
                icon: icon.into(),
                label: label.into(),
                action: None,
                disabled: false,
            },
        }
    }

    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.element.action = Some(action.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.element.disabled = disabled;
        self
    }
}

impl From<IconButton> for Element {
    fn from(button: IconButton) -> Self {
        Element::IconButton(button.element)
    }
}

#[derive(Clone, Debug)]
pub struct Toggle {
    element: ToggleElement,
}

impl Toggle {
    pub fn new(label: impl Into<String>, checked: bool) -> Self {
        Self {
            element: ToggleElement {
                label: label.into(),
                checked,
                action: None,
                disabled: false,
            },
        }
    }

    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.element.action = Some(action.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.element.disabled = disabled;
        self
    }
}

impl From<Toggle> for Element {
    fn from(toggle: Toggle) -> Self {
        Element::Toggle(toggle.element)
    }
}

#[derive(Clone, Debug)]
pub struct TextField {
    element: TextFieldElement,
}

impl TextField {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            element: TextFieldElement {
                label: None,
                text: value.into(),
                placeholder: String::new(),
                disabled: false,
                focused: false,
                multiline: false,
                caret: None,
                selection: None,
                background: true,
                padding_x: 16.0,
                padding_y: 12.0,
            },
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.element.label = Some(label.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.element.placeholder = placeholder.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.element.disabled = disabled;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.element.focused = focused;
        self
    }

    pub fn multiline(mut self, multiline: bool) -> Self {
        self.element.multiline = multiline;
        self
    }

    pub fn caret(mut self, caret: usize) -> Self {
        self.element.caret = Some(caret);
        self
    }

    pub fn selection(mut self, anchor: usize, focus: usize) -> Self {
        self.element.selection = Some((anchor, focus));
        self.element.caret = Some(focus);
        self
    }

    pub fn background(mut self, background: bool) -> Self {
        self.element.background = background;
        self
    }

    pub fn padding(mut self, x: f32, y: f32) -> Self {
        self.element.padding_x = x.max(0.0);
        self.element.padding_y = y.max(0.0);
        self
    }

    pub fn plain(mut self) -> Self {
        self.element.background = false;
        self.element.padding_x = 0.0;
        self.element.padding_y = 0.0;
        self
    }
}

impl From<TextField> for Element {
    fn from(field: TextField) -> Self {
        Element::TextField(field.element)
    }
}

#[derive(Clone, Debug)]
pub struct ScrollView {
    element: ScrollViewElement,
}

impl ScrollView {
    pub fn new(child: impl Into<Element>) -> Self {
        Self {
            element: ScrollViewElement::new(child),
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.element.height = Length::Fixed(height);
        self
    }

    pub fn fill_width(mut self) -> Self {
        self.element.width = Length::Fill;
        self
    }

    pub fn scroll_offset_y(mut self, offset: f32) -> Self {
        self.element.scroll_offset_y = offset.max(0.0);
        self
    }

    pub fn scroll_offset_x(mut self, offset: f32) -> Self {
        self.element.scroll_offset_x = offset.max(0.0);
        self
    }
}

impl From<ScrollView> for Element {
    fn from(scroll_view: ScrollView) -> Self {
        Element::ScrollView(scroll_view.element)
    }
}

#[derive(Clone, Debug)]
pub struct Sidebar {
    element: SidebarElement,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            element: SidebarElement::default(),
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.element.width = width;
        self
    }

    pub fn child(mut self, child: impl Into<Element>) -> Self {
        self.element.children.push(child.into());
        self
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Sidebar> for Element {
    fn from(sidebar: Sidebar) -> Self {
        Element::Sidebar(sidebar.element)
    }
}

#[derive(Clone, Debug)]
pub struct Toolbar {
    element: ToolbarElement,
}

impl Toolbar {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            element: ToolbarElement {
                title: title.into(),
                children: Vec::new(),
            },
        }
    }

    pub fn child(mut self, child: impl Into<Element>) -> Self {
        self.element.children.push(child.into());
        self
    }
}

impl From<Toolbar> for Element {
    fn from(toolbar: Toolbar) -> Self {
        Element::Toolbar(toolbar.element)
    }
}

#[derive(Clone, Debug)]
pub struct SplitView {
    element: SplitViewElement,
}

impl SplitView {
    pub fn new(sidebar: impl Into<Element>, main: impl Into<Element>) -> Self {
        Self {
            element: SplitViewElement {
                sidebar: Box::new(sidebar.into()),
                main: Box::new(main.into()),
                ratio: 0.28,
                resizable: false,
            },
        }
    }

    pub fn initial_ratio(mut self, ratio: f32) -> Self {
        self.element.ratio = ratio.clamp(0.18, 0.5);
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.element.resizable = resizable;
        self
    }
}

impl From<SplitView> for Element {
    fn from(split_view: SplitView) -> Self {
        Element::SplitView(split_view.element)
    }
}
