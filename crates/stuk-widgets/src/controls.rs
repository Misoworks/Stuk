use stuk_core::{
    AvatarElement, BadgeElement, CardElement, CheckboxElement, ControlOptionElement, Element,
    ProgressBarElement, RadioElement, SegmentedControlElement, SliderElement, TabsElement,
    TooltipElement,
};
use stuk_style::Color;

#[derive(Clone, Debug)]
pub struct Checkbox {
    element: CheckboxElement,
}

impl Checkbox {
    pub fn new(label: impl Into<String>, checked: bool) -> Self {
        Self {
            element: CheckboxElement {
                label: label.into(),
                checked,
                action: None,
                disabled: false,
            },
        }
    }

    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.element.action = Some(action.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.element.disabled = disabled;
        self
    }
}

impl From<Checkbox> for Element {
    fn from(checkbox: Checkbox) -> Self {
        checkbox.element.into()
    }
}

#[derive(Clone, Debug)]
pub struct Radio {
    element: RadioElement,
}

impl Radio {
    pub fn new(label: impl Into<String>, selected: bool) -> Self {
        Self {
            element: RadioElement {
                label: label.into(),
                selected,
                action: None,
                disabled: false,
            },
        }
    }

    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.element.action = Some(action.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.element.disabled = disabled;
        self
    }
}

impl From<Radio> for Element {
    fn from(radio: Radio) -> Self {
        radio.element.into()
    }
}

#[derive(Clone, Debug)]
pub struct Slider {
    element: SliderElement,
}

impl Slider {
    pub fn new(value: f32, min: f32, max: f32) -> Self {
        Self {
            element: SliderElement {
                label: None,
                value,
                min,
                max,
                step: 1.0,
                action: None,
                disabled: false,
            },
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.element.label = Some(label.into());
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.element.step = step.max(0.0);
        self
    }

    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.element.action = Some(action.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.element.disabled = disabled;
        self
    }
}

impl From<Slider> for Element {
    fn from(slider: Slider) -> Self {
        slider.element.into()
    }
}

#[derive(Clone, Debug)]
pub struct ProgressBar {
    element: ProgressBarElement,
}

impl ProgressBar {
    pub fn new(value: f32, max: f32) -> Self {
        Self {
            element: ProgressBarElement {
                label: None,
                value,
                max,
                color: Color::ACCENT,
            },
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.element.label = Some(label.into());
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.element.color = color;
        self
    }
}

impl From<ProgressBar> for Element {
    fn from(progress: ProgressBar) -> Self {
        progress.element.into()
    }
}

#[derive(Clone, Debug)]
pub struct Tabs {
    element: TabsElement,
}

impl Tabs {
    pub fn new() -> Self {
        Self {
            element: TabsElement {
                options: Vec::new(),
                selected: 0,
            },
        }
    }

    pub fn selected(mut self, selected: usize) -> Self {
        self.element.selected = selected;
        self
    }

    pub fn tab(mut self, id: impl Into<String>, label: impl Into<String>) -> Self {
        self.element.options.push(option(id, label));
        self
    }

    pub fn action_prefix(mut self, prefix: impl AsRef<str>) -> Self {
        for item in &mut self.element.options {
            item.action = Some(format!("{}.{}", prefix.as_ref(), item.id));
        }
        self
    }
}

impl Default for Tabs {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Tabs> for Element {
    fn from(tabs: Tabs) -> Self {
        tabs.element.into()
    }
}

#[derive(Clone, Debug)]
pub struct SegmentedControl {
    element: SegmentedControlElement,
}

impl SegmentedControl {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            element: SegmentedControlElement {
                label: Some(label.into()),
                options: Vec::new(),
                selected: 0,
            },
        }
    }

    pub fn selected(mut self, selected: usize) -> Self {
        self.element.selected = selected;
        self
    }

    pub fn option(mut self, id: impl Into<String>, label: impl Into<String>) -> Self {
        self.element.options.push(option(id, label));
        self
    }

    pub fn action_prefix(mut self, prefix: impl AsRef<str>) -> Self {
        for item in &mut self.element.options {
            item.action = Some(format!("{}.{}", prefix.as_ref(), item.id));
        }
        self
    }
}

impl From<SegmentedControl> for Element {
    fn from(control: SegmentedControl) -> Self {
        control.element.into()
    }
}

#[derive(Clone, Debug)]
pub struct Badge {
    element: BadgeElement,
}

impl Badge {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            element: BadgeElement {
                label: label.into(),
                color: Color::ACCENT,
            },
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.element.color = color;
        self
    }
}

impl From<Badge> for Element {
    fn from(badge: Badge) -> Self {
        badge.element.into()
    }
}

#[derive(Clone, Debug)]
pub struct Avatar {
    element: AvatarElement,
}

impl Avatar {
    pub fn new(label: impl Into<String>, initials: impl Into<String>) -> Self {
        Self {
            element: AvatarElement {
                label: label.into(),
                initials: initials.into(),
            },
        }
    }
}

impl From<Avatar> for Element {
    fn from(avatar: Avatar) -> Self {
        avatar.element.into()
    }
}

#[derive(Clone, Debug)]
pub struct Card {
    element: CardElement,
}

impl Card {
    pub fn new(child: impl Into<Element>) -> Self {
        Self {
            element: CardElement {
                child: Box::new(child.into()),
            },
        }
    }
}

impl From<Card> for Element {
    fn from(card: Card) -> Self {
        card.element.into()
    }
}

#[derive(Clone, Debug)]
pub struct Tooltip {
    element: TooltipElement,
}

impl Tooltip {
    pub fn new(label: impl Into<String>, child: impl Into<Element>) -> Self {
        Self {
            element: TooltipElement {
                label: label.into(),
                child: Box::new(child.into()),
            },
        }
    }
}

impl From<Tooltip> for Element {
    fn from(tooltip: Tooltip) -> Self {
        tooltip.element.into()
    }
}

fn option(id: impl Into<String>, label: impl Into<String>) -> ControlOptionElement {
    ControlOptionElement {
        id: id.into(),
        label: label.into(),
        action: None,
        disabled: false,
    }
}
