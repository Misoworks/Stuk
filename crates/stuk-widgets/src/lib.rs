mod app_shell;
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

pub use app_shell::{AppShell, CommandBar, ListSection, PageShell, Pane};
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
pub use layout_widgets::{Center, Flex, Grid, Overlay};
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
use stuk_platform::{WindowBackgroundEffect, WindowChrome, WindowRegion, WindowRegions};
use stuk_style::{ButtonVariant, Color, Material, NumberSpacing, TextAlign, TextWrap};

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

    pub fn glass(mut self) -> Self {
        self.element.material = Material::Luca;
        self.element.chrome = WindowChrome::System;
        self.element.transparent = true;
        self.element.background_effect = WindowBackgroundEffect::Luca;
        self
    }

    pub fn glass_material(mut self, material: Material) -> Self {
        self.element.background_effect = background_effect_for_glass_material(&material);
        self.element.material = material;
        self.element.chrome = WindowChrome::System;
        self.element.transparent = true;
        self
    }

    pub fn transparent(mut self, enabled: bool) -> Self {
        self.element.transparent = enabled;
        if !enabled {
            self.element.background_effect = WindowBackgroundEffect::None;
        }
        self
    }

    pub fn background_effect(mut self, effect: WindowBackgroundEffect) -> Self {
        self.element.background_effect = effect;
        if effect.requires_transparency() {
            self.element.transparent = true;
        }
        self
    }

    pub fn regions(mut self, regions: WindowRegions) -> Self {
        self.element.regions = regions;
        self
    }

    pub fn blur_region(mut self, region: WindowRegion) -> Self {
        self.element.regions.blur = Some(region);
        self
    }

    pub fn input_region(mut self, region: WindowRegion) -> Self {
        self.element.regions.input = Some(region);
        self
    }

    pub fn opaque_region(mut self, region: WindowRegion) -> Self {
        self.element.regions.opaque = Some(region);
        self
    }

    pub fn rounded_window_region(mut self, radius: i32) -> Self {
        self.element.regions.input = Some(WindowRegion::adaptive_rounded_rect(radius));
        self
    }

    pub fn sidebar_blur_region(mut self, sidebar_width: i32, radius: i32) -> Self {
        self.element.regions.blur =
            Some(WindowRegion::adaptive_rounded_left(sidebar_width, radius));
        self
    }

    pub fn titlebar_sidebar_blur_region(
        mut self,
        sidebar_width: i32,
        titlebar_height: i32,
        radius: i32,
    ) -> Self {
        self.element.regions.blur = Some(WindowRegion::adaptive_titlebar_sidebar(
            sidebar_width,
            titlebar_height,
            radius,
        ));
        self
    }

    pub fn content_opaque_region(mut self, sidebar_width: i32, titlebar_height: i32) -> Self {
        self.element.regions.opaque = Some(WindowRegion::adaptive_content_after_sidebar(
            sidebar_width,
            titlebar_height,
        ));
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.element.width = width;
        self.element.height = height;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.element.resizable = resizable;
        self
    }

    pub fn fixed_size(mut self) -> Self {
        self.element.resizable = false;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.element.visible = visible;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.element.active = active;
        self
    }

    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.element.always_on_top = always_on_top;
        self
    }

    pub fn continuous_redraw(mut self, enabled: bool) -> Self {
        self.element.continuous_redraw = enabled;
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

fn background_effect_for_glass_material(material: &Material) -> WindowBackgroundEffect {
    match material {
        Material::Luca => WindowBackgroundEffect::Luca,
        Material::Niko => WindowBackgroundEffect::Niko,
        Material::Maris => WindowBackgroundEffect::Maris,
        _ => WindowBackgroundEffect::Blur,
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
                wrap: TextWrap::Normal,
                number_spacing: NumberSpacing::Proportional,
                align: TextAlign::Start,
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
                wrap: TextWrap::Balance,
                number_spacing: NumberSpacing::Proportional,
                align: TextAlign::Start,
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

    pub fn balance(mut self) -> Self {
        self.element.wrap = TextWrap::Balance;
        self
    }

    pub fn pretty(mut self) -> Self {
        self.element.wrap = TextWrap::Pretty;
        self
    }

    pub fn tabular_nums(mut self) -> Self {
        self.element.number_spacing = NumberSpacing::Tabular;
        self
    }

    pub fn centered(mut self) -> Self {
        self.element.align = TextAlign::Center;
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
                text_align: stuk_style::ControlTextAlign::Center,
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

    pub fn align_start(mut self) -> Self {
        self.element.text_align = stuk_style::ControlTextAlign::Start;
        self
    }

    pub fn align_center(mut self) -> Self {
        self.element.text_align = stuk_style::ControlTextAlign::Center;
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

#[cfg(test)]
mod tests {
    use super::*;
    use stuk_core::Element;

    #[test]
    fn window_builder_produces_window_element() {
        let window = Window::new()
            .title("Test")
            .material(Material::Maris)
            .chrome(WindowChrome::System);
        let element: Element = window.into();
        assert!(matches!(element, Element::Window(_)));
    }

    #[test]
    fn window_effect_enables_transparency() {
        let window = Window::new().background_effect(WindowBackgroundEffect::Mica);
        let Element::Window(element) = Element::from(window) else {
            panic!("window builder should produce a window element");
        };

        assert!(element.transparent);
        assert_eq!(element.background_effect, WindowBackgroundEffect::Mica);
    }

    #[test]
    fn glass_window_sets_coherent_defaults() {
        let window = Window::new().glass();
        let Element::Window(element) = Element::from(window) else {
            panic!("window builder should produce a window element");
        };

        assert!(element.transparent);
        assert_eq!(element.background_effect, WindowBackgroundEffect::Luca);
        assert_eq!(element.chrome, WindowChrome::System);
        assert_eq!(element.material, Material::Luca);
    }

    #[test]
    fn glass_material_selects_matching_background_effect() {
        let window = Window::new().glass_material(Material::Niko);
        let Element::Window(element) = Element::from(window) else {
            panic!("window builder should produce a window element");
        };

        assert!(element.transparent);
        assert_eq!(element.background_effect, WindowBackgroundEffect::Niko);
        assert_eq!(element.material, Material::Niko);
    }

    #[test]
    fn disabling_window_transparency_clears_effect() {
        let window = Window::new()
            .background_effect(WindowBackgroundEffect::Acrylic)
            .transparent(false);
        let Element::Window(element) = Element::from(window) else {
            panic!("window builder should produce a window element");
        };

        assert!(!element.transparent);
        assert_eq!(element.background_effect, WindowBackgroundEffect::None);
    }

    #[test]
    fn text_builder_produces_text_element() {
        let text = Text::new("hello");
        let element: Element = text.into();
        assert!(matches!(element, Element::Text(_)));
    }

    #[test]
    fn text_title_produces_text_element() {
        let text = Text::title("Heading");
        let element: Element = text.into();
        assert!(matches!(element, Element::Text(_)));
    }

    #[test]
    fn button_primary_produces_button() {
        let button = Button::primary("Save");
        let element: Element = button.into();
        assert!(matches!(element, Element::Button(_)));
    }

    #[test]
    fn button_variants_all_build() {
        let _element: Element = Button::primary("OK").into();
        let _element: Element = Button::secondary("Cancel").into();
        let _element: Element = Button::ghost("Skip").into();
        let _element: Element = Button::destructive("Delete").into();
    }

    #[test]
    fn vstack_accumulates_children() {
        let stack = VStack::new()
            .spacing(8.0)
            .padding(12.0)
            .child(Text::new("a"))
            .child(Text::new("b"));
        let _element: Element = stack.into();
    }

    #[test]
    fn hstack_accumulates_children() {
        let stack = HStack::new()
            .spacing(10.0)
            .child(Button::new("OK"))
            .child(Spacer::new())
            .child(Button::secondary("Cancel"));
        let _element: Element = stack.into();
    }

    #[test]
    fn split_view_builds() {
        let split = SplitView::new(
            Sidebar::new().child(Text::new("sidebar")),
            VStack::new().child(Text::new("main")),
        )
        .resizable(true);
        let _element: Element = split.into();
    }

    #[test]
    fn toggle_builds() {
        let toggle = Toggle::new("Sync", true).action("sync.toggle");
        let _element: Element = toggle.into();
    }

    #[test]
    fn text_field_builder() {
        let field = TextField::new("").label("Name").placeholder("Enter name");
        let _element: Element = field.into();
    }

    #[test]
    fn scroll_view_builder() {
        let scroll = ScrollView::new(Text::new("content")).height(200.0);
        let _element: Element = scroll.into();
    }

    #[test]
    fn divider_builds() {
        let h = Divider::horizontal();
        let v = Divider::vertical();
        let _h_elem: Element = h.into();
        let _v_elem: Element = v.into();
    }

    #[test]
    fn icon_button_builds() {
        let ib = IconButton::new("X", "Close").action("app.close");
        let _element: Element = ib.into();
    }

    #[test]
    fn empty_state_builds() {
        let es = EmptyState::new("Nothing here").message("Try creating something");
        let _element: Element = es.into();
    }

    #[test]
    fn frame_sizes() {
        let frame = Frame::new(Text::new("content")).width(400.0).height(300.0);
        let element: Element = frame.into();
        assert!(matches!(element, Element::Frame(_)));
    }

    #[test]
    fn spacer_fills() {
        let _element: Element = Spacer::new().into();
    }

    #[test]
    fn muted_text_is_still_text() {
        let text = Text::new("muted").muted();
        let element: Element = text.into();
        assert!(matches!(element, Element::Text(_)));
    }

    #[test]
    fn sized_text_is_still_text() {
        let text = Text::new("sized").size(20.0);
        let element: Element = text.into();
        assert!(matches!(element, Element::Text(_)));
    }
}
