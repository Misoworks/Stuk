use stuk_core::{Element, MediaElement, MediaSource};
use stuk_layout::{Length, Size};
use stuk_style::Color;

#[derive(Clone, Debug)]
pub struct Image {
    element: MediaElement,
}

impl Image {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            element: MediaElement::new(MediaSource::Image, id),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.element.label = Some(label.into());
        self
    }

    pub fn decorative(mut self) -> Self {
        self.element.decorative = true;
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.element.natural_size = Size::new(width.max(1.0), height.max(1.0));
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

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.element.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn outline(mut self, enabled: bool) -> Self {
        self.element.outline = enabled;
        self
    }

    pub fn without_outline(self) -> Self {
        self.outline(false)
    }
}

impl From<Image> for Element {
    fn from(image: Image) -> Self {
        image.element.into()
    }
}

#[derive(Clone, Debug)]
pub struct Svg {
    element: MediaElement,
}

impl Svg {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            element: MediaElement::new(MediaSource::Svg, id),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.element.label = Some(label.into());
        self
    }

    pub fn decorative(mut self) -> Self {
        self.element.decorative = true;
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.element.natural_size = Size::new(width.max(1.0), height.max(1.0));
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

    pub fn tint(mut self, tint: Color) -> Self {
        self.element.tint = Some(tint);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.element.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn outline(mut self, enabled: bool) -> Self {
        self.element.outline = enabled;
        self
    }
}

impl From<Svg> for Element {
    fn from(svg: Svg) -> Self {
        svg.element.into()
    }
}
