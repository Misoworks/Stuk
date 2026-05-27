use stuk_layout::{FlexLayout, GridLayout, Length, Rect, Size};

use crate::Element;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayAlignment {
    TopStart,
    Top,
    TopEnd,
    CenterStart,
    Center,
    CenterEnd,
    BottomStart,
    Bottom,
    BottomEnd,
}

impl OverlayAlignment {
    pub fn place(self, bounds: Rect, size: Size, offset_x: f32, offset_y: f32) -> Rect {
        let x = match self {
            Self::TopStart | Self::CenterStart | Self::BottomStart => bounds.x,
            Self::Top | Self::Center | Self::Bottom => {
                bounds.x + (bounds.width - size.width).max(0.0) * 0.5
            }
            Self::TopEnd | Self::CenterEnd | Self::BottomEnd => {
                bounds.x + (bounds.width - size.width).max(0.0)
            }
        };
        let y = match self {
            Self::TopStart | Self::Top | Self::TopEnd => bounds.y,
            Self::CenterStart | Self::Center | Self::CenterEnd => {
                bounds.y + (bounds.height - size.height).max(0.0) * 0.5
            }
            Self::BottomStart | Self::Bottom | Self::BottomEnd => {
                bounds.y + (bounds.height - size.height).max(0.0)
            }
        };

        Rect::new(x + offset_x, y + offset_y, size.width, size.height)
    }
}

#[derive(Clone, Debug)]
pub struct OverlayElement {
    pub child: Box<Element>,
    pub overlay: Box<Element>,
    pub alignment: OverlayAlignment,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl OverlayElement {
    pub fn new(child: impl Into<Element>, overlay: impl Into<Element>) -> Self {
        Self {
            child: Box::new(child.into()),
            overlay: Box::new(overlay.into()),
            alignment: OverlayAlignment::Center,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FlexChildElement {
    pub child: Element,
    pub grow: f32,
    pub shrink: f32,
    pub basis: Option<f32>,
}

impl FlexChildElement {
    pub fn new(child: impl Into<Element>) -> Self {
        Self {
            child: child.into(),
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

#[derive(Clone, Debug)]
pub struct FlexElement {
    pub layout: FlexLayout,
    pub children: Vec<FlexChildElement>,
    pub width: Length,
    pub height: Length,
}

impl FlexElement {
    pub fn new(layout: FlexLayout) -> Self {
        Self {
            layout,
            children: Vec::new(),
            width: Length::Fit,
            height: Length::Fit,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GridChildElement {
    pub child: Element,
    pub column: usize,
    pub row: usize,
    pub column_span: usize,
    pub row_span: usize,
}

impl GridChildElement {
    pub fn new(column: usize, row: usize, child: impl Into<Element>) -> Self {
        Self {
            child: child.into(),
            column,
            row,
            column_span: 1,
            row_span: 1,
        }
    }

    pub fn span(mut self, columns: usize, rows: usize) -> Self {
        self.column_span = columns.max(1);
        self.row_span = rows.max(1);
        self
    }
}

#[derive(Clone, Debug)]
pub struct GridElement {
    pub layout: GridLayout,
    pub children: Vec<GridChildElement>,
    pub width: Length,
    pub height: Length,
}

impl GridElement {
    pub fn new(layout: GridLayout) -> Self {
        Self {
            layout,
            children: Vec::new(),
            width: Length::Fit,
            height: Length::Fit,
        }
    }
}
