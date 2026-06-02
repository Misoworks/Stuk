use stuk_actions::ActionHitRegion;
use stuk_layout::{
    Axis, FlexItem, GridItem, Rect, Size, flex_layout, grid_layout, stack_layout_items,
};
use stuk_platform::{NativeFrame, WindowBackgroundEffect, WindowChrome, WindowRegions};
use stuk_render::{DisplayList, RectCommand, RoundedRectCommand, TextCommand};
use stuk_style::{Material, NumberSpacing, TextAlign, TextWrap, Theme};

use crate::accessibility::build_accessibility_tree;
use crate::app::{Cx, View};
use crate::control_render::{
    render_avatar, render_badge, render_button, render_checkbox, render_icon_button,
    render_progress_bar, render_radio, render_slider, render_text_field, render_toggle,
    render_tooltip_label,
};
use crate::element::{
    DividerElement, Element, FrameElement, ScrollViewElement, SidebarElement, SplitViewElement,
    StackElement, TextElement, ToolbarElement,
};
use crate::layout_elements::{FlexElement, GridElement, OverlayElement};
use crate::list_elements::VirtualListElement;
use crate::measure::measure_element;
use crate::media_render::render_media;
use crate::option_render::{render_segmented_control, render_tabs};
use crate::surface_render::render_surface_commands;
use crate::window_chrome_render::{content_bounds, render_window_chrome};

pub struct BuiltWindow {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub visible: bool,
    pub active: bool,
    pub always_on_top: bool,
    pub continuous_redraw: bool,
    pub chrome: WindowChrome,
    pub transparent: bool,
    pub background_effect: WindowBackgroundEffect,
    pub regions: WindowRegions,
    material: Material,
    theme: Theme,
    content: Element,
}

pub(crate) fn build_window<V: View>(root: &V, cx: &mut Cx) -> BuiltWindow {
    let app_name = cx.app_name().to_string();
    let view = root.view(cx);
    let theme = cx.theme();
    match view {
        Element::Window(window) => BuiltWindow {
            title: window.title,
            width: window.width,
            height: window.height,
            resizable: window.resizable,
            visible: window.visible,
            active: window.active,
            always_on_top: window.always_on_top,
            chrome: window.chrome,
            transparent: window.transparent,
            background_effect: window.background_effect,
            regions: window.regions,
            continuous_redraw: window.continuous_redraw,
            material: window.material,
            theme,
            content: window
                .content
                .map(|content| *content)
                .unwrap_or(Element::Empty),
        },
        element => BuiltWindow {
            title: app_name.to_string(),
            width: 760,
            height: 520,
            resizable: true,
            visible: true,
            active: true,
            always_on_top: false,
            chrome: WindowChrome::System,
            transparent: false,
            background_effect: WindowBackgroundEffect::None,
            regions: WindowRegions::default(),
            continuous_redraw: false,
            material: Material::Maris,
            theme,
            content: element,
        },
    }
}

pub(crate) fn render_window(
    window: &BuiltWindow,
    size: Size,
    hovered: Option<&str>,
    pressed: Option<&str>,
    focused: Option<&str>,
) -> NativeFrame {
    let theme = &window.theme;
    let mut list = DisplayList::new(window_background(window, theme));
    list.hovered_region = hovered.map(str::to_string);
    list.pressed_region = pressed.map(str::to_string);
    list.focused_region = focused.map(str::to_string);
    let mut hit_regions = Vec::new();
    let bounds = Rect::new(0.0, 0.0, size.width.max(1.0), size.height.max(1.0));
    let panel = bounds;
    let panel_radius = if window.chrome.uses_native_decorations() {
        0.0
    } else {
        theme.radius.lg
    };

    list.push(RoundedRectCommand {
        x: panel.x,
        y: panel.y,
        width: panel.width,
        height: panel.height,
        radius: panel_radius,
        color: window_panel_color(window, theme),
    });
    render_window_chrome(
        window.chrome,
        &window.title,
        panel,
        panel_radius,
        theme,
        &mut list,
        &mut hit_regions,
    );
    let content = content_bounds(window.chrome, panel);
    render_element(&window.content, content, theme, &mut list, &mut hit_regions);
    NativeFrame {
        display_list: list,
        hit_regions,
        accessibility_tree: build_accessibility_tree(&window.title, &window.content, content),
        hovered_id: hovered.map(str::to_string),
        pressed_id: pressed.map(str::to_string),
        continuous_redraw: window.continuous_redraw,
    }
}

fn window_background(window: &BuiltWindow, theme: &Theme) -> stuk_style::Color {
    if window.transparent {
        return stuk_style::Color::rgba(0.0, 0.0, 0.0, 0.0);
    }
    window.material.fallback_color_for(theme)
}

fn window_panel_color(window: &BuiltWindow, theme: &Theme) -> stuk_style::Color {
    if window.transparent {
        return theme.colors.window.opacity(0.84);
    }
    Material::SurfaceElevated.fallback_color_for(theme)
}

fn render_element(
    element: &Element,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    match element {
        Element::Empty | Element::Spacer(_) => {}
        Element::Window(window) => {
            if let Some(content) = &window.content {
                render_element(content, bounds, theme, list, hit_regions);
            }
        }
        Element::Text(text) => render_text(text, bounds, theme, list),
        Element::Button(button) => render_button(button, bounds, theme, list, hit_regions),
        Element::IconButton(button) => render_icon_button(button, bounds, theme, list, hit_regions),
        Element::Toggle(toggle) => render_toggle(toggle, bounds, theme, list, hit_regions),
        Element::Checkbox(checkbox) => render_checkbox(checkbox, bounds, theme, list, hit_regions),
        Element::Radio(radio) => render_radio(radio, bounds, theme, list, hit_regions),
        Element::Slider(slider) => render_slider(slider, bounds, theme, list, hit_regions),
        Element::ProgressBar(progress) => render_progress_bar(progress, bounds, theme, list),
        Element::Tabs(tabs) => render_tabs(tabs, bounds, theme, list, hit_regions),
        Element::SegmentedControl(control) => {
            render_segmented_control(control, bounds, theme, list, hit_regions)
        }
        Element::Badge(badge) => render_badge(badge, bounds, theme, list),
        Element::Avatar(avatar) => render_avatar(avatar, bounds, theme, list),
        Element::Card(card) => render_card(card, bounds, theme, list, hit_regions),
        Element::Tooltip(tooltip) => {
            render_element(&tooltip.child, bounds, theme, list, hit_regions);
            render_tooltip_label(tooltip, bounds, theme, list);
        }
        Element::TextField(field) => render_text_field(field, bounds, theme, list, hit_regions),
        Element::Stack(stack) => render_stack(stack, bounds, theme, list, hit_regions),
        Element::Flex(flex) => render_flex(flex, bounds, theme, list, hit_regions),
        Element::Grid(grid) => render_grid(grid, bounds, theme, list, hit_regions),
        Element::Overlay(overlay) => render_overlay(overlay, bounds, theme, list, hit_regions),
        Element::Surface(surface) => {
            let inner =
                render_surface_commands(surface, surface.surface_bounds(bounds), theme, list);
            render_element(&surface.child, inner, theme, list, hit_regions);
        }
        Element::Media(media) => render_media(media, bounds, theme, list),
        Element::Frame(frame) => render_frame(frame, bounds, theme, list, hit_regions),
        Element::Divider(divider) => render_divider(divider, bounds, theme, list),
        Element::ScrollView(scroll_view) => {
            render_scroll_view(scroll_view, bounds, theme, list, hit_regions)
        }
        Element::VirtualList(virtual_list) => {
            render_virtual_list(virtual_list, bounds, theme, list, hit_regions)
        }
        Element::Sidebar(sidebar) => render_sidebar(sidebar, bounds, theme, list, hit_regions),
        Element::Toolbar(toolbar) => render_toolbar(toolbar, bounds, theme, list, hit_regions),
        Element::SplitView(split_view) => {
            render_split_view(split_view, bounds, theme, list, hit_regions)
        }
    }
}

fn render_stack(
    stack: &StackElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    let child_items = stack
        .children
        .iter()
        .map(measure_element)
        .collect::<Vec<_>>();
    let child_boxes = stack_layout_items(
        stack.axis,
        bounds,
        stack.padding,
        stack.spacing,
        &child_items,
    );

    for (child, child_box) in stack.children.iter().zip(child_boxes) {
        render_element(child, child_box.rect, theme, list, hit_regions);
    }
}

fn render_flex(
    flex: &FlexElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    let items = flex
        .children
        .iter()
        .map(|child| {
            let mut item = FlexItem::new(measure_element(&child.child).size)
                .grow(child.grow)
                .shrink(child.shrink);
            if let Some(basis) = child.basis {
                item = item.basis(basis);
            }
            item
        })
        .collect::<Vec<_>>();
    let boxes = flex_layout(flex.layout, bounds, &items);

    for (child, child_box) in flex.children.iter().zip(boxes) {
        render_element(&child.child, child_box.rect, theme, list, hit_regions);
    }
}

fn render_grid(
    grid: &GridElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    let items = grid
        .children
        .iter()
        .map(|child| {
            GridItem::new(child.column, child.row, measure_element(&child.child).size)
                .span(child.column_span, child.row_span)
        })
        .collect::<Vec<_>>();
    let boxes = grid_layout(&grid.layout, bounds, &items);

    for (child, child_box) in grid.children.iter().zip(boxes) {
        render_element(&child.child, child_box.rect, theme, list, hit_regions);
    }
}

fn render_overlay(
    overlay: &OverlayElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    render_element(&overlay.child, bounds, theme, list, hit_regions);
    let overlay_size = measure_element(&overlay.overlay).size;
    let overlay_bounds =
        overlay
            .alignment
            .place(bounds, overlay_size, overlay.offset_x, overlay.offset_y);
    render_element(&overlay.overlay, overlay_bounds, theme, list, hit_regions);
}

fn render_frame(
    frame: &FrameElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    render_element(
        &frame.child,
        frame.child_bounds(bounds),
        theme,
        list,
        hit_regions,
    );
}

fn render_text(text: &TextElement, bounds: Rect, theme: &Theme, list: &mut DisplayList) {
    list.push(TextCommand {
        text: text.text.clone(),
        x: bounds.x,
        y: bounds.y,
        width: bounds.width.max(1.0),
        height: bounds.height.max(text.line_height),
        size: text.size,
        line_height: text.line_height,
        color: theme.resolve_color(text.color),
        wrap: text.wrap,
        align: text.align,
        number_spacing: text.number_spacing,
    });
}

fn render_card(
    card: &crate::CardElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    list.push(RoundedRectCommand {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width,
        height: bounds.height,
        radius: theme.radius.lg,
        color: theme.colors.surface,
    });
    let inner = Rect::new(
        bounds.x + 16.0,
        bounds.y + 16.0,
        (bounds.width - 32.0).max(1.0),
        (bounds.height - 32.0).max(1.0),
    );
    render_element(&card.child, inner, theme, list, hit_regions);
}

fn render_divider(divider: &DividerElement, bounds: Rect, theme: &Theme, list: &mut DisplayList) {
    list.push(RectCommand {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width,
        height: bounds.height,
        color: theme.resolve_color(divider.color),
    });
}

fn render_scroll_view(
    scroll_view: &ScrollViewElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    let _ = theme;
    render_element(&scroll_view.child, bounds, theme, list, hit_regions);
}

fn render_virtual_list(
    virtual_list: &VirtualListElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    list.push(RoundedRectCommand {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width,
        height: bounds.height,
        radius: theme.radius.md,
        color: theme.colors.surface,
    });

    for index in virtual_list.visible_range() {
        let row_bounds = virtual_list.row_rect(bounds, index);
        if row_bounds.y >= bounds.y + bounds.height || row_bounds.y + row_bounds.height <= bounds.y
        {
            continue;
        }
        let content = Rect::new(
            row_bounds.x + 10.0,
            row_bounds.y + 4.0,
            (row_bounds.width - 20.0).max(1.0),
            (row_bounds.height - 8.0).max(1.0),
        );
        render_element(
            &virtual_list.rows[index].child,
            content,
            theme,
            list,
            hit_regions,
        );
        list.push(RectCommand {
            x: row_bounds.x + 10.0,
            y: row_bounds.y + row_bounds.height - 1.0,
            width: (row_bounds.width - 20.0).max(1.0),
            height: 1.0,
            color: theme.colors.outline.opacity(0.42),
        });
    }
}

fn render_sidebar(
    sidebar: &SidebarElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    list.push(RectCommand {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width,
        height: bounds.height,
        color: Material::Sidebar
            .fallback_color_for(theme)
            .opacity(sidebar.opacity.clamp(0.0, 1.0)),
    });
    let stack = crate::element::StackElement {
        axis: Axis::Vertical,
        padding: stuk_layout::EdgeInsets::all(14.0),
        spacing: 8.0,
        children: sidebar.children.clone(),
    };
    render_stack(&stack, bounds, theme, list, hit_regions);
}

fn render_toolbar(
    toolbar: &ToolbarElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    list.push(RectCommand {
        x: bounds.x,
        y: bounds.y + bounds.height - 1.0,
        width: bounds.width,
        height: 1.0,
        color: theme.colors.outline,
    });
    list.push(TextCommand {
        text: toolbar.title.clone(),
        x: bounds.x,
        y: bounds.y + 9.0,
        width: (bounds.width * 0.45).max(1.0),
        height: 24.0,
        size: 18.0,
        line_height: 24.0,
        color: theme.colors.text,
        wrap: TextWrap::Balance,
        align: TextAlign::Start,
        number_spacing: NumberSpacing::Proportional,
    });
    let actions = Rect::new(
        bounds.x + bounds.width * 0.48,
        bounds.y + 4.0,
        bounds.width * 0.52,
        36.0,
    );
    let stack = crate::element::StackElement {
        axis: Axis::Horizontal,
        padding: stuk_layout::EdgeInsets::default(),
        spacing: 8.0,
        children: toolbar.children.clone(),
    };
    render_stack(&stack, actions, theme, list, hit_regions);
}

fn render_split_view(
    split_view: &SplitViewElement,
    bounds: Rect,
    theme: &Theme,
    list: &mut DisplayList,
    hit_regions: &mut Vec<ActionHitRegion>,
) {
    let sidebar_width = (bounds.width * split_view.ratio).clamp(160.0, bounds.width * 0.5);
    let sidebar = Rect::new(bounds.x, bounds.y, sidebar_width, bounds.height);
    let gap = 18.0;
    let main = Rect::new(
        bounds.x + sidebar_width + gap,
        bounds.y,
        (bounds.width - sidebar_width - gap).max(1.0),
        bounds.height,
    );
    render_element(&split_view.sidebar, sidebar, theme, list, hit_regions);
    list.push(RectCommand {
        x: bounds.x + sidebar_width,
        y: bounds.y,
        width: 1.0,
        height: bounds.height.max(1.0),
        color: theme
            .colors
            .outline
            .opacity(if split_view.resizable { 1.0 } else { 0.55 }),
    });
    render_element(&split_view.main, main, theme, list, hit_regions);
}
