mod async_view;
mod chrome_widgets;
mod command_palette;
mod composite_widgets;
mod controls;
mod data_widgets;
mod feedback;
mod layout_widgets;
mod media_widgets;
mod mvp;
mod navigation;
mod settings_page;
mod surface_widget;
mod text_widgets;
mod virtual_list;

pub use async_view::{MutationView, ResourceView};
pub use chrome_widgets::{ResizablePane, Titlebar};
pub use command_palette::CommandPalette;
pub use composite_widgets::{
    ContextMenu, Dropdown, DropdownOption, Form, FormRow, Menu, MenuItem, Toast, ToastKind,
};
pub use controls::{
    Avatar, Badge, Card, Checkbox, ProgressBar, Radio, SegmentedControl, Slider, Tabs, Tooltip,
};
pub use data_widgets::{ColorWell, Table, TableColumn, TableRow, Tree, TreeNode};
pub use feedback::{Dialog, EmptyState, ErrorView, List, Popover, Spinner};
pub use layout_widgets::{Flex, Grid, Overlay};
pub use media_widgets::{Image, Svg};
pub use mvp::{IconButton, ScrollView, Sidebar, SplitView, TextField, Toggle, Toolbar};
pub use navigation::{NavigationItem, NavigationView, SidebarLayout};
pub use settings_page::SettingsPage;
pub use surface_widget::Surface;
pub use text_widgets::{
    Label, PasswordField, SearchField, SelectableText, TextArea, TextEditorLite,
};
pub use virtual_list::VirtualList;

use stuk_core::{
    ButtonElement, DividerElement, Element, FrameElement, SpacerElement, StackElement, TextElement,
    WindowElement,
};
use stuk_layout::{Axis, EdgeInsets, Length};
use stuk_platform::WindowChrome;
use stuk_style::{ButtonVariant, Color, Material};

#[derive(Clone, Debug)]
pub struct Window {
    element: WindowElement,
}

impl Window {
    pub fn new() -> Self {
        Self {
            element: WindowElement::default(),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.element.title = title.into();
        self
    }

    pub fn material(mut self, material: Material) -> Self {
        self.element.material = material;
        self
    }

    pub fn chrome(mut self, chrome: WindowChrome) -> Self {
        self.element.chrome = chrome;
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.element.width = width;
        self.element.height = height;
        self
    }

    pub fn content(mut self, content: impl Into<Element>) -> Self {
        self.element.content = Some(Box::new(content.into()));
        self
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Window> for Element {
    fn from(window: Window) -> Self {
        Element::Window(window.element)
    }
}

#[derive(Clone, Debug)]
pub struct Text {
    element: TextElement,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            element: TextElement {
                text: text.into(),
                size: 14.0,
                line_height: 20.0,
                color: Color::TEXT,
            },
        }
    }

    pub fn title(text: impl Into<String>) -> Self {
        Self {
            element: TextElement {
                text: text.into(),
                size: 26.0,
                line_height: 34.0,
                color: Color::TEXT,
            },
        }
    }

    pub fn muted(mut self) -> Self {
        self.element.color = Color::TEXT_MUTED;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.element.size = size;
        self.element.line_height = (size * 1.32).ceil();
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.element.color = color;
        self
    }
}

impl From<Text> for Element {
    fn from(text: Text) -> Self {
        Element::Text(text.element)
    }
}

#[derive(Clone, Debug)]
pub struct Button {
    element: ButtonElement,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            element: ButtonElement {
                label: label.into(),
                variant: ButtonVariant::Secondary,
                action: None,
                disabled: false,
            },
        }
    }

    pub fn primary(label: impl Into<String>) -> Self {
        Self::new(label).variant(ButtonVariant::Primary)
    }

    pub fn secondary(label: impl Into<String>) -> Self {
        Self::new(label).variant(ButtonVariant::Secondary)
    }

    pub fn destructive(label: impl Into<String>) -> Self {
        Self::new(label).variant(ButtonVariant::Destructive)
    }

    pub fn ghost(label: impl Into<String>) -> Self {
        Self::new(label).variant(ButtonVariant::Ghost)
    }

    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.element.action = Some(action.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.element.disabled = disabled;
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.element.variant = variant;
        self
    }
}

impl From<Button> for Element {
    fn from(button: Button) -> Self {
        Element::Button(button.element)
    }
}

#[derive(Clone, Debug)]
pub struct Frame {
    element: FrameElement,
}

impl Frame {
    pub fn new(child: impl Into<Element>) -> Self {
        Self {
            element: FrameElement::new(child),
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.element.width = Length::Fixed(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.element.height = Length::Fixed(height);
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.element.width = Length::Fixed(width);
        self.element.height = Length::Fixed(height);
        self
    }

    pub fn margin(mut self, margin: f32) -> Self {
        self.element.margin = EdgeInsets::all(margin);
        self
    }

    pub fn fill_width(mut self) -> Self {
        self.element.width = Length::Fill;
        self
    }

    pub fn fill_height(mut self) -> Self {
        self.element.height = Length::Fill;
        self
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.element.min_width = Some(width.max(0.0));
        self
    }

    pub fn max_width(mut self, width: f32) -> Self {
        self.element.max_width = Some(width.max(0.0));
        self
    }

    pub fn min_height(mut self, height: f32) -> Self {
        self.element.min_height = Some(height.max(0.0));
        self
    }

    pub fn max_height(mut self, height: f32) -> Self {
        self.element.max_height = Some(height.max(0.0));
        self
    }
}

impl From<Frame> for Element {
    fn from(frame: Frame) -> Self {
        Element::Frame(frame.element)
    }
}

#[derive(Clone, Debug)]
pub struct VStack {
    element: StackElement,
}

impl VStack {
    pub fn new() -> Self {
        Self {
            element: StackElement::new(Axis::Vertical),
        }
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.element.padding = EdgeInsets::all(padding);
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.element.spacing = spacing;
        self
    }

    pub fn child(mut self, child: impl Into<Element>) -> Self {
        self.element.children.push(child.into());
        self
    }
}

impl Default for VStack {
    fn default() -> Self {
        Self::new()
    }
}

impl From<VStack> for Element {
    fn from(stack: VStack) -> Self {
        Element::Stack(stack.element)
    }
}

#[derive(Clone, Debug)]
pub struct HStack {
    element: StackElement,
}

impl HStack {
    pub fn new() -> Self {
        Self {
            element: StackElement::new(Axis::Horizontal),
        }
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.element.padding = EdgeInsets::all(padding);
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.element.spacing = spacing;
        self
    }

    pub fn child(mut self, child: impl Into<Element>) -> Self {
        self.element.children.push(child.into());
        self
    }
}

impl Default for HStack {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HStack> for Element {
    fn from(stack: HStack) -> Self {
        Element::Stack(stack.element)
    }
}

#[derive(Clone, Debug)]
pub struct ZStack {
    element: StackElement,
}

impl ZStack {
    pub fn new() -> Self {
        Self {
            element: StackElement::new(Axis::Depth),
        }
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.element.padding = EdgeInsets::all(padding);
        self
    }

    pub fn child(mut self, child: impl Into<Element>) -> Self {
        self.element.children.push(child.into());
        self
    }
}

impl Default for ZStack {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ZStack> for Element {
    fn from(stack: ZStack) -> Self {
        Element::Stack(stack.element)
    }
}

#[derive(Clone, Debug)]
pub struct Spacer {
    element: SpacerElement,
}

impl Spacer {
    pub fn new() -> Self {
        Self {
            element: SpacerElement::default(),
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.element.width = Length::Fixed(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.element.height = Length::Fixed(height);
        self
    }
}

impl Default for Spacer {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Spacer> for Element {
    fn from(spacer: Spacer) -> Self {
        Element::Spacer(spacer.element)
    }
}

#[derive(Clone, Debug)]
pub struct Divider {
    element: DividerElement,
}

impl Divider {
    pub fn horizontal() -> Self {
        Self {
            element: DividerElement::horizontal(),
        }
    }

    pub fn vertical() -> Self {
        Self {
            element: DividerElement::vertical(),
        }
    }

    pub fn thickness(mut self, thickness: f32) -> Self {
        self.element.thickness = thickness;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.element.color = color;
        self
    }
}

impl From<Divider> for Element {
    fn from(divider: Divider) -> Self {
        Element::Divider(divider.element)
    }
}
