use stuk_core::{Element, SurfaceBorder, SurfaceElement, SurfaceShadow};
use stuk_layout::{EdgeInsets, Length};
use stuk_style::Material;

#[derive(Clone, Debug)]
pub struct Surface {
    element: SurfaceElement,
}

impl Surface {
    pub fn new(child: impl Into<Element>) -> Self {
        Self {
            element: SurfaceElement::new(child),
        }
    }

    pub fn material(mut self, material: Material) -> Self {
        self.element.material = material;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.element.padding = EdgeInsets::all(padding);
        self
    }

    pub fn margin(mut self, margin: f32) -> Self {
        self.element.margin = EdgeInsets::all(margin);
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.element.radius = radius.max(0.0);
        self
    }

    pub fn border(mut self, border: SurfaceBorder) -> Self {
        self.element.border = Some(border);
        self
    }

    pub fn shadow(mut self, shadow: SurfaceShadow) -> Self {
        self.element.shadow = Some(shadow);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.element.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn clip(mut self, clip: bool) -> Self {
        self.element.clip = clip;
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

    pub fn min_width(mut self, width: f32) -> Self {
        self.element.min_width = Some(width.max(0.0));
        self
    }

    pub fn max_width(mut self, width: f32) -> Self {
        self.element.max_width = Some(width.max(0.0));
        self
    }

    pub fn min_height(mut self, height: f32) -> Self {
        self.element.min_height = Some(height.max(0.0));
        self
    }

    pub fn max_height(mut self, height: f32) -> Self {
        self.element.max_height = Some(height.max(0.0));
        self
    }
}

impl From<Surface> for Element {
    fn from(surface: Surface) -> Self {
        surface.element.into()
    }
}
