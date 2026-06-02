use crate::Size;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Breakpoint {
    Compact,
    Medium,
    Expanded,
    Wide,
}

impl Breakpoint {
    pub const MEDIUM_MIN: f32 = 600.0;
    pub const EXPANDED_MIN: f32 = 900.0;
    pub const WIDE_MIN: f32 = 1200.0;

    pub fn from_width(width: f32) -> Self {
        if width < Self::MEDIUM_MIN {
            Self::Compact
        } else if width < Self::EXPANDED_MIN {
            Self::Medium
        } else if width < Self::WIDE_MIN {
            Self::Expanded
        } else {
            Self::Wide
        }
    }

    pub fn from_size(size: Size) -> Self {
        Self::from_width(size.width)
    }

    pub fn is_at_least(self, breakpoint: Self) -> bool {
        self >= breakpoint
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Responsive<T> {
    base: T,
    compact: Option<T>,
    medium: Option<T>,
    expanded: Option<T>,
    wide: Option<T>,
}

impl<T> Responsive<T> {
    pub fn new(base: T) -> Self {
        Self {
            base,
            compact: None,
            medium: None,
            expanded: None,
            wide: None,
        }
    }

    pub fn compact(mut self, value: T) -> Self {
        self.compact = Some(value);
        self
    }

    pub fn medium(mut self, value: T) -> Self {
        self.medium = Some(value);
        self
    }

    pub fn expanded(mut self, value: T) -> Self {
        self.expanded = Some(value);
        self
    }

    pub fn wide(mut self, value: T) -> Self {
        self.wide = Some(value);
        self
    }
}

impl<T: Clone> Responsive<T> {
    pub fn resolve(&self, breakpoint: Breakpoint) -> T {
        match breakpoint {
            Breakpoint::Compact => self.compact.as_ref().unwrap_or(&self.base),
            Breakpoint::Medium => self
                .medium
                .as_ref()
                .or(self.compact.as_ref())
                .unwrap_or(&self.base),
            Breakpoint::Expanded => self
                .expanded
                .as_ref()
                .or(self.medium.as_ref())
                .or(self.compact.as_ref())
                .unwrap_or(&self.base),
            Breakpoint::Wide => self
                .wide
                .as_ref()
                .or(self.expanded.as_ref())
                .or(self.medium.as_ref())
                .or(self.compact.as_ref())
                .unwrap_or(&self.base),
        }
        .clone()
    }

    pub fn resolve_width(&self, width: f32) -> T {
        self.resolve(Breakpoint::from_width(width))
    }

    pub fn resolve_size(&self, size: Size) -> T {
        self.resolve(Breakpoint::from_size(size))
    }
}

#[cfg(test)]
mod tests {
    use super::{Breakpoint, Responsive};

    #[test]
    fn breakpoint_resolves_from_width() {
        assert_eq!(Breakpoint::from_width(320.0), Breakpoint::Compact);
        assert_eq!(Breakpoint::from_width(700.0), Breakpoint::Medium);
        assert_eq!(Breakpoint::from_width(1000.0), Breakpoint::Expanded);
        assert_eq!(Breakpoint::from_width(1400.0), Breakpoint::Wide);
    }

    #[test]
    fn responsive_values_cascade_upward() {
        let value = Responsive::new("base").compact("phone").expanded("desktop");

        assert_eq!(value.resolve(Breakpoint::Compact), "phone");
        assert_eq!(value.resolve(Breakpoint::Medium), "phone");
        assert_eq!(value.resolve(Breakpoint::Expanded), "desktop");
        assert_eq!(value.resolve(Breakpoint::Wide), "desktop");
    }
}
