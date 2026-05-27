use stuk_style::Color;

use crate::element::Element;

#[derive(Clone, Debug)]
pub struct CheckboxElement {
    pub label: String,
    pub checked: bool,
    pub action: Option<String>,
    pub disabled: bool,
}

#[derive(Clone, Debug)]
pub struct RadioElement {
    pub label: String,
    pub selected: bool,
    pub action: Option<String>,
    pub disabled: bool,
}

#[derive(Clone, Debug)]
pub struct SliderElement {
    pub label: Option<String>,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub action: Option<String>,
    pub disabled: bool,
}

#[derive(Clone, Debug)]
pub struct ProgressBarElement {
    pub label: Option<String>,
    pub value: f32,
    pub max: f32,
}

#[derive(Clone, Debug)]
pub struct ControlOptionElement {
    pub id: String,
    pub label: String,
    pub action: Option<String>,
    pub disabled: bool,
}

#[derive(Clone, Debug)]
pub struct TabsElement {
    pub options: Vec<ControlOptionElement>,
    pub selected: usize,
}

#[derive(Clone, Debug)]
pub struct SegmentedControlElement {
    pub label: Option<String>,
    pub options: Vec<ControlOptionElement>,
    pub selected: usize,
}

#[derive(Clone, Debug)]
pub struct BadgeElement {
    pub label: String,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub struct AvatarElement {
    pub label: String,
    pub initials: String,
}

#[derive(Clone, Debug)]
pub struct CardElement {
    pub child: Box<Element>,
}

#[derive(Clone, Debug)]
pub struct TooltipElement {
    pub label: String,
    pub child: Box<Element>,
}
