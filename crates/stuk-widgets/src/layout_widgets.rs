use stuk_core::{
    Element, FlexChildElement, FlexElement, GridChildElement, GridElement, OverlayAlignment,
    OverlayElement,
};
use stuk_layout::{
    EdgeInsets, FlexAlign, FlexJustify, FlexLayout, FlexWrap, GridLayout, GridTrack, Length,
};

#[derive(Clone, Debug)]
pub struct Flex {
    element: FlexElement,
}

impl Flex {
    pub fn row() -> Self {
        Self {
            element: FlexElement::new(FlexLayout::row()),
        }
    }

    pub fn column() -> Self {
        Self {
            element: FlexElement::new(FlexLayout::column()),
        }
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.element.layout = self.element.layout.padding(EdgeInsets::all(padding));
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.element.layout = self.element.layout.gap(gap);
        self
    }

    pub fn line_gap(mut self, gap: f32) -> Self {
        self.element.layout = self.element.layout.line_gap(gap);
        self
    }

    pub fn wrap(mut self, wrap: FlexWrap) -> Self {
        self.element.layout = self.element.layout.wrap(wrap);
        self
    }

    pub fn justify(mut self, justify: FlexJustify) -> Self {
        self.element.layout = self.element.layout.justify(justify);
        self
    }

    pub fn align(mut self, align: FlexAlign) -> Self {
        self.element.layout = self.element.layout.align(align);
        self
    }

    pub fn child(mut self, child: impl Into<Element>) -> Self {
        self.element.children.push(FlexChildElement::new(child));
        self
    }

    pub fn flex_child(mut self, child: FlexChildElement) -> Self {
        self.element.children.push(child);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.element.width = Length::Fixed(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.element.height = Length::Fixed(height);
        self
    }

    pub fn fill_width(mut self) -> Self {
        self.element.width = Length::Fill;
        self
    }

    pub fn fill_height(mut self) -> Self {
        self.element.height = Length::Fill;
        self
    }
}

impl From<Flex> for Element {
    fn from(flex: Flex) -> Self {
        Element::Flex(flex.element)
    }
}

#[derive(Clone, Debug)]
pub struct Grid {
    element: GridElement,
}

impl Grid {
    pub fn new(columns: impl Into<Vec<GridTrack>>, rows: impl Into<Vec<GridTrack>>) -> Self {
        Self {
            element: GridElement::new(GridLayout::new(columns, rows)),
        }
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.element.layout = self.element.layout.padding(EdgeInsets::all(padding));
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.element.layout = self.element.layout.gap(gap);
        self
    }

    pub fn column_gap(mut self, gap: f32) -> Self {
        self.element.layout = self.element.layout.column_gap(gap);
        self
    }

    pub fn row_gap(mut self, gap: f32) -> Self {
        self.element.layout = self.element.layout.row_gap(gap);
        self
    }

    pub fn child(mut self, child: GridChildElement) -> Self {
        self.element.children.push(child);
        self
    }

    pub fn cell(mut self, column: usize, row: usize, child: impl Into<Element>) -> Self {
        self.element
            .children
            .push(GridChildElement::new(column, row, child));
        self
    }

    pub fn span(
        mut self,
        column: usize,
        row: usize,
        columns: usize,
        rows: usize,
        child: impl Into<Element>,
    ) -> Self {
        self.element
            .children
            .push(GridChildElement::new(column, row, child).span(columns, rows));
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.element.width = Length::Fixed(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.element.height = Length::Fixed(height);
        self
    }

    pub fn fill_width(mut self) -> Self {
        self.element.width = Length::Fill;
        self
    }

    pub fn fill_height(mut self) -> Self {
        self.element.height = Length::Fill;
        self
    }
}

impl From<Grid> for Element {
    fn from(grid: Grid) -> Self {
        Element::Grid(grid.element)
    }
}

#[derive(Clone, Debug)]
pub struct Overlay {
    element: OverlayElement,
}

impl Overlay {
    pub fn new(child: impl Into<Element>, overlay: impl Into<Element>) -> Self {
        Self {
            element: OverlayElement::new(child, overlay),
        }
    }

    pub fn alignment(mut self, alignment: OverlayAlignment) -> Self {
        self.element.alignment = alignment;
        self
    }

    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.element.offset_x = x;
        self.element.offset_y = y;
        self
    }
}

impl From<Overlay> for Element {
    fn from(overlay: Overlay) -> Self {
        Element::Overlay(overlay.element)
    }
}
