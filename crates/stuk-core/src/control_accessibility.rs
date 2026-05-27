use stuk_accessibility::{AccessRect, AccessibilityTreeBuilder, Action, Node, Role, Toggled};
use stuk_layout::Rect;

use crate::{
    CheckboxElement, ControlOptionElement, ProgressBarElement, RadioElement,
    SegmentedControlElement, SliderElement, TabsElement,
};

pub(crate) fn checkbox_node(
    checkbox: &CheckboxElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let mut node = node(Role::CheckBox, bounds);
    node.set_label(checkbox.label.clone());
    node.set_toggled(Toggled::from(checkbox.checked));
    if checkbox.disabled {
        node.set_disabled();
    } else if checkbox.action.is_some() {
        node.add_action(Action::Click);
    }
    builder.push(node)
}

pub(crate) fn radio_node(
    radio: &RadioElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let mut node = node(Role::RadioButton, bounds);
    node.set_label(radio.label.clone());
    node.set_toggled(Toggled::from(radio.selected));
    if radio.disabled {
        node.set_disabled();
    } else if radio.action.is_some() {
        node.add_action(Action::Click);
    }
    builder.push(node)
}

pub(crate) fn slider_node(
    slider: &SliderElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let mut node = node(Role::Slider, bounds);
    if let Some(label) = &slider.label {
        node.set_label(label.clone());
    }
    node.set_numeric_value(f64::from(slider.value.clamp(slider.min, slider.max)));
    node.set_min_numeric_value(f64::from(slider.min));
    node.set_max_numeric_value(f64::from(slider.max));
    node.set_numeric_value_step(f64::from(slider.step.max(0.0)));
    if slider.disabled {
        node.set_disabled();
    } else {
        node.add_action(Action::SetValue);
        if slider.action.is_some() {
            node.add_action(Action::Click);
        }
    }
    builder.push(node)
}

pub(crate) fn progress_node(
    progress: &ProgressBarElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let label = progress.label.as_deref().unwrap_or("Progress");
    let mut node = node(Role::ProgressIndicator, bounds);
    node.set_label(label);
    node.set_numeric_value(f64::from(progress.value.clamp(0.0, progress.max.max(0.0))));
    node.set_min_numeric_value(0.0);
    node.set_max_numeric_value(f64::from(progress.max.max(0.0)));
    builder.push(node)
}

pub(crate) fn tabs_node(
    tabs: &TabsElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let children = option_nodes(&tabs.options, tabs.selected, Role::Tab, bounds, builder);
    let mut node = node(Role::TabList, bounds);
    node.set_children(children);
    builder.push(node)
}

pub(crate) fn segmented_control_node(
    control: &SegmentedControlElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let children = option_nodes(
        &control.options,
        control.selected,
        Role::RadioButton,
        bounds,
        builder,
    );
    let mut node = node(Role::RadioGroup, bounds);
    if let Some(label) = &control.label {
        node.set_label(label.clone());
    }
    node.set_children(children);
    builder.push(node)
}

pub(crate) fn label_node(
    label: &str,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> stuk_accessibility::NodeId {
    let mut node = node(Role::Label, bounds);
    node.set_label(label);
    builder.push(node)
}

fn option_nodes(
    options: &[ControlOptionElement],
    selected: usize,
    role: Role,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> Vec<stuk_accessibility::NodeId> {
    let width = if options.is_empty() {
        bounds.width
    } else {
        bounds.width / options.len() as f32
    };
    options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let option_bounds = Rect::new(
                bounds.x + width * index as f32,
                bounds.y,
                width,
                bounds.height,
            );
            let mut node = node(role, option_bounds);
            node.set_label(option.label.clone());
            if role == Role::Tab {
                node.set_selected(index == selected);
            } else {
                node.set_toggled(Toggled::from(index == selected));
            }
            if option.disabled {
                node.set_disabled();
            } else if option.action.is_some() {
                node.add_action(Action::Click);
            }
            builder.push(node)
        })
        .collect()
}

fn node(role: Role, bounds: Rect) -> Node {
    let mut node = Node::new(role);
    node.set_bounds(AccessRect::new(
        bounds.x.into(),
        bounds.y.into(),
        (bounds.x + bounds.width).into(),
        (bounds.y + bounds.height).into(),
    ));
    node
}
