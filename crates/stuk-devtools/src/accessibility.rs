use stuk_core::{
    AccessibilityDiagnostic, AccessibilityDiagnosticKind, AccessibilityDiagnosticLevel, Element,
    ElementKind, validate_accessibility,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessibilityInspection {
    pub ok: bool,
    pub controls: usize,
    pub labeled_controls: usize,
    pub diagnostics: Vec<AccessibilityDiagnosticInspection>,
}

impl AccessibilityInspection {
    pub fn from_element(element: &Element) -> Self {
        let diagnostics = validate_accessibility(element);
        Self {
            ok: diagnostics
                .iter()
                .all(|diagnostic| diagnostic.level != AccessibilityDiagnosticLevel::Error),
            controls: control_count(element),
            labeled_controls: labeled_control_count(element),
            diagnostics: diagnostics
                .into_iter()
                .map(accessibility_diagnostic_inspection)
                .collect(),
        }
    }

    pub fn to_text(&self) -> String {
        let mut output = String::from("Accessibility\n");
        output.push_str(&format!("  controls: {}\n", self.controls));
        output.push_str(&format!("  labeled_controls: {}\n", self.labeled_controls));
        output.push_str(&format!("Diagnostics: {}\n", self.diagnostics.len()));
        for diagnostic in &self.diagnostics {
            output.push_str(&format!(
                "  {}: {}: {}: {}\n",
                diagnostic.level, diagnostic.path, diagnostic.element, diagnostic.message
            ));
            if let Some(fix_hint) = &diagnostic.fix_hint {
                output.push_str(&format!("    fix: {fix_hint}\n"));
            }
        }
        output
    }

    pub fn to_json(&self) -> String {
        let diagnostics = self
            .diagnostics
            .iter()
            .map(AccessibilityDiagnosticInspection::to_json)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"ok\":{},\"controls\":{},\"labeled_controls\":{},\"diagnostics\":[{}]}}",
            self.ok, self.controls, self.labeled_controls, diagnostics
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessibilityDiagnosticInspection {
    pub level: String,
    pub kind: String,
    pub path: String,
    pub element: String,
    pub message: String,
    pub fix_hint: Option<String>,
}

impl AccessibilityDiagnosticInspection {
    fn to_json(&self) -> String {
        format!(
            "{{\"level\":\"{}\",\"kind\":\"{}\",\"path\":\"{}\",\"element\":\"{}\",\"message\":\"{}\",\"fix_hint\":{}}}",
            escape_json(&self.level),
            escape_json(&self.kind),
            escape_json(&self.path),
            escape_json(&self.element),
            escape_json(&self.message),
            optional_json_string(self.fix_hint.as_deref())
        )
    }
}

pub fn inspect_accessibility(element: &Element) -> AccessibilityInspection {
    AccessibilityInspection::from_element(element)
}

fn accessibility_diagnostic_inspection(
    diagnostic: AccessibilityDiagnostic,
) -> AccessibilityDiagnosticInspection {
    AccessibilityDiagnosticInspection {
        level: accessibility_level_name(diagnostic.level).to_string(),
        kind: accessibility_kind_name(diagnostic.kind).to_string(),
        path: diagnostic.path,
        element: element_kind_name(diagnostic.element).to_string(),
        message: diagnostic.message,
        fix_hint: diagnostic.fix_hint,
    }
}

fn control_count(element: &Element) -> usize {
    usize::from(is_accessible_control(element))
        + element_children(element)
            .into_iter()
            .map(control_count)
            .sum::<usize>()
}

fn labeled_control_count(element: &Element) -> usize {
    usize::from(is_labeled_accessible_control(element))
        + element_children(element)
            .into_iter()
            .map(labeled_control_count)
            .sum::<usize>()
}

fn is_accessible_control(element: &Element) -> bool {
    matches!(
        element,
        Element::Button(_)
            | Element::IconButton(_)
            | Element::Toggle(_)
            | Element::Checkbox(_)
            | Element::Radio(_)
            | Element::Slider(_)
            | Element::Tabs(_)
            | Element::SegmentedControl(_)
            | Element::TextField(_)
    )
}

fn is_labeled_accessible_control(element: &Element) -> bool {
    match element {
        Element::Button(button) => !button.label.trim().is_empty(),
        Element::IconButton(button) => !button.label.trim().is_empty(),
        Element::Toggle(toggle) => !toggle.label.trim().is_empty(),
        Element::Checkbox(checkbox) => !checkbox.label.trim().is_empty(),
        Element::Radio(radio) => !radio.label.trim().is_empty(),
        Element::Slider(slider) => slider
            .label
            .as_deref()
            .is_some_and(|label| !label.trim().is_empty()),
        Element::Tabs(tabs) => tabs
            .options
            .iter()
            .all(|option| !option.label.trim().is_empty()),
        Element::SegmentedControl(control) => {
            control
                .label
                .as_deref()
                .is_some_and(|label| !label.trim().is_empty())
                && control
                    .options
                    .iter()
                    .all(|option| !option.label.trim().is_empty())
        }
        Element::TextField(field) => field
            .label
            .as_deref()
            .is_some_and(|label| !label.trim().is_empty()),
        _ => false,
    }
}

fn element_children(element: &Element) -> Vec<&Element> {
    match element {
        Element::Window(window) => window.content.iter().map(Box::as_ref).collect(),
        Element::Stack(stack) => stack.children.iter().collect(),
        Element::Flex(flex) => flex.children.iter().map(|child| &child.child).collect(),
        Element::Grid(grid) => grid.children.iter().map(|child| &child.child).collect(),
        Element::Overlay(overlay) => vec![overlay.child.as_ref(), overlay.overlay.as_ref()],
        Element::Surface(surface) => vec![surface.child.as_ref()],
        Element::Frame(frame) => vec![frame.child.as_ref()],
        Element::Card(card) => vec![card.child.as_ref()],
        Element::Tooltip(tooltip) => vec![tooltip.child.as_ref()],
        Element::ScrollView(scroll_view) => vec![scroll_view.child.as_ref()],
        Element::VirtualList(list) => list
            .visible_range()
            .map(|index| &list.rows[index].child)
            .collect(),
        Element::Sidebar(sidebar) => sidebar.children.iter().collect(),
        Element::Toolbar(toolbar) => toolbar.children.iter().collect(),
        Element::SplitView(split_view) => {
            vec![split_view.sidebar.as_ref(), split_view.main.as_ref()]
        }
        _ => Vec::new(),
    }
}

fn accessibility_level_name(level: AccessibilityDiagnosticLevel) -> &'static str {
    match level {
        AccessibilityDiagnosticLevel::Warning => "warning",
        AccessibilityDiagnosticLevel::Error => "error",
    }
}

fn accessibility_kind_name(kind: AccessibilityDiagnosticKind) -> &'static str {
    match kind {
        AccessibilityDiagnosticKind::MissingLabel => "missing_label",
        AccessibilityDiagnosticKind::MissingTitle => "missing_title",
        AccessibilityDiagnosticKind::UnreachableKeyboardControl => "unreachable_keyboard_control",
    }
}

fn element_kind_name(kind: ElementKind) -> &'static str {
    match kind {
        ElementKind::Empty => "empty",
        ElementKind::Window => "window",
        ElementKind::Text => "text",
        ElementKind::Button => "button",
        ElementKind::VStack => "vstack",
        ElementKind::HStack => "hstack",
        ElementKind::ZStack => "zstack",
        ElementKind::Flex => "flex",
        ElementKind::Grid => "grid",
        ElementKind::Overlay => "overlay",
        ElementKind::Surface => "surface",
        ElementKind::Image => "image",
        ElementKind::Svg => "svg",
        ElementKind::Frame => "frame",
        ElementKind::Spacer => "spacer",
        ElementKind::Divider => "divider",
        ElementKind::IconButton => "icon_button",
        ElementKind::Toggle => "toggle",
        ElementKind::Checkbox => "checkbox",
        ElementKind::Radio => "radio",
        ElementKind::Slider => "slider",
        ElementKind::ProgressBar => "progress_bar",
        ElementKind::Tabs => "tabs",
        ElementKind::SegmentedControl => "segmented_control",
        ElementKind::Badge => "badge",
        ElementKind::Avatar => "avatar",
        ElementKind::Card => "card",
        ElementKind::Tooltip => "tooltip",
        ElementKind::TextField => "text_field",
        ElementKind::ScrollView => "scroll_view",
        ElementKind::VirtualList => "virtual_list",
        ElementKind::Sidebar => "sidebar",
        ElementKind::Toolbar => "toolbar",
        ElementKind::SplitView => "split_view",
    }
}

fn optional_json_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", escape_json(value)))
        .unwrap_or_else(|| "null".to_string())
}

fn escape_json(value: &str) -> String {
    let mut output = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch.is_control() => output.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => output.push(ch),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use stuk_widgets::{Button, TextField, VStack};

    #[test]
    fn reports_preview_accessibility_diagnostics() {
        let element: Element = VStack::new()
            .child(Button::primary("").action("document.save"))
            .child(TextField::new("").placeholder("Search"))
            .into();

        let inspection = inspect_accessibility(&element);

        assert!(inspection.ok);
        assert_eq!(inspection.controls, 2);
        assert_eq!(inspection.labeled_controls, 0);
        assert!(inspection.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == "missing_label" && diagnostic.element == "button"
        }));
        assert!(inspection.to_json().contains("\"controls\":2"));
    }
}
