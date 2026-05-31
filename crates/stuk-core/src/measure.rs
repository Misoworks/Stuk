use stuk_layout::{
    Axis, FlexItem, GridItem, GridTrack, LayoutItem, Size, flex_layout, stack_size_items,
};

use crate::element::{
    ButtonElement, DividerElement, Element, FrameElement, ScrollViewElement, SidebarElement,
    SpacerElement, SplitViewElement, StackElement, TextElement, TextFieldElement, ToggleElement,
    ToolbarElement,
};
use crate::layout_elements::{FlexElement, GridElement, OverlayElement};
use crate::list_elements::VirtualListElement;
use crate::media_elements::MediaElement;
use crate::surface_elements::SurfaceElement;
use crate::{
    AvatarElement, BadgeElement, CardElement, CheckboxElement, ControlOptionElement,
    ProgressBarElement, RadioElement, SegmentedControlElement, SliderElement, TabsElement,
    TooltipElement,
};

#[derive(Clone, Copy, Debug, Default)]
struct SizeConstraints {
    min_width: Option<f32>,
    max_width: Option<f32>,
    min_height: Option<f32>,
    max_height: Option<f32>,
}

pub fn measure_element(element: &Element) -> LayoutItem {
    match element {
        Element::Empty => LayoutItem::fit(Size::new(0.0, 0.0)),
        Element::Window(window) => window
            .content
            .as_deref()
            .map(measure_element)
            .unwrap_or(LayoutItem::fit(Size::new(0.0, 0.0))),
        Element::Text(text) => measure_text(text),
        Element::Button(button) => measure_button(button),
        Element::IconButton(_) => LayoutItem::fit(Size::new(38.0, 38.0)),
        Element::Toggle(toggle) => measure_toggle(toggle),
        Element::Checkbox(checkbox) => measure_checkbox(checkbox),
        Element::Radio(radio) => measure_radio(radio),
        Element::Slider(slider) => measure_slider(slider),
        Element::ProgressBar(progress) => measure_progress_bar(progress),
        Element::Tabs(tabs) => measure_tabs(tabs),
        Element::SegmentedControl(control) => measure_segmented_control(control),
        Element::Badge(badge) => measure_badge(badge),
        Element::Avatar(avatar) => measure_avatar(avatar),
        Element::Card(card) => measure_card(card),
        Element::Tooltip(tooltip) => measure_tooltip(tooltip),
        Element::TextField(field) => measure_text_field(field),
        Element::Stack(stack) => measure_stack(stack),
        Element::Flex(flex) => measure_flex(flex),
        Element::Grid(grid) => measure_grid(grid),
        Element::Overlay(overlay) => measure_overlay(overlay),
        Element::Surface(surface) => measure_surface(surface),
        Element::Media(media) => measure_media(media),
        Element::Frame(frame) => measure_frame(frame),
        Element::Spacer(spacer) => measure_spacer(spacer),
        Element::Divider(divider) => measure_divider(divider),
        Element::ScrollView(scroll_view) => measure_scroll_view(scroll_view),
        Element::VirtualList(list) => measure_virtual_list(list),
        Element::Sidebar(sidebar) => measure_sidebar(sidebar),
        Element::Toolbar(toolbar) => measure_toolbar(toolbar),
        Element::SplitView(split_view) => measure_split_view(split_view),
    }
}

fn measure_text(text: &TextElement) -> LayoutItem {
    let width = (text.text.chars().count() as f32 * text.size * 0.56).clamp(1.0, 720.0);
    LayoutItem::fit(Size::new(width, text.line_height))
}

fn measure_button(button: &ButtonElement) -> LayoutItem {
    let width = (button.label.chars().count() as f32 * 8.5 + 40.0).clamp(72.0, 220.0);
    LayoutItem::fit(Size::new(width, 40.0))
}

fn measure_toggle(toggle: &ToggleElement) -> LayoutItem {
    let width = (toggle.label.chars().count() as f32 * 7.5 + 58.0).clamp(96.0, 260.0);
    LayoutItem::fit(Size::new(width, 30.0))
}

fn measure_checkbox(checkbox: &CheckboxElement) -> LayoutItem {
    let width = (checkbox.label.chars().count() as f32 * 7.5 + 34.0).clamp(80.0, 260.0);
    LayoutItem::fit(Size::new(width, 26.0))
}

fn measure_radio(radio: &RadioElement) -> LayoutItem {
    let width = (radio.label.chars().count() as f32 * 7.5 + 34.0).clamp(80.0, 260.0);
    LayoutItem::fit(Size::new(width, 26.0))
}

fn measure_slider(slider: &SliderElement) -> LayoutItem {
    let label_height = f32::from(slider.label.is_some()) * 20.0;
    LayoutItem::fit(Size::new(240.0, 32.0 + label_height))
}

fn measure_progress_bar(progress: &ProgressBarElement) -> LayoutItem {
    let label_height = f32::from(progress.label.is_some()) * 20.0;
    LayoutItem::fit(Size::new(220.0, 18.0 + label_height))
}

fn measure_tabs(tabs: &TabsElement) -> LayoutItem {
    LayoutItem::fit(Size::new(option_row_width(&tabs.options), 36.0))
}

fn measure_segmented_control(control: &SegmentedControlElement) -> LayoutItem {
    let label_height = f32::from(control.label.is_some()) * 20.0;
    LayoutItem::fit(Size::new(
        option_row_width(&control.options).max(120.0),
        34.0 + label_height,
    ))
}

fn measure_badge(badge: &BadgeElement) -> LayoutItem {
    let width = (badge.label.chars().count() as f32 * 7.0 + 18.0).clamp(28.0, 180.0);
    LayoutItem::fit(Size::new(width, 22.0))
}

fn measure_avatar(avatar: &AvatarElement) -> LayoutItem {
    let size = if avatar.initials.chars().count() > 2 {
        46.0
    } else {
        40.0
    };
    LayoutItem::fit(Size::new(size, size))
}

fn measure_card(card: &CardElement) -> LayoutItem {
    let child = measure_element(&card.child).size;
    LayoutItem::fit(Size::new(child.width + 32.0, child.height + 32.0))
}

fn measure_tooltip(tooltip: &TooltipElement) -> LayoutItem {
    measure_element(&tooltip.child)
}

fn option_row_width(options: &[ControlOptionElement]) -> f32 {
    options
        .iter()
        .map(|option| (option.label.chars().count() as f32 * 7.5 + 28.0).clamp(48.0, 180.0))
        .sum::<f32>()
}

fn measure_text_field(field: &TextFieldElement) -> LayoutItem {
    let label_height = f32::from(field.label.is_some()) * 22.0;
    let field_height = if field.multiline { 150.0 } else { 40.0 };
    LayoutItem::fit(Size::new(280.0, field_height + label_height))
}

fn measure_stack(stack: &StackElement) -> LayoutItem {
    let children = stack
        .children
        .iter()
        .map(measure_element)
        .collect::<Vec<_>>();
    LayoutItem::fit(stack_size_items(
        stack.axis,
        stack.padding,
        stack.spacing,
        &children,
    ))
}

fn measure_flex(flex: &FlexElement) -> LayoutItem {
    let items = flex_item_sizes(flex);
    let size = if items.is_empty() {
        Size::new(
            flex.layout.padding.horizontal(),
            flex.layout.padding.vertical(),
        )
    } else {
        let natural = stack_size_items(
            flex.layout.axis,
            flex.layout.padding,
            flex.layout.gap,
            &items
                .iter()
                .map(|item| LayoutItem::fit(item.size))
                .collect::<Vec<_>>(),
        );
        if flex.layout.wrap == stuk_layout::FlexWrap::NoWrap {
            natural
        } else {
            let boxes = flex_layout(
                flex.layout,
                stuk_layout::Rect::new(0.0, 0.0, natural.width, natural.height),
                &items,
            );
            boxes.iter().fold(Size::new(0.0, 0.0), |size, layout_box| {
                Size::new(
                    size.width.max(layout_box.rect.x + layout_box.rect.width),
                    size.height.max(layout_box.rect.y + layout_box.rect.height),
                )
            })
        }
    };

    LayoutItem::fit(size)
        .with_width(flex.width)
        .with_height(flex.height)
}

fn flex_item_sizes(flex: &FlexElement) -> Vec<FlexItem> {
    flex.children
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
        .collect()
}

fn measure_grid(grid: &GridElement) -> LayoutItem {
    let items = grid_items(grid);
    let size = grid_natural_size(grid, &items);
    LayoutItem::fit(size)
        .with_width(grid.width)
        .with_height(grid.height)
}

fn measure_overlay(overlay: &OverlayElement) -> LayoutItem {
    measure_element(&overlay.child)
}

fn measure_surface(surface: &SurfaceElement) -> LayoutItem {
    let child = measure_element(&surface.child).size;
    let visual = LayoutItem::fit(Size::new(
        child.width + surface.padding.horizontal(),
        child.height + surface.padding.vertical(),
    ))
    .with_width(surface.width)
    .with_height(surface.height)
    .size;
    constrained_item(
        LayoutItem::fit(Size::new(
            visual.width + surface.margin.horizontal(),
            visual.height + surface.margin.vertical(),
        )),
        SizeConstraints {
            min_width: surface.min_width,
            max_width: surface.max_width,
            min_height: surface.min_height,
            max_height: surface.max_height,
        },
    )
}

fn constrained_item(mut item: LayoutItem, constraints: SizeConstraints) -> LayoutItem {
    item.size.width = constrain_axis(
        item.size.width,
        constraints.min_width,
        constraints.max_width,
    );
    item.size.height = constrain_axis(
        item.size.height,
        constraints.min_height,
        constraints.max_height,
    );
    item
}

fn constrain_axis(value: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let min = min.unwrap_or(0.0).max(0.0);
    let max = max.unwrap_or(f32::INFINITY).max(min);
    value.clamp(min, max)
}

fn measure_media(media: &MediaElement) -> LayoutItem {
    LayoutItem::fit(media.natural_size)
        .with_width(media.width)
        .with_height(media.height)
}

fn grid_items(grid: &GridElement) -> Vec<GridItem> {
    grid.children
        .iter()
        .map(|child| {
            GridItem::new(child.column, child.row, measure_element(&child.child).size)
                .span(child.column_span, child.row_span)
        })
        .collect()
}

fn grid_natural_size(grid: &GridElement, items: &[GridItem]) -> Size {
    let columns = track_natural_sizes(&grid.layout.columns, items, true);
    let rows = track_natural_sizes(&grid.layout.rows, items, false);
    Size::new(
        columns.iter().sum::<f32>()
            + grid.layout.column_gap * columns.len().saturating_sub(1) as f32
            + grid.layout.padding.horizontal(),
        rows.iter().sum::<f32>()
            + grid.layout.row_gap * rows.len().saturating_sub(1) as f32
            + grid.layout.padding.vertical(),
    )
}

fn track_natural_sizes(tracks: &[GridTrack], items: &[GridItem], columns: bool) -> Vec<f32> {
    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| match track {
            GridTrack::Fixed(value) => value.max(0.0),
            GridTrack::Fraction(value) => (120.0 * value.max(1.0)).max(1.0),
            GridTrack::Fit => items
                .iter()
                .filter(|item| {
                    if columns {
                        item.column == index && item.column_span == 1
                    } else {
                        item.row == index && item.row_span == 1
                    }
                })
                .map(|item| {
                    if columns {
                        item.size.width
                    } else {
                        item.size.height
                    }
                })
                .fold(0.0_f32, f32::max),
        })
        .collect()
}

fn measure_frame(frame: &FrameElement) -> LayoutItem {
    let child = measure_element(&frame.child);
    let visual = Size::new(
        child.size.width + frame.margin.horizontal(),
        child.size.height + frame.margin.vertical(),
    );
    constrained_item(
        LayoutItem::fit(visual)
            .with_width(frame.width)
            .with_height(frame.height),
        SizeConstraints {
            min_width: frame.min_width,
            max_width: frame.max_width,
            min_height: frame.min_height,
            max_height: frame.max_height,
        },
    )
}

fn measure_spacer(spacer: &SpacerElement) -> LayoutItem {
    LayoutItem::fit(Size::new(0.0, 0.0))
        .with_width(spacer.width)
        .with_height(spacer.height)
}

fn measure_divider(divider: &DividerElement) -> LayoutItem {
    match divider.axis {
        Axis::Horizontal => LayoutItem::fixed(Size::new(0.0, divider.thickness))
            .with_width(stuk_layout::Length::Fill),
        Axis::Vertical => LayoutItem::fixed(Size::new(divider.thickness, 0.0))
            .with_height(stuk_layout::Length::Fill),
        Axis::Depth => LayoutItem::fixed(Size::new(divider.thickness, divider.thickness)),
    }
}

fn measure_scroll_view(scroll_view: &ScrollViewElement) -> LayoutItem {
    let child = measure_element(&scroll_view.child);
    LayoutItem::fit(child.size)
        .with_width(scroll_view.width)
        .with_height(scroll_view.height)
}

fn measure_virtual_list(list: &VirtualListElement) -> LayoutItem {
    let visible_width = list
        .visible_range()
        .map(|index| measure_element(&list.rows[index].child).size.width)
        .fold(320.0_f32, f32::max);
    LayoutItem::fit(Size::new(
        visible_width,
        list.viewport_height.min(list.total_height()).max(1.0),
    ))
    .with_width(list.width)
    .with_height(stuk_layout::Length::Fixed(list.viewport_height.max(1.0)))
}

fn measure_sidebar(sidebar: &SidebarElement) -> LayoutItem {
    let stack = crate::element::StackElement {
        axis: Axis::Vertical,
        padding: stuk_layout::EdgeInsets::all(14.0),
        spacing: 8.0,
        children: sidebar.children.clone(),
    };
    LayoutItem::fixed(Size::new(
        sidebar.width,
        measure_stack(&stack).size.height.max(180.0),
    ))
}

fn measure_toolbar(toolbar: &ToolbarElement) -> LayoutItem {
    let children = toolbar
        .children
        .iter()
        .map(measure_element)
        .collect::<Vec<_>>();
    let child_width = children.iter().map(|child| child.size.width).sum::<f32>();
    let title_width = (toolbar.title.chars().count() as f32 * 10.0).max(120.0);
    LayoutItem::fit(Size::new(
        (title_width + child_width + 48.0).max(360.0),
        44.0,
    ))
}

fn measure_split_view(split_view: &SplitViewElement) -> LayoutItem {
    let sidebar = measure_element(&split_view.sidebar).size;
    let main = measure_element(&split_view.main).size;
    LayoutItem::fit(Size::new(
        (sidebar.width + main.width + 14.0).max(620.0),
        sidebar.height.max(main.height).max(260.0),
    ))
}
