use stuk_layout::{EdgeInsets, Length, Rect};
use stuk_style::{Color, Material};

use crate::Element;

#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceBorder {
    pub color: Color,
    pub thickness: f32,
}

impl SurfaceBorder {
    pub fn new(color: Color, thickness: f32) -> Self {
        Self {
            color,
            thickness: thickness.max(0.0),
        }
    }

    pub fn subtle() -> Self {
        Self::new(Color::WHITE.opacity(0.08), 1.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

impl SurfaceShadow {
    pub fn new(offset_x: f32, offset_y: f32, blur: f32, spread: f32, color: Color) -> Self {
        Self {
            offset_x,
            offset_y,
            blur: blur.max(0.0),
            spread,
            color,
        }
    }

    pub fn soft() -> Self {
        Self::new(0.0, 10.0, 24.0, -8.0, Color::rgba(0.0, 0.0, 0.0, 0.28))
    }

    pub fn medium() -> Self {
        Self::new(0.0, 18.0, 36.0, -10.0, Color::rgba(0.0, 0.0, 0.0, 0.36))
    }
}

#[derive(Clone, Debug)]
pub struct SurfaceElement {
    pub child: Box<Element>,
    pub material: Material,
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,
    pub radius: f32,
    pub border: Option<SurfaceBorder>,
    pub shadow: Option<SurfaceShadow>,
    pub opacity: f32,
    pub width: Length,
    pub height: Length,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub clip: bool,
}

impl SurfaceElement {
    pub fn new(child: impl Into<Element>) -> Self {
        Self {
            child: Box::new(child.into()),
            material: Material::SurfaceElevated,
            padding: EdgeInsets::default(),
            margin: EdgeInsets::default(),
            radius: 12.0,
            border: None,
            shadow: None,
            opacity: 1.0,
            width: Length::Fit,
            height: Length::Fit,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            clip: true,
        }
    }

    pub fn inner_bounds(&self, bounds: Rect) -> Rect {
        self.surface_bounds(bounds).inset(self.padding)
    }

    pub fn surface_bounds(&self, bounds: Rect) -> Rect {
        bounds.inset(self.margin)
    }
}
