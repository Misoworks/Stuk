use stuk_core::{Element, VirtualListElement, VirtualListRowElement};
use stuk_layout::Length;

#[derive(Clone, Debug)]
pub struct VirtualList {
    element: VirtualListElement,
}

impl VirtualList {
    pub fn new() -> Self {
        Self {
            element: VirtualListElement::new(),
        }
    }

    pub fn from_items<I, T, K, R, F, E>(items: I, key: K, mut row: F) -> Self
    where
        I: IntoIterator<Item = T>,
        K: Fn(&T) -> R,
        R: ToString,
        F: FnMut(&T) -> E,
        E: Into<Element>,
    {
        let mut list = Self::new();
        for item in items {
            list = list.row(key(&item), row(&item));
        }
        list
    }

    pub fn row(mut self, key: impl ToString, child: impl Into<Element>) -> Self {
        self.element.rows.push(VirtualListRowElement {
            key: key.to_string(),
            child: child.into(),
        });
        self
    }

    pub fn row_height(mut self, row_height: f32) -> Self {
        self.element.row_height = row_height.max(1.0);
        self
    }

    pub fn viewport_height(mut self, height: f32) -> Self {
        self.element.viewport_height = height.max(1.0);
        self
    }

    pub fn scroll_offset(mut self, offset: f32) -> Self {
        self.element.scroll_offset = offset.max(0.0);
        self
    }

    pub fn overscan(mut self, rows: usize) -> Self {
        self.element.overscan = rows;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.element.width = Length::Fixed(width);
        self
    }

    pub fn fill_width(mut self) -> Self {
        self.element.width = Length::Fill;
        self
    }
}

impl Default for VirtualList {
    fn default() -> Self {
        Self::new()
    }
}

impl From<VirtualList> for Element {
    fn from(list: VirtualList) -> Self {
        Element::VirtualList(list.element)
    }
}
