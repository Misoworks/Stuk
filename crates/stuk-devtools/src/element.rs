use stuk_core::{Element, ElementKind, StackElement, measure_element};
use stuk_layout::{
    Axis, EdgeInsets, FlexItem, GridItem, LayoutItem, Rect, Size, flex_layout, grid_layout,
    stack_layout_items,
};

use crate::layout_metrics::{LayoutMetrics, layout_metrics};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementSnapshot {
    pub kind: ElementKind,
    pub label: Option<String>,
    pub action: Option<String>,
    pub children: Vec<ElementSnapshot>,
}

impl ElementSnapshot {
    pub fn descendant_count(&self) -> usize {
        self.children
            .iter()
            .map(|child| 1 + child.descendant_count())
            .sum()
    }

    pub fn action_count(&self) -> usize {
        usize::from(self.action.is_some())
            + self
                .children
                .iter()
                .map(ElementSnapshot::action_count)
                .sum::<usize>()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutSnapshot {
    pub kind: ElementKind,
    pub label: Option<String>,
    pub action: Option<String>,
    pub rect: Rect,
    pub intrinsic_size: Size,
    pub metrics: LayoutMetrics,
    pub children: Vec<LayoutSnapshot>,
}

impl LayoutSnapshot {
    pub fn descendant_count(&self) -> usize {
        self.children
            .iter()
            .map(|child| 1 + child.descendant_count())
            .sum()
    }

    pub fn action_count(&self) -> usize {
        usize::from(self.action.is_some())
            + self
                .children
                .iter()
                .map(LayoutSnapshot::action_count)
                .sum::<usize>()
    }
}

pub fn inspect_element(element: &Element) -> ElementSnapshot {
    ElementSnapshot {
        kind: element.kind(),
        label: element_label(element),
        action: element_action(element),
        children: element_children(element)
            .into_iter()
            .map(inspect_element)
            .collect(),
    }
}

pub fn inspect_layout_for_window(element: &Element) -> LayoutSnapshot {
    let item = measure_element(element);
    inspect_layout(
        element,
        Rect::new(
            0.0,
            0.0,
            item.size.width.max(1.0),
            item.size.height.max(1.0),
        ),
    )
}

pub fn inspect_layout(element: &Element, rect: Rect) -> LayoutSnapshot {
    let intrinsic_size = measure_element(element).size;
    let children = layout_children(element, rect)
        .into_iter()
        .map(|(child, child_rect)| inspect_layout(child, child_rect))
        .collect();

    LayoutSnapshot {
        kind: element.kind(),
        label: element_label(element),
        action: element_action(element),
        rect,
        intrinsic_size,
        metrics: layout_metrics(element, rect, intrinsic_size),
        children,
    }
}

fn layout_children(element: &Element, rect: Rect) -> Vec<(&Element, Rect)> {
    match element {
        Element::Window(window) => window
            .content
            .as_deref()
            .map(|content| vec![(content, rect)])
            .unwrap_or_default(),
        Element::Stack(stack) => stack_child_layout(stack, rect),
        Element::Flex(flex) => {
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
            let boxes = flex_layout(flex.layout, rect, &items);
            flex.children
                .iter()
                .zip(boxes)
                .map(|(child, layout_box)| (&child.child, layout_box.rect))
                .collect()
        }
        Element::Grid(grid) => {
            let items = grid
                .children
                .iter()
                .map(|child| {
                    GridItem::new(child.column, child.row, measure_element(&child.child).size)
                        .span(child.column_span, child.row_span)
                })
                .collect::<Vec<_>>();
            let boxes = grid_layout(&grid.layout, rect, &items);
            grid.children
                .iter()
                .zip(boxes)
                .map(|(child, layout_box)| (&child.child, layout_box.rect))
                .collect()
        }
        Element::Overlay(overlay) => {
            let overlay_size = measure_element(&overlay.overlay).size;
            let overlay_rect =
                overlay
                    .alignment
                    .place(rect, overlay_size, overlay.offset_x, overlay.offset_y);
            vec![
                (overlay.child.as_ref(), rect),
                (overlay.overlay.as_ref(), overlay_rect),
            ]
        }
        Element::Surface(surface) => vec![(surface.child.as_ref(), surface.inner_bounds(rect))],
        Element::Frame(frame) => vec![(frame.child.as_ref(), frame.child_bounds(rect))],
        Element::ScrollView(scroll_view) => vec![(scroll_view.child.as_ref(), rect)],
        Element::VirtualList(list) => list
            .visible_range()
            .map(|index| (&list.rows[index].child, list.row_rect(rect, index)))
            .collect(),
        Element::Sidebar(sidebar) => stack_children_layout(
            &sidebar.children,
            Axis::Vertical,
            EdgeInsets::all(14.0),
            8.0,
            rect,
        ),
        Element::Toolbar(toolbar) => {
            let action_rect = Rect::new(
                rect.x + rect.width * 0.48,
                rect.y + 4.0,
                rect.width * 0.52,
                36.0,
            );
            stack_children_layout(
                &toolbar.children,
                Axis::Horizontal,
                EdgeInsets::default(),
                8.0,
                action_rect,
            )
        }
        Element::SplitView(split_view) => {
            let sidebar_width = (rect.width * split_view.ratio).clamp(160.0, rect.width * 0.5);
            let sidebar = Rect::new(rect.x, rect.y, sidebar_width, rect.height);
            let main = Rect::new(
                rect.x + sidebar_width + 14.0,
                rect.y,
                (rect.width - sidebar_width - 14.0).max(1.0),
                rect.height,
            );
            vec![
                (split_view.sidebar.as_ref(), sidebar),
                (split_view.main.as_ref(), main),
            ]
        }
        _ => Vec::new(),
    }
}

fn stack_child_layout(stack: &StackElement, rect: Rect) -> Vec<(&Element, Rect)> {
    stack_children_layout(
        &stack.children,
        stack.axis,
        stack.padding,
        stack.spacing,
        rect,
    )
}

fn stack_children_layout(
    children: &[Element],
    axis: Axis,
    padding: EdgeInsets,
    spacing: f32,
    rect: Rect,
) -> Vec<(&Element, Rect)> {
    let items = children
        .iter()
        .map(measure_element)
        .collect::<Vec<LayoutItem>>();
    let boxes = stack_layout_items(axis, rect, padding, spacing, &items);

    children
        .iter()
        .zip(boxes)
        .map(|(child, layout_box)| (child, layout_box.rect))
        .collect()
}

fn element_label(element: &Element) -> Option<String> {
    match element {
        Element::Window(window) => Some(window.title.clone()),
        Element::Text(text) => Some(text.text.clone()),
        Element::Media(media) => media.label.clone().or_else(|| Some(media.id.clone())),
        Element::Button(button) => Some(button.label.clone()),
        Element::IconButton(button) => Some(button.label.clone()),
        Element::Toggle(toggle) => Some(toggle.label.clone()),
        Element::TextField(field) => field.label.clone(),
        Element::Toolbar(toolbar) => Some(toolbar.title.clone()),
        Element::VirtualList(list) => Some(format!("{} rows", list.rows.len())),
        Element::Flex(flex) => Some(format!("{} children", flex.children.len())),
        Element::Grid(grid) => Some(format!("{} children", grid.children.len())),
        Element::Overlay(_) => Some("overlay".to_string()),
        Element::Surface(_) => Some("surface".to_string()),
        _ => None,
    }
}

fn element_action(element: &Element) -> Option<String> {
    match element {
        Element::Button(button) => button.action.clone(),
        Element::IconButton(button) => button.action.clone(),
        Element::Toggle(toggle) => toggle.action.clone(),
        _ => None,
    }
}

fn element_children(element: &Element) -> Vec<&Element> {
    match element {
        Element::Window(window) => window.content.iter().map(Box::as_ref).collect(),
        Element::Stack(stack) => stack.children.iter().collect(),
        Element::Flex(flex) => flex.children.iter().map(|child| &child.child).collect(),
        Element::Grid(grid) => grid.children.iter().map(|child| &child.child).collect(),
        Element::Overlay(overlay) => vec![overlay.child.as_ref(), overlay.overlay.as_ref()],
        Element::Surface(surface) => vec![surface.child.as_ref()],
        Element::Frame(frame) => vec![frame.child.as_ref()],
        Element::ScrollView(scroll_view) => vec![scroll_view.child.as_ref()],
        Element::VirtualList(list) => list
            .visible_range()
            .map(|index| &list.rows[index].child)
            .collect(),
        Element::Sidebar(sidebar) => sidebar.children.iter().collect(),
        Element::Toolbar(toolbar) => toolbar.children.iter().collect(),
        Element::SplitView(split_view) => {
            vec![split_view.sidebar.as_ref(), split_view.main.as_ref()]
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stuk_core::ElementKind;
    use stuk_layout::{EdgeInsets, GridTrack, Length};
    use stuk_widgets::{Button, Flex, Frame, Grid, Surface, Text, VStack};

    #[test]
    fn captures_component_tree_labels_and_actions() {
        let element: Element = VStack::new()
            .child(Text::title("Draft"))
            .child(Button::primary("Save").action("document.save"))
            .into();

        let snapshot = inspect_element(&element);

        assert_eq!(snapshot.kind, ElementKind::VStack);
        assert_eq!(snapshot.descendant_count(), 2);
        assert_eq!(snapshot.action_count(), 1);
        assert_eq!(snapshot.children[0].label.as_deref(), Some("Draft"));
        assert_eq!(
            snapshot.children[1].action.as_deref(),
            Some("document.save")
        );
    }

    #[test]
    fn captures_layout_boxes_for_stack_children() {
        let element: Element = VStack::new()
            .padding(10.0)
            .spacing(6.0)
            .child(Text::new("Title"))
            .child(Button::primary("Save").action("document.save"))
            .into();

        let snapshot = inspect_layout(&element, Rect::new(0.0, 0.0, 320.0, 180.0));

        assert_eq!(snapshot.kind, ElementKind::VStack);
        assert_eq!(snapshot.children.len(), 2);
        assert_eq!(snapshot.children[0].rect.x, 10.0);
        assert_eq!(snapshot.children[0].rect.y, 10.0);
        assert_eq!(snapshot.children[1].rect.y, 36.0);
        assert_eq!(snapshot.action_count(), 1);
        assert_eq!(snapshot.metrics.padding, EdgeInsets::all(10.0));
        assert_eq!(snapshot.metrics.vertical_gap, Some(6.0));
    }

    #[test]
    fn captures_layout_boxes_for_flex_and_grid_children() {
        let flex: Element = Flex::row()
            .gap(10.0)
            .child(Text::new("A"))
            .child(Button::primary("Save").action("document.save"))
            .into();
        let flex = inspect_layout(&flex, Rect::new(0.0, 0.0, 240.0, 80.0));

        assert_eq!(flex.kind, ElementKind::Flex);
        assert_eq!(flex.children.len(), 2);
        assert_eq!(flex.children[1].rect.x, flex.children[0].rect.width + 10.0);

        let grid: Element = Grid::new(
            vec![GridTrack::fixed(100.0), GridTrack::fraction(1.0)],
            vec![GridTrack::fixed(30.0)],
        )
        .gap(8.0)
        .cell(0, 0, Text::new("Name"))
        .cell(1, 0, Button::secondary("Edit").action("profile.edit"))
        .into();
        let grid = inspect_layout(&grid, Rect::new(0.0, 0.0, 260.0, 80.0));

        assert_eq!(grid.kind, ElementKind::Grid);
        assert_eq!(grid.children.len(), 2);
        assert_eq!(grid.children[0].rect.width, 100.0);
        assert_eq!(grid.children[1].rect.x, 108.0);
        assert_eq!(grid.metrics.horizontal_gap, Some(8.0));
        assert_eq!(grid.metrics.vertical_gap, Some(8.0));
    }

    #[test]
    fn captures_layout_metrics_for_margins_constraints_and_overflow() {
        let element: Element = Surface::new(
            Frame::new(Text::new("A wide line"))
                .width(180.0)
                .margin(5.0),
        )
        .padding(12.0)
        .margin(4.0)
        .min_width(200.0)
        .max_width(260.0)
        .into();

        let snapshot = inspect_layout(&element, Rect::new(0.0, 0.0, 100.0, 48.0));

        assert_eq!(snapshot.metrics.padding, EdgeInsets::all(12.0));
        assert_eq!(snapshot.metrics.margin, EdgeInsets::all(4.0));
        assert_eq!(snapshot.metrics.min_width, Some(200.0));
        assert_eq!(snapshot.metrics.max_width, Some(260.0));
        assert!(snapshot.metrics.overflow.unwrap().width > 0.0);

        let frame = &snapshot.children[0];
        assert_eq!(frame.metrics.margin, EdgeInsets::all(5.0));
        assert_eq!(frame.metrics.width, Some(Length::Fixed(180.0)));
        assert!(frame.metrics.overflow.unwrap().width > 0.0);
    }
}
