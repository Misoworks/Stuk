mod flex;
mod grid;
mod responsive;

pub use flex::{FlexAlign, FlexItem, FlexJustify, FlexLayout, FlexWrap, flex_layout};
pub use grid::{GridItem, GridLayout, GridTrack, grid_layout};
pub use responsive::{Breakpoint, Responsive};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn clamp_non_negative(self) -> Self {
        Self {
            width: self.width.max(0.0),
            height: self.height.max(0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn size(self) -> Size {
        Size::new(self.width, self.height)
    }

    pub fn inset(self, edges: EdgeInsets) -> Self {
        Self {
            x: self.x + edges.left,
            y: self.y + edges.top,
            width: (self.width - edges.left - edges.right).max(0.0),
            height: (self.height - edges.top - edges.bottom).max(0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
    Depth,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Length {
    #[default]
    Fit,
    Fixed(f32),
    Fill,
}

impl Length {
    pub const fn fixed(value: f32) -> Self {
        Self::Fixed(value)
    }

    pub const fn fill() -> Self {
        Self::Fill
    }

    pub const fn fit() -> Self {
        Self::Fit
    }

    pub fn is_fill(self) -> bool {
        matches!(self, Self::Fill)
    }

    pub fn resolve(self, intrinsic: f32) -> f32 {
        match self {
            Self::Fit => intrinsic,
            Self::Fixed(value) => value.max(0.0),
            Self::Fill => intrinsic,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutBox {
    pub rect: Rect,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutItem {
    pub size: Size,
    pub width: Length,
    pub height: Length,
}

impl LayoutItem {
    pub fn fit(size: Size) -> Self {
        Self {
            size,
            width: Length::Fit,
            height: Length::Fit,
        }
    }

    pub fn fixed(size: Size) -> Self {
        Self {
            size,
            width: Length::Fixed(size.width),
            height: Length::Fixed(size.height),
        }
    }

    pub fn with_width(mut self, width: Length) -> Self {
        self.width = width;
        self.size.width = width.resolve(self.size.width);
        self
    }

    pub fn with_height(mut self, height: Length) -> Self {
        self.height = height;
        self.size.height = height.resolve(self.size.height);
        self
    }

    pub fn primary_length(self, axis: Axis) -> Length {
        match axis {
            Axis::Horizontal => self.width,
            Axis::Vertical => self.height,
            Axis::Depth => Length::Fit,
        }
    }
}

pub fn stack_size(axis: Axis, padding: EdgeInsets, spacing: f32, children: &[Size]) -> Size {
    let items = children
        .iter()
        .copied()
        .map(LayoutItem::fit)
        .collect::<Vec<_>>();
    stack_size_items(axis, padding, spacing, &items)
}

pub fn stack_size_items(
    axis: Axis,
    padding: EdgeInsets,
    spacing: f32,
    children: &[LayoutItem],
) -> Size {
    let spacing_total = if children.len() > 1 {
        spacing * (children.len() - 1) as f32
    } else {
        0.0
    };

    match axis {
        Axis::Vertical => {
            let width = children
                .iter()
                .fold(0.0_f32, |width, child| width.max(child.size.width));
            let height =
                children.iter().map(|child| child.size.height).sum::<f32>() + spacing_total;
            Size::new(width + padding.horizontal(), height + padding.vertical())
        }
        Axis::Horizontal => {
            let width = children.iter().map(|child| child.size.width).sum::<f32>() + spacing_total;
            let height = children
                .iter()
                .fold(0.0_f32, |height, child| height.max(child.size.height));
            Size::new(width + padding.horizontal(), height + padding.vertical())
        }
        Axis::Depth => {
            let width = children
                .iter()
                .fold(0.0_f32, |width, child| width.max(child.size.width));
            let height = children
                .iter()
                .fold(0.0_f32, |height, child| height.max(child.size.height));
            Size::new(width + padding.horizontal(), height + padding.vertical())
        }
    }
}

pub fn stack_layout(
    axis: Axis,
    bounds: Rect,
    padding: EdgeInsets,
    spacing: f32,
    children: &[Size],
) -> Vec<LayoutBox> {
    let items = children
        .iter()
        .copied()
        .map(LayoutItem::fit)
        .collect::<Vec<_>>();
    stack_layout_items(axis, bounds, padding, spacing, &items)
}

pub fn stack_layout_items(
    axis: Axis,
    bounds: Rect,
    padding: EdgeInsets,
    spacing: f32,
    children: &[LayoutItem],
) -> Vec<LayoutBox> {
    let content = bounds.inset(padding);
    let mut cursor = Point::new(content.x, content.y);

    if axis == Axis::Depth {
        return children
            .iter()
            .map(|item| LayoutBox {
                rect: Rect::new(
                    content.x,
                    content.y,
                    resolve_cross(item.width, item.size.width, content.width),
                    resolve_cross(item.height, item.size.height, content.height),
                ),
            })
            .collect();
    }

    let primary_capacity = match axis {
        Axis::Horizontal => content.width,
        Axis::Vertical => content.height,
        Axis::Depth => 0.0,
    };
    let spacing_total = if children.len() > 1 {
        spacing * (children.len() - 1) as f32
    } else {
        0.0
    };
    let fixed_primary = children
        .iter()
        .filter(|item| !item.primary_length(axis).is_fill())
        .map(|item| match axis {
            Axis::Horizontal => item.size.width,
            Axis::Vertical => item.size.height,
            Axis::Depth => 0.0,
        })
        .sum::<f32>();
    let fill_count = children
        .iter()
        .filter(|item| item.primary_length(axis).is_fill())
        .count();
    let fill_size = if fill_count > 0 {
        ((primary_capacity - fixed_primary - spacing_total) / fill_count as f32).max(0.0)
    } else {
        0.0
    };

    children
        .iter()
        .map(|item| {
            let size = resolve_item(*item, axis, content.size(), fill_size);
            let rect = match axis {
                Axis::Vertical => Rect::new(content.x, cursor.y, size.width, size.height),
                Axis::Horizontal => Rect::new(cursor.x, content.y, size.width, size.height),
                Axis::Depth => Rect::new(content.x, content.y, size.width, size.height),
            };

            match axis {
                Axis::Vertical => cursor.y += size.height + spacing,
                Axis::Horizontal => cursor.x += size.width + spacing,
                Axis::Depth => {}
            }

            LayoutBox { rect }
        })
        .collect()
}

fn resolve_item(item: LayoutItem, axis: Axis, available: Size, fill_size: f32) -> Size {
    match axis {
        Axis::Horizontal => Size::new(
            resolve_primary(item.width, item.size.width, fill_size),
            resolve_cross(item.height, item.size.height, available.height),
        ),
        Axis::Vertical => Size::new(
            resolve_cross(item.width, item.size.width, available.width),
            resolve_primary(item.height, item.size.height, fill_size),
        ),
        Axis::Depth => Size::new(
            resolve_cross(item.width, item.size.width, available.width),
            resolve_cross(item.height, item.size.height, available.height),
        ),
    }
}

fn resolve_primary(length: Length, intrinsic: f32, fill_size: f32) -> f32 {
    match length {
        Length::Fit => intrinsic,
        Length::Fixed(value) => value.max(0.0),
        Length::Fill => fill_size,
    }
}

fn resolve_cross(length: Length, intrinsic: f32, available: f32) -> f32 {
    match length {
        Length::Fit => intrinsic.min(available).max(0.0),
        Length::Fixed(value) => value.max(0.0).min(available),
        Length::Fill => available.max(0.0),
    }
}
