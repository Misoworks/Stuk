use crate::{Axis, EdgeInsets, LayoutBox, Rect, Size};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexJustify {
    Start,
    Center,
    End,
    SpaceBetween,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexAlign {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexItem {
    pub size: Size,
    pub grow: f32,
    pub shrink: f32,
    pub basis: Option<f32>,
}

impl FlexItem {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            grow: 0.0,
            shrink: 1.0,
            basis: None,
        }
    }

    pub fn grow(mut self, grow: f32) -> Self {
        self.grow = grow.max(0.0);
        self
    }

    pub fn shrink(mut self, shrink: f32) -> Self {
        self.shrink = shrink.max(0.0);
        self
    }

    pub fn basis(mut self, basis: f32) -> Self {
        self.basis = Some(basis.max(0.0));
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexLayout {
    pub axis: Axis,
    pub padding: EdgeInsets,
    pub gap: f32,
    pub line_gap: f32,
    pub wrap: FlexWrap,
    pub justify: FlexJustify,
    pub align: FlexAlign,
}

impl FlexLayout {
    pub fn row() -> Self {
        Self {
            axis: Axis::Horizontal,
            padding: EdgeInsets::default(),
            gap: 0.0,
            line_gap: 0.0,
            wrap: FlexWrap::NoWrap,
            justify: FlexJustify::Start,
            align: FlexAlign::Start,
        }
    }

    pub fn column() -> Self {
        Self {
            axis: Axis::Vertical,
            ..Self::row()
        }
    }

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }

    pub fn line_gap(mut self, line_gap: f32) -> Self {
        self.line_gap = line_gap.max(0.0);
        self
    }

    pub fn wrap(mut self, wrap: FlexWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn justify(mut self, justify: FlexJustify) -> Self {
        self.justify = justify;
        self
    }

    pub fn align(mut self, align: FlexAlign) -> Self {
        self.align = align;
        self
    }
}

impl Default for FlexLayout {
    fn default() -> Self {
        Self::row()
    }
}

pub fn flex_layout(layout: FlexLayout, bounds: Rect, items: &[FlexItem]) -> Vec<LayoutBox> {
    if items.is_empty() {
        return Vec::new();
    }

    let axis = flex_axis(layout.axis);
    let content = bounds.inset(layout.padding);
    let capacity = primary(content.size(), axis);
    let lines = flex_lines(items, axis, capacity, layout.gap, layout.wrap);
    let mut boxes = Vec::with_capacity(items.len());
    let mut cross_cursor = cross_origin(content, axis);

    for line in lines {
        let count = line.end - line.start;
        let line_cross = if layout.align == FlexAlign::Stretch && layout.wrap == FlexWrap::NoWrap {
            cross(content.size(), axis)
        } else {
            line.cross
        };
        let sizes = line_item_sizes(&line, items, axis, capacity, layout.gap);
        let occupied = sizes.iter().map(|size| size.primary).sum::<f32>()
            + layout.gap * count.saturating_sub(1) as f32;
        let free = (capacity - occupied).max(0.0);
        let start = justify_offset(layout.justify, free, count);
        let gap = if layout.justify == FlexJustify::SpaceBetween && count > 1 {
            layout.gap + free / (count - 1) as f32
        } else {
            layout.gap
        };
        let mut primary_cursor = primary_origin(content, axis) + start;

        for (index, size) in sizes.iter().enumerate() {
            let item = items[line.start + index];
            let item_cross = aligned_cross_size(layout.align, size.cross, line_cross);
            let cross_offset = align_offset(layout.align, line_cross, item_cross);
            boxes.push(LayoutBox {
                rect: rect_for_axes(
                    axis,
                    primary_cursor,
                    cross_cursor + cross_offset,
                    size.primary,
                    item_cross_for(layout.align, item, axis, item_cross),
                ),
            });
            primary_cursor += size.primary + gap;
        }

        cross_cursor += line_cross + layout.line_gap;
    }

    boxes
}

fn flex_axis(axis: Axis) -> Axis {
    if axis == Axis::Vertical {
        Axis::Vertical
    } else {
        Axis::Horizontal
    }
}

#[derive(Clone, Copy)]
struct FlexLine {
    start: usize,
    end: usize,
    base: f32,
    cross: f32,
    grow: f32,
    shrink: f32,
}

#[derive(Clone, Copy)]
struct ItemSize {
    primary: f32,
    cross: f32,
}

fn flex_lines(
    items: &[FlexItem],
    axis: Axis,
    capacity: f32,
    gap: f32,
    wrap: FlexWrap,
) -> Vec<FlexLine> {
    let mut lines = Vec::new();
    let mut line = empty_line(0);

    for (index, item) in items.iter().enumerate() {
        let base = item_base(*item, axis);
        let candidate = line.base + if line.end > line.start { gap } else { 0.0 } + base;
        if wrap == FlexWrap::Wrap && line.end > line.start && candidate > capacity {
            lines.push(line);
            line = empty_line(index);
        }

        line.end = index + 1;
        line.base += if line.end - line.start > 1 {
            gap + base
        } else {
            base
        };
        line.cross = line.cross.max(cross(item.size, axis));
        line.grow += item.grow.max(0.0);
        line.shrink += item.shrink.max(0.0) * base;
    }

    lines.push(line);
    lines
}

fn empty_line(start: usize) -> FlexLine {
    FlexLine {
        start,
        end: start,
        base: 0.0,
        cross: 0.0,
        grow: 0.0,
        shrink: 0.0,
    }
}

fn line_item_sizes(
    line: &FlexLine,
    items: &[FlexItem],
    axis: Axis,
    capacity: f32,
    gap: f32,
) -> Vec<ItemSize> {
    let count = line.end - line.start;
    let gaps = gap * count.saturating_sub(1) as f32;
    let base_without_gaps = line.base - gaps;
    let free = capacity - line.base;

    items[line.start..line.end]
        .iter()
        .map(|item| {
            let base = item_base(*item, axis);
            let primary = if free > 0.0 && line.grow > 0.0 {
                base + free * item.grow.max(0.0) / line.grow
            } else if free < 0.0 && line.shrink > 0.0 {
                let weighted = item.shrink.max(0.0) * base;
                (base + free * weighted / line.shrink).max(0.0)
            } else if free < 0.0 && base_without_gaps > 0.0 {
                (base + free * base / base_without_gaps).max(0.0)
            } else {
                base
            };
            ItemSize {
                primary,
                cross: cross(item.size, axis),
            }
        })
        .collect()
}

fn item_base(item: FlexItem, axis: Axis) -> f32 {
    item.basis
        .unwrap_or_else(|| primary(item.size, axis))
        .max(0.0)
}

fn primary(size: Size, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => size.width,
        Axis::Vertical => size.height,
        Axis::Depth => size.width,
    }
}

fn cross(size: Size, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => size.height,
        Axis::Vertical => size.width,
        Axis::Depth => size.height,
    }
}

fn primary_origin(rect: Rect, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => rect.x,
        Axis::Vertical => rect.y,
        Axis::Depth => rect.x,
    }
}

fn cross_origin(rect: Rect, axis: Axis) -> f32 {
    match axis {
        Axis::Horizontal => rect.y,
        Axis::Vertical => rect.x,
        Axis::Depth => rect.y,
    }
}

fn justify_offset(justify: FlexJustify, free: f32, count: usize) -> f32 {
    match justify {
        FlexJustify::Start | FlexJustify::SpaceBetween => 0.0,
        FlexJustify::Center => free * 0.5,
        FlexJustify::End => free,
    }
    .max(0.0)
    .min(if count == 0 { 0.0 } else { free })
}

fn aligned_cross_size(align: FlexAlign, cross: f32, line_cross: f32) -> f32 {
    if align == FlexAlign::Stretch {
        line_cross
    } else {
        cross
    }
}

fn align_offset(align: FlexAlign, line_cross: f32, cross: f32) -> f32 {
    match align {
        FlexAlign::Start | FlexAlign::Stretch => 0.0,
        FlexAlign::Center => (line_cross - cross).max(0.0) * 0.5,
        FlexAlign::End => (line_cross - cross).max(0.0),
    }
}

fn item_cross_for(align: FlexAlign, item: FlexItem, axis: Axis, resolved: f32) -> f32 {
    if align == FlexAlign::Stretch {
        resolved
    } else {
        cross(item.size, axis)
    }
}

fn rect_for_axes(axis: Axis, primary: f32, cross: f32, primary_size: f32, cross_size: f32) -> Rect {
    match axis {
        Axis::Horizontal => Rect::new(primary, cross, primary_size, cross_size),
        Axis::Vertical => Rect::new(cross, primary, cross_size, primary_size),
        Axis::Depth => Rect::new(primary, cross, primary_size, cross_size),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flex_distributes_grow_space_on_primary_axis() {
        let layout = FlexLayout::row().gap(10.0);
        let boxes = flex_layout(
            layout,
            Rect::new(0.0, 0.0, 220.0, 40.0),
            &[
                FlexItem::new(Size::new(50.0, 20.0)).grow(1.0),
                FlexItem::new(Size::new(50.0, 20.0)).grow(2.0),
            ],
        );

        assert_eq!(boxes[0].rect, Rect::new(0.0, 0.0, 86.66667, 20.0));
        assert_eq!(boxes[1].rect.x, 96.66667);
        assert_eq!(boxes[1].rect.width, 123.333336);
    }

    #[test]
    fn flex_wraps_lines_and_aligns_cross_axis() {
        let layout = FlexLayout::row()
            .gap(8.0)
            .line_gap(6.0)
            .wrap(FlexWrap::Wrap)
            .align(FlexAlign::End);
        let boxes = flex_layout(
            layout,
            Rect::new(0.0, 0.0, 130.0, 100.0),
            &[
                FlexItem::new(Size::new(80.0, 20.0)),
                FlexItem::new(Size::new(80.0, 30.0)),
                FlexItem::new(Size::new(40.0, 10.0)),
            ],
        );

        assert_eq!(boxes[0].rect, Rect::new(0.0, 0.0, 80.0, 20.0));
        assert_eq!(boxes[1].rect, Rect::new(0.0, 26.0, 80.0, 30.0));
        assert_eq!(boxes[2].rect, Rect::new(88.0, 46.0, 40.0, 10.0));
    }

    #[test]
    fn flex_column_can_stretch_cross_axis() {
        let layout = FlexLayout::column().align(FlexAlign::Stretch);
        let boxes = flex_layout(
            layout,
            Rect::new(10.0, 20.0, 160.0, 100.0),
            &[FlexItem::new(Size::new(40.0, 20.0))],
        );

        assert_eq!(boxes[0].rect, Rect::new(10.0, 20.0, 160.0, 20.0));
    }
}
