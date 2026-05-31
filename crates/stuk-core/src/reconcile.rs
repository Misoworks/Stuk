use crate::element::{Element, ElementKind};
use crate::list_elements::VirtualListElement;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReconcileOp {
    Insert {
        path: Vec<usize>,
        kind: ElementKind,
    },
    Remove {
        path: Vec<usize>,
        kind: ElementKind,
    },
    Replace {
        path: Vec<usize>,
        from: ElementKind,
        to: ElementKind,
    },
    Update {
        path: Vec<usize>,
        kind: ElementKind,
    },
}

pub fn reconcile(previous: &Element, next: &Element) -> Vec<ReconcileOp> {
    let mut ops = Vec::new();
    reconcile_at(Vec::new(), previous, next, &mut ops);
    ops
}

fn reconcile_at(path: Vec<usize>, previous: &Element, next: &Element, ops: &mut Vec<ReconcileOp>) {
    let previous_kind = previous.kind();
    let next_kind = next.kind();

    if previous_kind != next_kind {
        ops.push(ReconcileOp::Replace {
            path,
            from: previous_kind,
            to: next_kind,
        });
        return;
    }

    if shallow_changed(previous, next) {
        ops.push(ReconcileOp::Update {
            path: path.clone(),
            kind: next_kind,
        });
    }

    if let (Element::VirtualList(previous), Element::VirtualList(next)) = (previous, next) {
        reconcile_virtual_list(path, previous, next, ops);
        return;
    }

    let previous_len = child_count(previous);
    let next_len = child_count(next);
    let max_len = previous_len.max(next_len);

    for index in 0..max_len {
        let mut child_path = path.clone();
        child_path.push(index);
        match (child_at(previous, index), child_at(next, index)) {
            (Some(previous_child), Some(next_child)) => {
                reconcile_at(child_path, previous_child, next_child, ops);
            }
            (Some(previous_child), None) => ops.push(ReconcileOp::Remove {
                path: child_path,
                kind: previous_child.kind(),
            }),
            (None, Some(next_child)) => ops.push(ReconcileOp::Insert {
                path: child_path,
                kind: next_child.kind(),
            }),
            (None, None) => {}
        }
    }
}

fn child_count(element: &Element) -> usize {
    match element {
        Element::Window(window) => usize::from(window.content.is_some()),
        Element::Stack(stack) => stack.children.len(),
        Element::Flex(flex) => flex.children.len(),
        Element::Grid(grid) => grid.children.len(),
        Element::Overlay(_) => 2,
        Element::Surface(_) => 1,
        Element::Frame(_) => 1,
        Element::Card(_) => 1,
        Element::Tooltip(_) => 1,
        Element::ScrollView(_) => 1,
        Element::VirtualList(list) => list.rows.len(),
        Element::Sidebar(sidebar) => sidebar.children.len(),
        Element::Toolbar(toolbar) => toolbar.children.len(),
        Element::SplitView(_) => 2,
        Element::Empty
        | Element::Text(_)
        | Element::Media(_)
        | Element::Button(_)
        | Element::IconButton(_)
        | Element::Toggle(_)
        | Element::Checkbox(_)
        | Element::Radio(_)
        | Element::Slider(_)
        | Element::ProgressBar(_)
        | Element::Tabs(_)
        | Element::SegmentedControl(_)
        | Element::Badge(_)
        | Element::Avatar(_)
        | Element::TextField(_)
        | Element::Spacer(_)
        | Element::Divider(_) => 0,
    }
}

fn child_at(element: &Element, index: usize) -> Option<&Element> {
    match element {
        Element::Window(window) if index == 0 => window.content.as_deref(),
        Element::Stack(stack) => stack.children.get(index),
        Element::Flex(flex) => flex.children.get(index).map(|child| &child.child),
        Element::Grid(grid) => grid.children.get(index).map(|child| &child.child),
        Element::Overlay(overlay) if index == 0 => Some(&overlay.child),
        Element::Overlay(overlay) if index == 1 => Some(&overlay.overlay),
        Element::Surface(surface) if index == 0 => Some(&surface.child),
        Element::Frame(frame) if index == 0 => Some(&frame.child),
        Element::Card(card) if index == 0 => Some(&card.child),
        Element::Tooltip(tooltip) if index == 0 => Some(&tooltip.child),
        Element::ScrollView(scroll_view) if index == 0 => Some(&scroll_view.child),
        Element::VirtualList(list) => list.rows.get(index).map(|row| &row.child),
        Element::Sidebar(sidebar) => sidebar.children.get(index),
        Element::Toolbar(toolbar) => toolbar.children.get(index),
        Element::SplitView(split_view) if index == 0 => Some(&split_view.sidebar),
        Element::SplitView(split_view) if index == 1 => Some(&split_view.main),
        _ => None,
    }
}

fn shallow_changed(previous: &Element, next: &Element) -> bool {
    match (previous, next) {
        (Element::Empty, Element::Empty) => false,
        (Element::Window(previous), Element::Window(next)) => {
            previous.title != next.title
                || previous.material != next.material
                || previous.width != next.width
                || previous.height != next.height
        }
        (Element::Text(previous), Element::Text(next)) => {
            previous.text != next.text
                || previous.size != next.size
                || previous.line_height != next.line_height
                || previous.color != next.color
        }
        (Element::Button(previous), Element::Button(next)) => {
            previous.label != next.label
                || previous.variant != next.variant
                || previous.action != next.action
                || previous.disabled != next.disabled
        }
        (Element::IconButton(previous), Element::IconButton(next)) => {
            previous.icon != next.icon
                || previous.label != next.label
                || previous.action != next.action
                || previous.disabled != next.disabled
        }
        (Element::Toggle(previous), Element::Toggle(next)) => {
            previous.label != next.label
                || previous.checked != next.checked
                || previous.action != next.action
                || previous.disabled != next.disabled
        }
        (Element::Checkbox(previous), Element::Checkbox(next)) => {
            previous.label != next.label
                || previous.checked != next.checked
                || previous.action != next.action
                || previous.disabled != next.disabled
        }
        (Element::Radio(previous), Element::Radio(next)) => {
            previous.label != next.label
                || previous.selected != next.selected
                || previous.action != next.action
                || previous.disabled != next.disabled
        }
        (Element::Slider(previous), Element::Slider(next)) => {
            previous.label != next.label
                || previous.value != next.value
                || previous.min != next.min
                || previous.max != next.max
                || previous.step != next.step
                || previous.action != next.action
                || previous.disabled != next.disabled
        }
        (Element::ProgressBar(previous), Element::ProgressBar(next)) => {
            previous.label != next.label || previous.value != next.value || previous.max != next.max
        }
        (Element::Tabs(previous), Element::Tabs(next)) => {
            previous.selected != next.selected || !same_options(&previous.options, &next.options)
        }
        (Element::SegmentedControl(previous), Element::SegmentedControl(next)) => {
            previous.label != next.label
                || previous.selected != next.selected
                || !same_options(&previous.options, &next.options)
        }
        (Element::Badge(previous), Element::Badge(next)) => {
            previous.label != next.label || previous.color != next.color
        }
        (Element::Avatar(previous), Element::Avatar(next)) => {
            previous.label != next.label || previous.initials != next.initials
        }
        (Element::Card(_), Element::Card(_)) => false,
        (Element::Tooltip(previous), Element::Tooltip(next)) => previous.label != next.label,
        (Element::TextField(previous), Element::TextField(next)) => {
            previous.label != next.label
                || previous.text != next.text
                || previous.placeholder != next.placeholder
                || previous.disabled != next.disabled
        }
        (Element::Stack(previous), Element::Stack(next)) => {
            previous.axis != next.axis
                || previous.padding != next.padding
                || previous.spacing != next.spacing
        }
        (Element::Flex(previous), Element::Flex(next)) => {
            previous.layout != next.layout
                || previous.width != next.width
                || previous.height != next.height
                || previous.children.len() != next.children.len()
                || previous
                    .children
                    .iter()
                    .zip(&next.children)
                    .any(|(previous, next)| {
                        previous.grow != next.grow
                            || previous.shrink != next.shrink
                            || previous.basis != next.basis
                    })
        }
        (Element::Grid(previous), Element::Grid(next)) => {
            previous.layout != next.layout
                || previous.width != next.width
                || previous.height != next.height
                || previous.children.len() != next.children.len()
                || previous
                    .children
                    .iter()
                    .zip(&next.children)
                    .any(|(previous, next)| {
                        previous.column != next.column
                            || previous.row != next.row
                            || previous.column_span != next.column_span
                            || previous.row_span != next.row_span
                    })
        }
        (Element::Overlay(previous), Element::Overlay(next)) => {
            previous.alignment != next.alignment
                || previous.offset_x != next.offset_x
                || previous.offset_y != next.offset_y
        }
        (Element::Surface(previous), Element::Surface(next)) => {
            previous.material != next.material
                || previous.padding != next.padding
                || previous.radius != next.radius
                || previous.border != next.border
                || previous.shadow != next.shadow
                || previous.opacity != next.opacity
                || previous.width != next.width
                || previous.height != next.height
                || previous.margin != next.margin
                || previous.min_width != next.min_width
                || previous.max_width != next.max_width
                || previous.min_height != next.min_height
                || previous.max_height != next.max_height
                || previous.clip != next.clip
        }
        (Element::Media(previous), Element::Media(next)) => previous != next,
        (Element::Frame(previous), Element::Frame(next)) => {
            previous.width != next.width
                || previous.height != next.height
                || previous.margin != next.margin
                || previous.min_width != next.min_width
                || previous.max_width != next.max_width
                || previous.min_height != next.min_height
                || previous.max_height != next.max_height
        }
        (Element::Spacer(previous), Element::Spacer(next)) => {
            previous.width != next.width || previous.height != next.height
        }
        (Element::Divider(previous), Element::Divider(next)) => {
            previous.axis != next.axis
                || previous.thickness != next.thickness
                || previous.color != next.color
        }
        (Element::ScrollView(previous), Element::ScrollView(next)) => {
            previous.width != next.width || previous.height != next.height
        }
        (Element::VirtualList(previous), Element::VirtualList(next)) => {
            previous.row_height != next.row_height
                || previous.viewport_height != next.viewport_height
                || previous.scroll_offset != next.scroll_offset
                || previous.overscan != next.overscan
                || previous.width != next.width
                || !same_row_keys(previous, next)
        }
        (Element::Sidebar(previous), Element::Sidebar(next)) => previous.width != next.width,
        (Element::Toolbar(previous), Element::Toolbar(next)) => previous.title != next.title,
        (Element::SplitView(previous), Element::SplitView(next)) => {
            previous.ratio != next.ratio || previous.resizable != next.resizable
        }
        _ => true,
    }
}

fn reconcile_virtual_list(
    path: Vec<usize>,
    previous: &VirtualListElement,
    next: &VirtualListElement,
    ops: &mut Vec<ReconcileOp>,
) {
    for (previous_index, previous_row) in previous.rows.iter().enumerate() {
        if next.rows.iter().all(|row| row.key != previous_row.key) {
            let mut row_path = path.clone();
            row_path.push(previous_index);
            ops.push(ReconcileOp::Remove {
                path: row_path,
                kind: previous_row.child.kind(),
            });
        }
    }

    for (next_index, next_row) in next.rows.iter().enumerate() {
        let mut row_path = path.clone();
        row_path.push(next_index);
        match previous.rows.iter().find(|row| row.key == next_row.key) {
            Some(previous_row) => reconcile_at(row_path, &previous_row.child, &next_row.child, ops),
            None => ops.push(ReconcileOp::Insert {
                path: row_path,
                kind: next_row.child.kind(),
            }),
        }
    }
}

fn same_row_keys(previous: &VirtualListElement, next: &VirtualListElement) -> bool {
    previous.rows.len() == next.rows.len()
        && previous
            .rows
            .iter()
            .zip(&next.rows)
            .all(|(previous, next)| previous.key == next.key)
}

fn same_options(
    previous: &[crate::ControlOptionElement],
    next: &[crate::ControlOptionElement],
) -> bool {
    previous.len() == next.len()
        && previous.iter().zip(next).all(|(previous, next)| {
            previous.id == next.id
                && previous.label == next.label
                && previous.action == next.action
                && previous.disabled == next.disabled
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TextElement, VirtualListRowElement};
    use stuk_style::{Color, NumberSpacing, TextWrap};

    fn text(value: &str) -> Element {
        Element::Text(TextElement {
            text: value.to_string(),
            size: 14.0,
            line_height: 20.0,
            color: Color::TEXT,
            wrap: TextWrap::Normal,
            number_spacing: NumberSpacing::Proportional,
            align: stuk_style::TextAlign::Start,
        })
    }

    fn row(key: &str, value: &str) -> VirtualListRowElement {
        VirtualListRowElement {
            key: key.to_string(),
            child: text(value),
        }
    }

    #[test]
    fn virtual_list_reconciles_rows_by_key() {
        let mut previous = VirtualListElement::new();
        previous.rows = vec![row("a", "Alpha"), row("b", "Beta")];
        let mut next = VirtualListElement::new();
        next.rows = vec![row("b", "Beta updated"), row("a", "Alpha")];

        let ops = reconcile(&Element::VirtualList(previous), &Element::VirtualList(next));

        assert!(ops.contains(&ReconcileOp::Update {
            path: vec![],
            kind: ElementKind::VirtualList,
        }));
        assert!(ops.contains(&ReconcileOp::Update {
            path: vec![0],
            kind: ElementKind::Text,
        }));
        assert!(
            !ops.iter()
                .any(|op| matches!(op, ReconcileOp::Replace { .. }))
        );
    }
}
