use stuk_layout::{Length, Size};
use stuk_style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaSource {
    Image,
    Svg,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MediaElement {
    pub source: MediaSource,
    pub id: String,
    pub label: Option<String>,
    pub width: Length,
    pub height: Length,
    pub natural_size: Size,
    pub opacity: f32,
    pub tint: Option<Color>,
    pub decorative: bool,
    pub outline: bool,
}

impl MediaElement {
    pub fn new(source: MediaSource, id: impl Into<String>) -> Self {
        Self {
            source,
            id: id.into(),
            label: None,
            width: Length::Fit,
            height: Length::Fit,
            natural_size: Size::new(120.0, 80.0),
            opacity: 1.0,
            tint: None,
            decorative: false,
            outline: source == MediaSource::Image,
        }
    }
}
