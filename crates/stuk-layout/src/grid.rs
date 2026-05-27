use crate::{EdgeInsets, LayoutBox, Rect, Size};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GridTrack {
    Fixed(f32),
    Fraction(f32),
    Fit,
}

impl GridTrack {
    pub const fn fixed(value: f32) -> Self {
        Self::Fixed(value)
    }

    pub const fn fraction(value: f32) -> Self {
        Self::Fraction(value)
    }

    pub const fn fit() -> Self {
        Self::Fit
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridLayout {
    pub columns: Vec<GridTrack>,
    pub rows: Vec<GridTrack>,
    pub padding: EdgeInsets,
    pub column_gap: f32,
    pub row_gap: f32,
}

impl GridLayout {
    pub fn new(columns: impl Into<Vec<GridTrack>>, rows: impl Into<Vec<GridTrack>>) -> Self {
        Self {
            columns: columns.into(),
            rows: rows.into(),
            padding: EdgeInsets::default(),
            column_gap: 0.0,
            row_gap: 0.0,
        }
    }

    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        let gap = gap.max(0.0);
        self.column_gap = gap;
        self.row_gap = gap;
        self
    }

    pub fn column_gap(mut self, gap: f32) -> Self {
        self.column_gap = gap.max(0.0);
        self
    }

    pub fn row_gap(mut self, gap: f32) -> Self {
        self.row_gap = gap.max(0.0);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridItem {
    pub column: usize,
    pub row: usize,
    pub column_span: usize,
    pub row_span: usize,
    pub size: Size,
}

impl GridItem {
    pub fn new(column: usize, row: usize, size: Size) -> Self {
        Self {
            column,
            row,
            column_span: 1,
            row_span: 1,
            size,
        }
    }

    pub fn span(mut self, columns: usize, rows: usize) -> Self {
        self.column_span = columns.max(1);
        self.row_span = rows.max(1);
        self
    }
}

pub fn grid_layout(layout: &GridLayout, bounds: Rect, items: &[GridItem]) -> Vec<LayoutBox> {
    let content = bounds.inset(layout.padding);
    if layout.columns.is_empty() || layout.rows.is_empty() {
        return items.iter().map(|_| zero_box(content)).collect();
    }

    let columns = resolve_tracks(
        &layout.columns,
        content.width,
        layout.column_gap,
        items,
        TrackAxis::Column,
    );
    let rows = resolve_tracks(
        &layout.rows,
        content.height,
        layout.row_gap,
        items,
        TrackAxis::Row,
    );
    let column_offsets = track_offsets(content.x, &columns, layout.column_gap);
    let row_offsets = track_offsets(content.y, &rows, layout.row_gap);

    items
        .iter()
        .map(|item| {
            if item.column >= columns.len() || item.row >= rows.len() {
                return zero_box(content);
            }

            let column_span = item.column_span.max(1).min(columns.len() - item.column);
            let row_span = item.row_span.max(1).min(rows.len() - item.row);
            LayoutBox {
                rect: Rect::new(
                    column_offsets[item.column],
                    row_offsets[item.row],
                    span_size(&columns, item.column, column_span, layout.column_gap),
                    span_size(&rows, item.row, row_span, layout.row_gap),
                ),
            }
        })
        .collect()
}

#[derive(Clone, Copy)]
enum TrackAxis {
    Column,
    Row,
}

fn resolve_tracks(
    tracks: &[GridTrack],
    available: f32,
    gap: f32,
    items: &[GridItem],
    axis: TrackAxis,
) -> Vec<f32> {
    let mut sizes = tracks
        .iter()
        .map(|track| match track {
            GridTrack::Fixed(value) => value.max(0.0),
            GridTrack::Fraction(_) | GridTrack::Fit => 0.0,
        })
        .collect::<Vec<_>>();

    for item in items {
        let (index, span, intrinsic) = match axis {
            TrackAxis::Column => (item.column, item.column_span, item.size.width),
            TrackAxis::Row => (item.row, item.row_span, item.size.height),
        };
        if span.max(1) == 1 && tracks.get(index) == Some(&GridTrack::Fit) {
            sizes[index] = sizes[index].max(intrinsic.max(0.0));
        }
    }

    let gap_total = gap * tracks.len().saturating_sub(1) as f32;
    let used = sizes.iter().sum::<f32>() + gap_total;
    let fraction_sum = tracks
        .iter()
        .map(|track| match track {
            GridTrack::Fraction(value) => value.max(0.0),
            GridTrack::Fixed(_) | GridTrack::Fit => 0.0,
        })
        .sum::<f32>();
    let remaining = (available - used).max(0.0);

    for (index, track) in tracks.iter().enumerate() {
        if let GridTrack::Fraction(value) = track {
            sizes[index] = if fraction_sum > 0.0 {
                remaining * value.max(0.0) / fraction_sum
            } else {
                0.0
            };
        }
    }

    sizes
}

fn track_offsets(start: f32, sizes: &[f32], gap: f32) -> Vec<f32> {
    let mut cursor = start;
    sizes
        .iter()
        .map(|size| {
            let offset = cursor;
            cursor += size + gap;
            offset
        })
        .collect()
}

fn span_size(sizes: &[f32], start: usize, span: usize, gap: f32) -> f32 {
    sizes[start..start + span].iter().sum::<f32>() + gap * span.saturating_sub(1) as f32
}

fn zero_box(content: Rect) -> LayoutBox {
    LayoutBox {
        rect: Rect::new(content.x, content.y, 0.0, 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_resolves_fixed_fit_and_fraction_tracks() {
        let layout = GridLayout::new(
            vec![
                GridTrack::fixed(80.0),
                GridTrack::fit(),
                GridTrack::fraction(1.0),
            ],
            vec![GridTrack::fit(), GridTrack::fraction(1.0)],
        )
        .gap(10.0);
        let boxes = grid_layout(
            &layout,
            Rect::new(0.0, 0.0, 300.0, 180.0),
            &[
                GridItem::new(1, 0, Size::new(70.0, 24.0)),
                GridItem::new(2, 1, Size::new(40.0, 40.0)),
            ],
        );

        assert_eq!(boxes[0].rect, Rect::new(90.0, 0.0, 70.0, 24.0));
        assert_eq!(boxes[1].rect, Rect::new(170.0, 34.0, 130.0, 146.0));
    }

    #[test]
    fn grid_items_can_span_tracks_and_padding() {
        let layout = GridLayout::new(
            vec![GridTrack::fixed(50.0), GridTrack::fixed(60.0)],
            vec![GridTrack::fixed(20.0), GridTrack::fixed(30.0)],
        )
        .padding(EdgeInsets::all(5.0))
        .gap(4.0);
        let boxes = grid_layout(
            &layout,
            Rect::new(0.0, 0.0, 200.0, 100.0),
            &[GridItem::new(0, 0, Size::new(0.0, 0.0)).span(2, 2)],
        );

        assert_eq!(boxes[0].rect, Rect::new(5.0, 5.0, 114.0, 54.0));
    }

    #[test]
    fn grid_preserves_output_for_invalid_items_with_zero_boxes() {
        let layout = GridLayout::new(vec![GridTrack::fixed(50.0)], vec![GridTrack::fixed(20.0)]);
        let boxes = grid_layout(
            &layout,
            Rect::new(10.0, 20.0, 100.0, 80.0),
            &[GridItem::new(2, 0, Size::new(10.0, 10.0))],
        );

        assert_eq!(boxes[0].rect, Rect::new(10.0, 20.0, 0.0, 0.0));
    }
}
