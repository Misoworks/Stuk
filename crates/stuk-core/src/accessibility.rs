use stuk_accessibility::{
    AccessRect, AccessibilityTree, AccessibilityTreeBuilder, Action, Node, Role, Toggled,
};
use stuk_layout::{Axis, Rect, stack_layout_items};

use crate::control_accessibility::{
    checkbox_node, label_node, progress_node, radio_node, segmented_control_node, slider_node,
    tabs_node,
};
use crate::element::{
    ButtonElement, DividerElement, Element, IconButtonElement, ScrollViewElement, SidebarElement,
    SplitViewElement, StackElement, TextElement, TextFieldElement, ToggleElement, ToolbarElement,
};
use crate::layout_accessibility::{flex_node, grid_node, overlay_node};
use crate::list_elements::VirtualListElement;
use crate::measure::measure_element;
use crate::media_elements::MediaElement;
use crate::surface_elements::SurfaceElement;

pub(crate) fn build_accessibility_tree(
    title: &str,
    content: &Element,
    bounds: Rect,
) -> AccessibilityTree {
    let mut builder = AccessibilityTreeBuilder::new();
    let children = element_child(content, bounds, &mut builder)
        .into_iter()
        .collect::<Vec<_>>();
    let mut root = node(Role::Window, bounds);
    root.set_label(title);
    root.set_children(children);
    let root = builder.push(root);
    builder.finish(root)
}

pub(crate) fn element_child(
    element: &Element,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> Option<stuk_accessibility::NodeId> {
    match element {
        Element::Empty | Element::Spacer(_) => None,
        Element::Window(window) => window
            .content
            .as_deref()
            .and_then(|content| element_child(content, bounds, builder)),
        Element::Text(text) => Some(text_node(text, bounds, builder)),
        Element::Button(button) => Some(button_node(button, bounds, builder)),
        Element::IconButton(button) => Some(icon_button_node(button, bounds, builder)),
        Element::Toggle(toggle) => Some(toggle_node(toggle, bounds, builder)),
        Element::Checkbox(checkbox) => Some(checkbox_node(checkbox, bounds, builder)),
        Element::Radio(radio) => Some(radio_node(radio, bounds, builder)),
        Element::Slider(slider) => Some(slider_node(slider, bounds, builder)),
        Element::ProgressBar(progress) => Some(progress_node(progress, bounds, builder)),
        Element::Tabs(tabs) => Some(tabs_node(tabs, bounds, builder)),
        Element::SegmentedControl(control) => {
            Some(segmented_control_node(control, bounds, builder))
        }
        Element::Badge(badge) => Some(label_node(&badge.label, bounds, builder)),
        Element::Avatar(avatar) => Some(label_node(&avatar.label, bounds, builder)),
        Element::Card(card) => element_child(&card.child, bounds, builder),
        Element::Tooltip(tooltip) => element_child(&tooltip.child, bounds, builder),
        Element::TextField(field) => Some(text_field_node(field, bounds, builder)),
        Element::Stack(stack) => Some(stack_node(stack, bounds, builder)),
        Element::Flex(flex) => Some(flex_node(flex, bounds, builder)),
        Element::Grid(grid) => Some(grid_node(grid, bounds, builder)),
        Element::Overlay(overlay) => Some(overlay_node(overlay, bounds, builder)),
        Element::Surface(surface) => Some(surface_node(surface, bounds, builder)),
        Element::Media(media) => media_node(media, bounds, builder),
        Element::Frame(frame) => element_child(&frame.child, frame.child_bounds(bounds), builder),
        Element::Divider(divider) => Some(divider_node(divider, bounds, builder)),
        Element::ScrollView(scroll_view) => Some(scroll_view_node(scroll_view, bounds, builder)),
        Element::VirtualList(list) => Some(virtual_list_node(list, bounds, builder)),
        Element::Sidebar(sidebar) => Some(sidebar_node(sidebar, bounds, builder)),
        Element::Toolbar(toolbar) => Some(toolbar_node(toolbar, bounds, builder)),
        Element::SplitView(split_view) => Some(split_view_node(split_view, bounds, builder)),
    }
}

fn stack_node(
    stack: &StackElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let children = stack_children(
        &stack.children,
        stack.axis,
        stack.padding,
        stack.spacing,
        bounds,
        builder,
    );
    let mut node = node(Role::Group, bounds);
    node.set_children(children);
    builder.push(node)
}

fn surface_node(
    surface: &SurfaceElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let child = element_child(&surface.child, surface.inner_bounds(bounds), builder);
    let mut node = node(Role::Group, bounds);
    node.set_children(child.into_iter().collect::<Vec<_>>());
    builder.push(node)
}

fn media_node(
    media: &MediaElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> Option<stuk_accessibility::NodeId> {
    if media.decorative {
        return None;
    }

    let mut node = node(Role::Image, bounds);
    if let Some(label) = &media.label {
        node.set_label(label.clone());
    }
    Some(builder.push(node))
}

fn stack_children(
    children: &[Element],
    axis: Axis,
    padding: stuk_layout::EdgeInsets,
    spacing: f32,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> Vec<stuk_accessibility::NodeId> {
    let items = children.iter().map(measure_element).collect::<Vec<_>>();
    let boxes = stack_layout_items(axis, bounds, padding, spacing, &items);
    children
        .iter()
        .zip(boxes)
        .filter_map(|(child, child_box)| element_child(child, child_box.rect, builder))
        .collect()
}

fn text_node(
    text: &TextElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let mut node = node(
        if text.size >= 22.0 {
            Role::Heading
        } else {
            Role::Label
        },
        bounds,
    );
    node.set_value(text.text.clone());
    builder.push(node)
}

fn button_node(
    button: &ButtonElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let bounds = Rect::new(bounds.x, bounds.y, bounds.width.min(220.0), bounds.height);
    let mut node = node(Role::Button, bounds);
    node.set_label(button.label.clone());
    if button.disabled {
        node.set_disabled();
    } else if button.action.is_some() {
        node.add_action(Action::Click);
    }
    builder.push(node)
}

fn icon_button_node(
    button: &IconButtonElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let bounds = Rect::new(
        bounds.x,
        bounds.y,
        bounds.width.min(38.0),
        bounds.height.min(38.0),
    );
    let mut node = node(Role::Button, bounds);
    node.set_label(button.label.clone());
    node.set_value(button.icon.clone());
    if button.disabled {
        node.set_disabled();
    } else if button.action.is_some() {
        node.add_action(Action::Click);
    }
    builder.push(node)
}

fn toggle_node(
    toggle: &ToggleElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let mut node = node(Role::Switch, bounds);
    node.set_label(toggle.label.clone());
    node.set_toggled(Toggled::from(toggle.checked));
    if toggle.disabled {
        node.set_disabled();
    } else if toggle.action.is_some() {
        node.add_action(Action::Click);
    }
    builder.push(node)
}

fn text_field_node(
    field: &TextFieldElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let label_height = f32::from(field.label.is_some()) * 22.0;
    let field_rect = Rect::new(
        bounds.x,
        bounds.y + label_height,
        bounds.width.min(280.0),
        38.0,
    );
    let mut node = node(Role::TextInput, field_rect);
    if let Some(label) = &field.label {
        node.set_label(label.clone());
    }
    if !field.text.is_empty() {
        node.set_value(field.text.clone());
    }
    if !field.placeholder.is_empty() {
        node.set_placeholder(field.placeholder.clone());
    }
    if field.disabled {
        node.set_disabled();
    } else {
        node.add_action(Action::Focus);
        node.add_action(Action::SetValue);
    }
    builder.push(node)
}

fn divider_node(
    _divider: &DividerElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    builder.push(node(Role::Splitter, bounds))
}

fn scroll_view_node(
    scroll_view: &ScrollViewElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let child = element_child(&scroll_view.child, bounds, builder);
    let mut node = node(Role::ScrollView, bounds);
    if let Some(child) = child {
        node.push_child(child);
    }
    builder.push(node)
}

fn virtual_list_node(
    list: &VirtualListElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let children = list
        .visible_range()
        .filter_map(|index| {
            let row = &list.rows[index];
            let row_bounds = list.row_rect(bounds, index);
            let child = element_child(&row.child, row_bounds, builder);
            let mut row_node = node(Role::ListItem, row_bounds);
            row_node.set_label(row.key.clone());
            if let Some(child) = child {
                row_node.push_child(child);
            }
            Some(builder.push(row_node))
        })
        .collect::<Vec<_>>();

    let mut node = node(Role::List, bounds);
    node.set_children(children);
    node.add_action(Action::ScrollDown);
    node.add_action(Action::ScrollUp);
    builder.push(node)
}

fn sidebar_node(
    sidebar: &SidebarElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let stack = crate::element::StackElement {
        axis: Axis::Vertical,
        padding: stuk_layout::EdgeInsets::all(14.0),
        spacing: 8.0,
        children: sidebar.children.clone(),
    };
    let children = stack_children(
        &stack.children,
        stack.axis,
        stack.padding,
        stack.spacing,
        bounds,
        builder,
    );
    let mut node = node(Role::Navigation, bounds);
    node.set_children(children);
    builder.push(node)
}

fn toolbar_node(
    toolbar: &ToolbarElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let mut title = node(
        Role::Heading,
        Rect::new(bounds.x, bounds.y + 9.0, bounds.width * 0.45, 24.0),
    );
    title.set_value(toolbar.title.clone());
    let title = builder.push(title);

    let action_bounds = Rect::new(
        bounds.x + bounds.width * 0.48,
        bounds.y + 4.0,
        bounds.width * 0.52,
        36.0,
    );
    let action_children = stack_children(
        &toolbar.children,
        Axis::Horizontal,
        stuk_layout::EdgeInsets::default(),
        8.0,
        action_bounds,
        builder,
    );

    let mut children = Vec::with_capacity(action_children.len() + 1);
    children.push(title);
    children.extend(action_children);

    let mut node = node(Role::Toolbar, bounds);
    node.set_children(children);
    builder.push(node)
}

fn split_view_node(
    split_view: &SplitViewElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let sidebar_width = (bounds.width * split_view.ratio).clamp(160.0, bounds.width * 0.5);
    let sidebar_bounds = Rect::new(bounds.x, bounds.y, sidebar_width, bounds.height);
    let main_bounds = Rect::new(
        bounds.x + sidebar_width + 14.0,
        bounds.y,
        (bounds.width - sidebar_width - 14.0).max(1.0),
        bounds.height,
    );

    let mut children = Vec::new();
    if let Some(sidebar) = element_child(&split_view.sidebar, sidebar_bounds, builder) {
        children.push(sidebar);
    }
    if let Some(main) = element_child(&split_view.main, main_bounds, builder) {
        children.push(main);
    }

    let mut node = node(Role::Group, bounds);
    node.set_children(children);
    builder.push(node)
}

pub(crate) fn node(role: Role, bounds: Rect) -> Node {
    let mut node = Node::new(role);
    node.set_bounds(access_bounds(bounds));
    node
}

fn access_bounds(bounds: Rect) -> AccessRect {
    AccessRect::new(
        bounds.x.into(),
        bounds.y.into(),
        (bounds.x + bounds.width).into(),
        (bounds.y + bounds.height).into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{ButtonElement, ToggleElement};
    use crate::{CheckboxElement, SliderElement};
    use stuk_style::ButtonVariant;

    #[test]
    fn exposes_actionable_widget_metadata() {
        let content = Element::Stack(crate::element::StackElement {
            axis: Axis::Vertical,
            padding: stuk_layout::EdgeInsets::default(),
            spacing: 8.0,
            children: vec![
                Element::Button(ButtonElement {
                    label: "Save".to_string(),
                    variant: ButtonVariant::Primary,
                    action: Some("document.save".to_string()),
                    disabled: false,
                    text_align: stuk_style::ControlTextAlign::Center,
                }),
                Element::Toggle(ToggleElement {
                    label: "Sync".to_string(),
                    checked: true,
                    action: Some("settings.sync.enabled".to_string()),
                    disabled: false,
                }),
            ],
        });

        let tree = build_accessibility_tree("Test", &content, Rect::new(0.0, 0.0, 400.0, 160.0));

        let button = tree
            .nodes()
            .iter()
            .map(|(_, node)| node)
            .find(|node| node.role() == Role::Button)
            .expect("button node should exist");
        assert_eq!(button.label(), Some("Save"));
        assert!(button.supports_action(Action::Click));

        let toggle = tree
            .nodes()
            .iter()
            .map(|(_, node)| node)
            .find(|node| node.role() == Role::Switch)
            .expect("toggle node should exist");
        assert_eq!(toggle.label(), Some("Sync"));
        assert_eq!(toggle.toggled(), Some(Toggled::True));
        assert!(toggle.supports_action(Action::Click));
    }

    #[test]
    fn exposes_first_class_control_roles() {
        let content = Element::Stack(crate::element::StackElement {
            axis: Axis::Vertical,
            padding: stuk_layout::EdgeInsets::default(),
            spacing: 8.0,
            children: vec![
                Element::Checkbox(CheckboxElement {
                    label: "Remember".to_string(),
                    checked: true,
                    action: Some("settings.remember".to_string()),
                    disabled: false,
                }),
                Element::Slider(SliderElement {
                    label: Some("Volume".to_string()),
                    value: 42.0,
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                    action: Some("settings.volume".to_string()),
                    disabled: false,
                }),
            ],
        });

        let tree = build_accessibility_tree("Test", &content, Rect::new(0.0, 0.0, 400.0, 160.0));

        let checkbox = tree
            .nodes()
            .iter()
            .map(|(_, node)| node)
            .find(|node| node.role() == Role::CheckBox)
            .expect("checkbox node should exist");
        assert_eq!(checkbox.label(), Some("Remember"));
        assert_eq!(checkbox.toggled(), Some(Toggled::True));

        let slider = tree
            .nodes()
            .iter()
            .map(|(_, node)| node)
            .find(|node| node.role() == Role::Slider)
            .expect("slider node should exist");
        assert_eq!(slider.label(), Some("Volume"));
        assert_eq!(slider.numeric_value(), Some(42.0));
        assert_eq!(slider.max_numeric_value(), Some(100.0));
    }
}
