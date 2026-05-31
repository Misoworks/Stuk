use stuk_layout::{Axis, EdgeInsets, Length};
use stuk_platform::{WindowBackgroundEffect, WindowChrome};
use stuk_style::{ButtonVariant, Color, Material, NumberSpacing, TextWrap};

use crate::control_elements::{
    AvatarElement, BadgeElement, CardElement, CheckboxElement, ProgressBarElement, RadioElement,
    SegmentedControlElement, SliderElement, TabsElement, TooltipElement,
};
use crate::layout_elements::{FlexElement, GridElement, OverlayElement};
use crate::list_elements::VirtualListElement;
use crate::media_elements::{MediaElement, MediaSource};
use crate::surface_elements::SurfaceElement;

#[derive(Clone, Debug)]
pub enum Element {
    Empty,
    Window(WindowElement),
    Text(TextElement),
    Button(ButtonElement),
    Stack(StackElement),
    Flex(FlexElement),
    Grid(GridElement),
    Overlay(OverlayElement),
    Surface(SurfaceElement),
    Media(MediaElement),
    Frame(FrameElement),
    Spacer(SpacerElement),
    Divider(DividerElement),
    IconButton(IconButtonElement),
    Toggle(ToggleElement),
    Checkbox(CheckboxElement),
    Radio(RadioElement),
    Slider(SliderElement),
    ProgressBar(ProgressBarElement),
    Tabs(TabsElement),
    SegmentedControl(SegmentedControlElement),
    Badge(BadgeElement),
    Avatar(AvatarElement),
    Card(CardElement),
    Tooltip(TooltipElement),
    TextField(TextFieldElement),
    ScrollView(ScrollViewElement),
    VirtualList(VirtualListElement),
    Sidebar(SidebarElement),
    Toolbar(ToolbarElement),
    SplitView(SplitViewElement),
}

impl Element {
    pub fn kind(&self) -> ElementKind {
        match self {
            Self::Empty => ElementKind::Empty,
            Self::Window(_) => ElementKind::Window,
            Self::Text(_) => ElementKind::Text,
            Self::Button(_) => ElementKind::Button,
            Self::Stack(stack) => match stack.axis {
                Axis::Horizontal => ElementKind::HStack,
                Axis::Vertical => ElementKind::VStack,
                Axis::Depth => ElementKind::ZStack,
            },
            Self::Flex(_) => ElementKind::Flex,
            Self::Grid(_) => ElementKind::Grid,
            Self::Overlay(_) => ElementKind::Overlay,
            Self::Surface(_) => ElementKind::Surface,
            Self::Media(media) => match media.source {
                MediaSource::Image => ElementKind::Image,
                MediaSource::Svg => ElementKind::Svg,
            },
            Self::Frame(_) => ElementKind::Frame,
            Self::Spacer(_) => ElementKind::Spacer,
            Self::Divider(_) => ElementKind::Divider,
            Self::IconButton(_) => ElementKind::IconButton,
            Self::Toggle(_) => ElementKind::Toggle,
            Self::Checkbox(_) => ElementKind::Checkbox,
            Self::Radio(_) => ElementKind::Radio,
            Self::Slider(_) => ElementKind::Slider,
            Self::ProgressBar(_) => ElementKind::ProgressBar,
            Self::Tabs(_) => ElementKind::Tabs,
            Self::SegmentedControl(_) => ElementKind::SegmentedControl,
            Self::Badge(_) => ElementKind::Badge,
            Self::Avatar(_) => ElementKind::Avatar,
            Self::Card(_) => ElementKind::Card,
            Self::Tooltip(_) => ElementKind::Tooltip,
            Self::TextField(_) => ElementKind::TextField,
            Self::ScrollView(_) => ElementKind::ScrollView,
            Self::VirtualList(_) => ElementKind::VirtualList,
            Self::Sidebar(_) => ElementKind::Sidebar,
            Self::Toolbar(_) => ElementKind::Toolbar,
            Self::SplitView(_) => ElementKind::SplitView,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementKind {
    Empty,
    Window,
    Text,
    Button,
    VStack,
    HStack,
    ZStack,
    Flex,
    Grid,
    Overlay,
    Surface,
    Image,
    Svg,
    Frame,
    Spacer,
    Divider,
    IconButton,
    Toggle,
    Checkbox,
    Radio,
    Slider,
    ProgressBar,
    Tabs,
    SegmentedControl,
    Badge,
    Avatar,
    Card,
    Tooltip,
    TextField,
    ScrollView,
    VirtualList,
    Sidebar,
    Toolbar,
    SplitView,
}

#[derive(Clone, Debug)]
pub struct WindowElement {
    pub title: String,
    pub material: Material,
    pub chrome: WindowChrome,
    pub transparent: bool,
    pub background_effect: WindowBackgroundEffect,
    pub content: Option<Box<Element>>,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub visible: bool,
    pub active: bool,
    pub always_on_top: bool,
    pub continuous_redraw: bool,
}

impl Default for WindowElement {
    fn default() -> Self {
        Self {
            title: "Stuk".to_string(),
            material: Material::Maris,
            chrome: WindowChrome::System,
            transparent: false,
            background_effect: WindowBackgroundEffect::None,
            content: None,
            width: 760,
            height: 520,
            resizable: true,
            visible: true,
            active: true,
            always_on_top: false,
            continuous_redraw: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextElement {
    pub text: String,
    pub size: f32,
    pub line_height: f32,
    pub color: Color,
    pub wrap: TextWrap,
    pub number_spacing: NumberSpacing,
    pub align: stuk_style::TextAlign,
}

#[derive(Clone, Debug)]
pub struct ButtonElement {
    pub label: String,
    pub variant: ButtonVariant,
    pub action: Option<String>,
    pub disabled: bool,
    pub text_align: stuk_style::ControlTextAlign,
}

#[derive(Clone, Debug)]
pub struct IconButtonElement {
    pub icon: String,
    pub label: String,
    pub action: Option<String>,
    pub disabled: bool,
}

#[derive(Clone, Debug)]
pub struct ToggleElement {
    pub label: String,
    pub checked: bool,
    pub action: Option<String>,
    pub disabled: bool,
}

#[derive(Clone, Debug)]
pub struct TextFieldElement {
    pub label: Option<String>,
    pub text: String,
    pub placeholder: String,
    pub disabled: bool,
    pub focused: bool,
    pub multiline: bool,
    pub caret: Option<usize>,
    pub selection: Option<(usize, usize)>,
    pub background: bool,
    pub padding_x: f32,
    pub padding_y: f32,
}

#[derive(Clone, Debug)]
pub struct StackElement {
    pub axis: Axis,
    pub padding: EdgeInsets,
    pub spacing: f32,
    pub children: Vec<Element>,
}

impl StackElement {
    pub fn new(axis: Axis) -> Self {
        Self {
            axis,
            padding: EdgeInsets::default(),
            spacing: 0.0,
            children: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FrameElement {
    pub child: Box<Element>,
    pub width: Length,
    pub height: Length,
    pub margin: EdgeInsets,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
}

impl FrameElement {
    pub fn new(child: impl Into<Element>) -> Self {
        Self {
            child: Box::new(child.into()),
            width: Length::Fit,
            height: Length::Fit,
            margin: EdgeInsets::default(),
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
        }
    }

    pub fn child_bounds(&self, bounds: stuk_layout::Rect) -> stuk_layout::Rect {
        bounds.inset(self.margin)
    }
}

#[derive(Clone, Debug)]
pub struct SpacerElement {
    pub width: Length,
    pub height: Length,
}

impl Default for SpacerElement {
    fn default() -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DividerElement {
    pub axis: Axis,
    pub thickness: f32,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub struct ScrollViewElement {
    pub child: Box<Element>,
    pub width: Length,
    pub height: Length,
    pub scroll_offset_x: f32,
    pub scroll_offset_y: f32,
}

impl ScrollViewElement {
    pub fn new(child: impl Into<Element>) -> Self {
        Self {
            child: Box::new(child.into()),
            width: Length::Fit,
            height: Length::Fit,
            scroll_offset_x: 0.0,
            scroll_offset_y: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SidebarElement {
    pub children: Vec<Element>,
    pub width: f32,
}

impl Default for SidebarElement {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            width: 220.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ToolbarElement {
    pub title: String,
    pub children: Vec<Element>,
}

#[derive(Clone, Debug)]
pub struct SplitViewElement {
    pub sidebar: Box<Element>,
    pub main: Box<Element>,
    pub ratio: f32,
    pub resizable: bool,
}

impl DividerElement {
    pub fn horizontal() -> Self {
        Self {
            axis: Axis::Horizontal,
            thickness: 1.0,
            color: Color::WHITE.opacity(0.08),
        }
    }

    pub fn vertical() -> Self {
        Self {
            axis: Axis::Vertical,
            thickness: 1.0,
            color: Color::WHITE.opacity(0.08),
        }
    }
}
