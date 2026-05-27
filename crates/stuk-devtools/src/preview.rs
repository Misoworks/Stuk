use stuk_core::Element;
use stuk_layout::Rect;

use crate::{
    AccessibilityInspection, ElementSnapshot, LayoutSnapshot, inspect_accessibility,
    inspect_element, inspect_layout,
};

#[derive(Clone, Debug, PartialEq)]
pub struct PreviewDescriptor {
    pub id: String,
    pub label: String,
    pub width: u32,
    pub height: u32,
    pub theme: Option<String>,
    pub density: Option<String>,
}

impl PreviewDescriptor {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            width: 980,
            height: 680,
            theme: None,
            density: None,
        }
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn theme(mut self, theme: impl Into<String>) -> Self {
        self.theme = Some(theme.into());
        self
    }

    pub fn density(mut self, density: impl Into<String>) -> Self {
        self.density = Some(density.into());
        self
    }
}

#[derive(Clone, Debug)]
pub struct PreviewElement {
    pub descriptor: PreviewDescriptor,
    pub element: Element,
}

impl PreviewElement {
    pub fn new(descriptor: PreviewDescriptor, element: impl Into<Element>) -> Self {
        Self {
            descriptor,
            element: element.into(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PreviewRegistry {
    previews: Vec<PreviewElement>,
}

impl PreviewRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, preview: PreviewElement) {
        if let Some(existing) = self
            .previews
            .iter_mut()
            .find(|existing| existing.descriptor.id == preview.descriptor.id)
        {
            *existing = preview;
        } else {
            self.previews.push(preview);
        }
    }

    pub fn previews(&self) -> &[PreviewElement] {
        &self.previews
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.previews
            .iter()
            .map(|preview| preview.descriptor.id.as_str())
    }

    pub fn get(&self, id: &str) -> Option<&PreviewElement> {
        self.previews
            .iter()
            .find(|preview| preview.descriptor.id == id)
    }

    pub fn inspect(&self, id: &str) -> Option<ElementSnapshot> {
        self.get(id)
            .map(|preview| inspect_element(&preview.element))
    }

    pub fn inspect_layout(&self, id: &str, bounds: Rect) -> Option<LayoutSnapshot> {
        self.get(id)
            .map(|preview| inspect_layout(&preview.element, bounds))
    }

    pub fn inspect_accessibility(&self, id: &str) -> Option<AccessibilityInspection> {
        self.get(id)
            .map(|preview| inspect_accessibility(&preview.element))
    }
}

#[macro_export]
macro_rules! preview {
    ($($name:ident => $view:expr),+ $(,)?) => {{
        let mut registry = $crate::PreviewRegistry::new();
        $(
            registry.insert($crate::PreviewElement::new(
                $crate::PreviewDescriptor::new(stringify!($name), stringify!($name)),
                $view,
            ));
        )+
        registry
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use stuk_widgets::Text;

    #[test]
    fn stores_and_replaces_previews_by_id() {
        let mut registry = PreviewRegistry::new();
        registry.insert(PreviewElement::new(
            PreviewDescriptor::new("note.row", "Note Row").size(420, 160),
            Text::new("Draft"),
        ));
        registry.insert(PreviewElement::new(
            PreviewDescriptor::new("note.row", "Note Row").theme("dark"),
            Text::new("Updated"),
        ));

        let preview = registry.get("note.row").unwrap();
        assert_eq!(registry.previews().len(), 1);
        assert_eq!(preview.descriptor.theme.as_deref(), Some("dark"));
    }

    #[test]
    fn macro_registers_named_previews() {
        let registry = crate::preview! {
            DraftPreview => Text::new("Draft")
        };

        assert_eq!(registry.names().collect::<Vec<_>>(), vec!["DraftPreview"]);
        assert!(registry.inspect("DraftPreview").is_some());
    }

    #[test]
    fn inspects_accessibility_for_registered_preview() {
        let registry = crate::preview! {
            BrokenPreview => stuk_widgets::Button::primary("")
        };

        let inspection = registry.inspect_accessibility("BrokenPreview").unwrap();

        assert_eq!(inspection.controls, 1);
        assert_eq!(inspection.labeled_controls, 0);
        assert!(!inspection.diagnostics.is_empty());
    }
}
