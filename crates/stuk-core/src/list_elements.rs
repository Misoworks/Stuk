use std::ops::Range;

use stuk_layout::{Length, Rect};

use crate::Element;

#[derive(Clone, Debug)]
pub struct VirtualListRowElement {
    pub key: String,
    pub child: Element,
}

#[derive(Clone, Debug)]
pub struct VirtualListElement {
    pub rows: Vec<VirtualListRowElement>,
    pub row_height: f32,
    pub viewport_height: f32,
    pub scroll_offset: f32,
    pub overscan: usize,
    pub width: Length,
}

impl VirtualListElement {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            row_height: 44.0,
            viewport_height: 240.0,
            scroll_offset: 0.0,
            overscan: 2,
            width: Length::Fill,
        }
    }

    pub fn total_height(&self) -> f32 {
        self.row_height.max(1.0) * self.rows.len() as f32
    }

    pub fn visible_range(&self) -> Range<usize> {
        if self.rows.is_empty() {
            return 0..0;
        }

        let row_height = self.row_height.max(1.0);
        let start = (self.scroll_offset.max(0.0) / row_height).floor() as usize;
        let start = start.saturating_sub(self.overscan);
        let visible = (self.viewport_height.max(1.0) / row_height).ceil() as usize;
        let count = visible.saturating_add(self.overscan.saturating_mul(2).saturating_add(1));
        let end = start.saturating_add(count).min(self.rows.len());
        start..end
    }

    pub fn row_rect(&self, bounds: Rect, index: usize) -> Rect {
        let row_height = self.row_height.max(1.0);
        Rect::new(
            bounds.x,
            bounds.y + index as f32 * row_height - self.scroll_offset.max(0.0),
            bounds.width,
            row_height,
        )
    }
}

impl Default for VirtualListElement {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_range_accounts_for_scroll_and_overscan() {
        let mut list = VirtualListElement::new();
        list.row_height = 20.0;
        list.viewport_height = 60.0;
        list.scroll_offset = 100.0;
        list.overscan = 1;
        list.rows = (0..100)
            .map(|index| VirtualListRowElement {
                key: format!("row-{index}"),
                child: Element::Empty,
            })
            .collect();

        assert_eq!(list.visible_range(), 4..10);
        assert_eq!(
            list.row_rect(Rect::new(10.0, 20.0, 300.0, 60.0), 5),
            Rect::new(10.0, 20.0, 300.0, 20.0)
        );
    }
}
