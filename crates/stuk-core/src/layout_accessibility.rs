use stuk_accessibility::{AccessibilityTreeBuilder, NodeId, Role};
use stuk_layout::{FlexItem, GridItem, Rect, flex_layout, grid_layout};

use crate::accessibility::{element_child, node};
use crate::layout_elements::{FlexElement, GridElement, OverlayElement};
use crate::measure::measure_element;

pub(crate) fn flex_node(
    flex: &FlexElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> NodeId {
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
    let children: Vec<NodeId> = flex
        .children
        .iter()
        .zip(flex_layout(flex.layout, bounds, &items))
        .filter_map(|(child, child_box)| element_child(&child.child, child_box.rect, builder))
        .collect();

    let mut node = node(Role::Group, bounds);
    node.set_children(children);
    builder.push(node)
}

pub(crate) fn grid_node(
    grid: &GridElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> NodeId {
    let items = grid
        .children
        .iter()
        .map(|child| {
            GridItem::new(child.column, child.row, measure_element(&child.child).size)
                .span(child.column_span, child.row_span)
        })
        .collect::<Vec<_>>();
    let children: Vec<NodeId> = grid
        .children
        .iter()
        .zip(grid_layout(&grid.layout, bounds, &items))
        .filter_map(|(child, child_box)| element_child(&child.child, child_box.rect, builder))
        .collect();

    let mut node = node(Role::Grid, bounds);
    node.set_children(children);
    builder.push(node)
}

pub(crate) fn overlay_node(
    overlay: &OverlayElement,
    bounds: Rect,
    builder: &mut AccessibilityTreeBuilder,
) -> NodeId {
    let overlay_size = measure_element(&overlay.overlay).size;
    let overlay_bounds =
        overlay
            .alignment
            .place(bounds, overlay_size, overlay.offset_x, overlay.offset_y);
    let children = [
        element_child(&overlay.child, bounds, builder),
        element_child(&overlay.overlay, overlay_bounds, builder),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let mut node = node(Role::Group, bounds);
    node.set_children(children);
    builder.push(node)
}
