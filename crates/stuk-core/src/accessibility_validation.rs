use crate::layout_elements::{FlexElement, GridElement};
use crate::list_elements::VirtualListElement;
use crate::{
    CheckboxElement, ControlOptionElement, Element, ElementKind, IconButtonElement, RadioElement,
    SegmentedControlElement, SliderElement, SplitViewElement, StackElement, TextFieldElement,
    ToggleElement,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessibilityDiagnostic {
    pub level: AccessibilityDiagnosticLevel,
    pub kind: AccessibilityDiagnosticKind,
    pub path: String,
    pub element: ElementKind,
    pub message: String,
    pub fix_hint: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessibilityDiagnosticLevel {
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessibilityDiagnosticKind {
    MissingLabel,
    MissingTitle,
    UnreachableKeyboardControl,
}

pub fn validate_accessibility(element: &Element) -> Vec<AccessibilityDiagnostic> {
    let mut diagnostics = Vec::new();
    validate_element(element, "root", &mut diagnostics);
    diagnostics
}

fn validate_element(element: &Element, path: &str, diagnostics: &mut Vec<AccessibilityDiagnostic>) {
    match element {
        Element::Empty | Element::Spacer(_) | Element::Text(_) | Element::Divider(_) => {}
        Element::Window(window) => {
            if window.title.trim().is_empty() {
                diagnostics.push(diagnostic(
                    AccessibilityDiagnosticKind::MissingTitle,
                    path,
                    element.kind(),
                    "Window title is empty.",
                    "Set a title so assistive technologies can identify the window.",
                ));
            }
            if let Some(content) = &window.content {
                validate_element(content, &format!("{path}.content"), diagnostics);
            }
        }
        Element::Button(button) => {
            validate_label(&button.label, path, element.kind(), "Button", diagnostics);
            validate_action(
                button.action.as_deref(),
                button.disabled,
                path,
                element.kind(),
                diagnostics,
            );
        }
        Element::IconButton(button) => {
            validate_icon_button(button, path, element.kind(), diagnostics)
        }
        Element::Toggle(toggle) => validate_toggle(toggle, path, element.kind(), diagnostics),
        Element::Checkbox(checkbox) => {
            validate_checkbox(checkbox, path, element.kind(), diagnostics)
        }
        Element::Radio(radio) => validate_radio(radio, path, element.kind(), diagnostics),
        Element::Slider(slider) => validate_slider(slider, path, element.kind(), diagnostics),
        Element::ProgressBar(progress) => {
            if progress.label.as_deref().is_none_or(str::is_empty) {
                diagnostics.push(diagnostic(
                    AccessibilityDiagnosticKind::MissingLabel,
                    path,
                    element.kind(),
                    "Progress bar is missing a label.",
                    "Set a label so screen readers can identify the progress value.",
                ));
            }
        }
        Element::Tabs(tabs) => validate_options(&tabs.options, path, element.kind(), diagnostics),
        Element::SegmentedControl(control) => {
            validate_segmented_control(control, path, element.kind(), diagnostics)
        }
        Element::Badge(badge) => {
            validate_label(&badge.label, path, element.kind(), "Badge", diagnostics)
        }
        Element::Avatar(avatar) => {
            validate_label(&avatar.label, path, element.kind(), "Avatar", diagnostics)
        }
        Element::Card(card) => validate_element(&card.child, &format!("{path}.child"), diagnostics),
        Element::Tooltip(tooltip) => {
            validate_label(&tooltip.label, path, element.kind(), "Tooltip", diagnostics);
            validate_element(&tooltip.child, &format!("{path}.child"), diagnostics);
        }
        Element::TextField(field) => validate_text_field(field, path, element.kind(), diagnostics),
        Element::Stack(stack) => validate_stack(stack, path, diagnostics),
        Element::Flex(flex) => validate_flex(flex, path, diagnostics),
        Element::Grid(grid) => validate_grid(grid, path, diagnostics),
        Element::Overlay(overlay) => {
            validate_element(&overlay.child, &format!("{path}.child"), diagnostics);
            validate_element(&overlay.overlay, &format!("{path}.overlay"), diagnostics);
        }
        Element::Surface(surface) => {
            validate_element(&surface.child, &format!("{path}.child"), diagnostics);
        }
        Element::Media(media) => {
            if !media.decorative
                && media
                    .label
                    .as_deref()
                    .is_none_or(|label| label.trim().is_empty())
            {
                diagnostics.push(diagnostic(
                    AccessibilityDiagnosticKind::MissingLabel,
                    path,
                    element.kind(),
                    "Media is missing a label.",
                    "Set a label or mark decorative media as decorative.",
                ));
            }
        }
        Element::Frame(frame) => {
            validate_element(&frame.child, &format!("{path}.child"), diagnostics)
        }
        Element::ScrollView(scroll_view) => {
            validate_element(&scroll_view.child, &format!("{path}.child"), diagnostics);
        }
        Element::VirtualList(list) => validate_virtual_list(list, path, diagnostics),
        Element::Sidebar(sidebar) => validate_children(&sidebar.children, path, diagnostics),
        Element::Toolbar(toolbar) => {
            if toolbar.title.trim().is_empty() {
                diagnostics.push(diagnostic(
                    AccessibilityDiagnosticKind::MissingTitle,
                    path,
                    element.kind(),
                    "Toolbar title is empty.",
                    "Set a toolbar title or provide an accessible label.",
                ));
            }
            validate_children(&toolbar.children, path, diagnostics);
        }
        Element::SplitView(split_view) => validate_split_view(split_view, path, diagnostics),
    }
}

fn validate_icon_button(
    button: &IconButtonElement,
    path: &str,
    element: ElementKind,
    diagnostics: &mut Vec<AccessibilityDiagnostic>,
) {
    validate_label(&button.label, path, element, "Icon button", diagnostics);
    validate_action(
        button.action.as_deref(),
        button.disabled,
        path,
        element,
        diagnostics,
    );
}

fn validate_toggle(
    toggle: &ToggleElement,
    path: &str,
    element: ElementKind,
    diagnostics: &mut Vec<AccessibilityDiagnostic>,
) {
    validate_label(&toggle.label, path, element, "Toggle", diagnostics);
    validate_action(
        toggle.action.as_deref(),
        toggle.disabled,
        path,
        element,
        diagnostics,
    );
}

fn validate_checkbox(
    checkbox: &CheckboxElement,
    path: &str,
    element: ElementKind,
    diagnostics: &mut Vec<AccessibilityDiagnostic>,
) {
    validate_label(&checkbox.label, path, element, "Checkbox", diagnostics);
    validate_action(
        checkbox.action.as_deref(),
        checkbox.disabled,
        path,
        element,
        diagnostics,
    );
}

fn validate_radio(
    radio: &RadioElement,
    path: &str,
    element: ElementKind,
    diagnostics: &mut Vec<AccessibilityDiagnostic>,
) {
    validate_label(&radio.label, path, element, "Radio", diagnostics);
    validate_action(
        radio.action.as_deref(),
        radio.disabled,
        path,
        element,
        diagnostics,
    );
}

fn validate_slider(
    slider: &SliderElement,
    path: &str,
    element: ElementKind,
    diagnostics: &mut Vec<AccessibilityDiagnostic>,
) {
    if slider.label.as_deref().is_none_or(str::is_empty) {
        diagnostics.push(diagnostic(
            AccessibilityDiagnosticKind::MissingLabel,
            path,
            element,
            "Slider is missing a label.",
            "Set a label so screen readers can identify the slider.",
        ));
    }
    validate_action(
        slider.action.as_deref(),
        slider.disabled,
        path,
        element,
        diagnostics,
    );
}

fn validate_segmented_control(
    control: &SegmentedControlElement,
    path: &str,
    element: ElementKind,
    diagnostics: &mut Vec<AccessibilityDiagnostic>,
) {
    if control.label.as_deref().is_none_or(str::is_empty) {
        diagnostics.push(diagnostic(
            AccessibilityDiagnosticKind::MissingLabel,
            path,
            element,
            "Segmented control is missing a label.",
            "Set a label so screen readers can identify the option group.",
        ));
    }
    validate_options(&control.options, path, element, diagnostics);
}

fn validate_options(
    options: &[ControlOptionElement],
    path: &str,
    element: ElementKind,
    diagnostics: &mut Vec<AccessibilityDiagnostic>,
) {
    for (index, option) in options.iter().enumerate() {
        let option_path = format!("{path}.options[{index}]");
        validate_label(&option.label, &option_path, element, "Option", diagnostics);
        validate_action(
            option.action.as_deref(),
            option.disabled,
            &option_path,
            element,
            diagnostics,
        );
    }
}

fn validate_text_field(
    field: &TextFieldElement,
    path: &str,
    element: ElementKind,
    diagnostics: &mut Vec<AccessibilityDiagnostic>,
) {
    if field
        .label
        .as_deref()
        .is_none_or(|label| label.trim().is_empty())
    {
        diagnostics.push(diagnostic(
            AccessibilityDiagnosticKind::MissingLabel,
            path,
            element,
            "Text field is missing a label.",
            "Set a label so screen readers can identify the field.",
        ));
    }
}

fn validate_stack(
    stack: &StackElement,
    path: &str,
    diagnostics: &mut Vec<AccessibilityDiagnostic>,
) {
    validate_children(&stack.children, path, diagnostics);
}

fn validate_flex(flex: &FlexElement, path: &str, diagnostics: &mut Vec<AccessibilityDiagnostic>) {
    for (index, child) in flex.children.iter().enumerate() {
        validate_element(
            &child.child,
            &format!("{path}.children[{index}]"),
            diagnostics,
        );
    }
}

fn validate_grid(grid: &GridElement, path: &str, diagnostics: &mut Vec<AccessibilityDiagnostic>) {
    for (index, child) in grid.children.iter().enumerate() {
        validate_element(
            &child.child,
            &format!("{path}.children[{index}]"),
            diagnostics,
        );
    }
}

fn validate_split_view(
    split_view: &SplitViewElement,
    path: &str,
    diagnostics: &mut Vec<AccessibilityDiagnostic>,
) {
    validate_element(&split_view.sidebar, &format!("{path}.sidebar"), diagnostics);
    validate_element(&split_view.main, &format!("{path}.main"), diagnostics);
}

fn validate_virtual_list(
    list: &VirtualListElement,
    path: &str,
    diagnostics: &mut Vec<AccessibilityDiagnostic>,
) {
    for (index, row) in list.rows.iter().enumerate() {
        validate_label(
            &row.key,
            &format!("{path}.rows[{index}]"),
            ElementKind::VirtualList,
            "Virtual list row",
            diagnostics,
        );
        validate_element(
            &row.child,
            &format!("{path}.rows[{index}].child"),
            diagnostics,
        );
    }
}

fn validate_children(
    children: &[Element],
    path: &str,
    diagnostics: &mut Vec<AccessibilityDiagnostic>,
) {
    for (index, child) in children.iter().enumerate() {
        validate_element(child, &format!("{path}.children[{index}]"), diagnostics);
    }
}

fn validate_label(
    label: &str,
    path: &str,
    element: ElementKind,
    widget_name: &str,
    diagnostics: &mut Vec<AccessibilityDiagnostic>,
) {
    if label.trim().is_empty() {
        diagnostics.push(diagnostic(
            AccessibilityDiagnosticKind::MissingLabel,
            path,
            element,
            &format!("{widget_name} is missing a label."),
            "Provide a concise accessible label.",
        ));
    }
}

fn validate_action(
    action: Option<&str>,
    disabled: bool,
    path: &str,
    element: ElementKind,
    diagnostics: &mut Vec<AccessibilityDiagnostic>,
) {
    if !disabled && action.is_none_or(|action| action.trim().is_empty()) {
        diagnostics.push(diagnostic(
            AccessibilityDiagnosticKind::UnreachableKeyboardControl,
            path,
            element,
            "Interactive control is not wired to an action.",
            "Attach an action so the control can be reached by keyboard and command systems.",
        ));
    }
}

fn diagnostic(
    kind: AccessibilityDiagnosticKind,
    path: &str,
    element: ElementKind,
    message: &str,
    fix_hint: &str,
) -> AccessibilityDiagnostic {
    AccessibilityDiagnostic {
        level: AccessibilityDiagnosticLevel::Warning,
        kind,
        path: path.to_string(),
        element,
        message: message.to_string(),
        fix_hint: Some(fix_hint.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ButtonElement, TextFieldElement};
    use stuk_style::ButtonVariant;

    #[test]
    fn warns_about_missing_labels_and_unwired_controls() {
        let element = Element::Stack(StackElement {
            axis: stuk_layout::Axis::Vertical,
            padding: stuk_layout::EdgeInsets::default(),
            spacing: 0.0,
            children: vec![
                Element::Button(ButtonElement {
                    label: String::new(),
                    variant: ButtonVariant::Primary,
                    action: None,
                    disabled: false,
                }),
                Element::TextField(TextFieldElement {
                    label: None,
                    text: String::new(),
                    placeholder: "Search".to_string(),
                    disabled: false,
                }),
            ],
        });

        let diagnostics = validate_accessibility(&element);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == AccessibilityDiagnosticKind::MissingLabel
                && diagnostic.element == ElementKind::Button
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == AccessibilityDiagnosticKind::UnreachableKeyboardControl
                && diagnostic.element == ElementKind::Button
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == AccessibilityDiagnosticKind::MissingLabel
                && diagnostic.element == ElementKind::TextField
        }));
    }

    #[test]
    fn ignores_disabled_controls_without_actions() {
        let element = Element::Button(ButtonElement {
            label: "Save".to_string(),
            variant: ButtonVariant::Primary,
            action: None,
            disabled: true,
        });

        assert!(validate_accessibility(&element).is_empty());
    }
}
