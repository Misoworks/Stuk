use crate::{
    Element, ElementKind, IconButtonElement, SplitViewElement, StackElement, TextFieldElement,
    ToggleElement,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusTarget {
    pub path: String,
    pub element: ElementKind,
    pub label: Option<String>,
    pub action: Option<String>,
    pub disabled: bool,
}

impl FocusTarget {
    pub fn is_actionable(&self) -> bool {
        self.action
            .as_deref()
            .is_some_and(|action| !action.is_empty())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusDirection {
    Next,
    Previous,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusTraversal {
    targets: Vec<FocusTarget>,
    current: Option<usize>,
}

impl FocusTraversal {
    pub fn from_element(element: &Element) -> Self {
        let targets = focus_targets(element);
        let current = (!targets.is_empty()).then_some(0);
        Self { targets, current }
    }

    pub fn new(targets: Vec<FocusTarget>) -> Self {
        let current = (!targets.is_empty()).then_some(0);
        Self { targets, current }
    }

    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    pub fn len(&self) -> usize {
        self.targets.len()
    }

    pub fn targets(&self) -> &[FocusTarget] {
        &self.targets
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current
    }

    pub fn current(&self) -> Option<&FocusTarget> {
        self.current.and_then(|index| self.targets.get(index))
    }

    pub fn set_current_index(&mut self, index: usize) -> Option<&FocusTarget> {
        if index >= self.targets.len() {
            return None;
        }
        self.current = Some(index);
        self.current()
    }

    pub fn set_current_path(&mut self, path: &str) -> Option<&FocusTarget> {
        let index = self.targets.iter().position(|target| target.path == path)?;
        self.set_current_index(index)
    }

    pub fn focus_first(&mut self) -> Option<&FocusTarget> {
        self.set_current_index(0)
    }

    pub fn focus_last(&mut self) -> Option<&FocusTarget> {
        self.targets
            .len()
            .checked_sub(1)
            .and_then(|index| self.set_current_index(index))
    }

    pub fn focus_next(&mut self) -> Option<&FocusTarget> {
        self.move_focus(FocusDirection::Next)
    }

    pub fn focus_previous(&mut self) -> Option<&FocusTarget> {
        self.move_focus(FocusDirection::Previous)
    }

    pub fn move_focus(&mut self, direction: FocusDirection) -> Option<&FocusTarget> {
        if self.targets.is_empty() {
            self.current = None;
            return None;
        }
        let current = self.current.unwrap_or(0);
        let next = match direction {
            FocusDirection::Next => (current + 1) % self.targets.len(),
            FocusDirection::Previous => {
                if current == 0 {
                    self.targets.len() - 1
                } else {
                    current - 1
                }
            }
        };
        self.current = Some(next);
        self.current()
    }
}

pub fn focus_targets(element: &Element) -> Vec<FocusTarget> {
    let mut targets = Vec::new();
    collect_focus_targets(element, "root", &mut targets);
    targets
}

fn collect_focus_targets(element: &Element, path: &str, targets: &mut Vec<FocusTarget>) {
    match element {
        Element::Empty
        | Element::Spacer(_)
        | Element::Text(_)
        | Element::Media(_)
        | Element::Divider(_)
        | Element::ProgressBar(_)
        | Element::Badge(_)
        | Element::Avatar(_) => {}
        Element::Window(window) => {
            if let Some(content) = &window.content {
                collect_focus_targets(content, &format!("{path}.content"), targets);
            }
        }
        Element::Button(button) => {
            if !button.disabled {
                targets.push(FocusTarget {
                    path: path.to_string(),
                    element: element.kind(),
                    label: Some(button.label.clone()),
                    action: button.action.clone(),
                    disabled: button.disabled,
                });
            }
        }
        Element::IconButton(button) => collect_icon_button(button, path, element.kind(), targets),
        Element::Toggle(toggle) => collect_toggle(toggle, path, element.kind(), targets),
        Element::Checkbox(checkbox) => {
            collect_control(
                &checkbox.label,
                checkbox.action.clone(),
                checkbox.disabled,
                path,
                element.kind(),
                targets,
            );
        }
        Element::Radio(radio) => {
            collect_control(
                &radio.label,
                radio.action.clone(),
                radio.disabled,
                path,
                element.kind(),
                targets,
            );
        }
        Element::Slider(slider) => {
            if !slider.disabled {
                targets.push(FocusTarget {
                    path: path.to_string(),
                    element: element.kind(),
                    label: slider.label.clone(),
                    action: slider.action.clone(),
                    disabled: slider.disabled,
                });
            }
        }
        Element::Tabs(tabs) => collect_options(&tabs.options, path, element.kind(), targets),
        Element::SegmentedControl(control) => {
            collect_options(&control.options, path, element.kind(), targets)
        }
        Element::Card(card) => {
            collect_focus_targets(&card.child, &format!("{path}.child"), targets)
        }
        Element::Tooltip(tooltip) => {
            collect_focus_targets(&tooltip.child, &format!("{path}.child"), targets)
        }
        Element::TextField(field) => collect_text_field(field, path, element.kind(), targets),
        Element::Stack(stack) => collect_stack(stack, path, targets),
        Element::Flex(flex) => {
            for (index, child) in flex.children.iter().enumerate() {
                collect_focus_targets(&child.child, &format!("{path}.children[{index}]"), targets);
            }
        }
        Element::Grid(grid) => {
            for (index, child) in grid.children.iter().enumerate() {
                collect_focus_targets(&child.child, &format!("{path}.children[{index}]"), targets);
            }
        }
        Element::Overlay(overlay) => {
            collect_focus_targets(&overlay.child, &format!("{path}.child"), targets);
            collect_focus_targets(&overlay.overlay, &format!("{path}.overlay"), targets);
        }
        Element::Surface(surface) => {
            collect_focus_targets(&surface.child, &format!("{path}.child"), targets)
        }
        Element::Frame(frame) => {
            collect_focus_targets(&frame.child, &format!("{path}.child"), targets)
        }
        Element::ScrollView(scroll_view) => {
            collect_focus_targets(&scroll_view.child, &format!("{path}.child"), targets);
        }
        Element::VirtualList(list) => {
            for index in list.visible_range() {
                collect_focus_targets(
                    &list.rows[index].child,
                    &format!("{path}.rows[{index}]"),
                    targets,
                );
            }
        }
        Element::Sidebar(sidebar) => collect_children(&sidebar.children, path, targets),
        Element::Toolbar(toolbar) => collect_children(&toolbar.children, path, targets),
        Element::SplitView(split_view) => collect_split_view(split_view, path, targets),
    }
}

fn collect_icon_button(
    button: &IconButtonElement,
    path: &str,
    element: ElementKind,
    targets: &mut Vec<FocusTarget>,
) {
    if !button.disabled {
        targets.push(FocusTarget {
            path: path.to_string(),
            element,
            label: Some(button.label.clone()),
            action: button.action.clone(),
            disabled: button.disabled,
        });
    }
}

fn collect_toggle(
    toggle: &ToggleElement,
    path: &str,
    element: ElementKind,
    targets: &mut Vec<FocusTarget>,
) {
    if !toggle.disabled {
        targets.push(FocusTarget {
            path: path.to_string(),
            element,
            label: Some(toggle.label.clone()),
            action: toggle.action.clone(),
            disabled: toggle.disabled,
        });
    }
}

fn collect_text_field(
    field: &TextFieldElement,
    path: &str,
    element: ElementKind,
    targets: &mut Vec<FocusTarget>,
) {
    if !field.disabled {
        targets.push(FocusTarget {
            path: path.to_string(),
            element,
            label: field
                .label
                .clone()
                .or_else(|| (!field.placeholder.is_empty()).then(|| field.placeholder.clone())),
            action: None,
            disabled: field.disabled,
        });
    }
}

fn collect_control(
    label: &str,
    action: Option<String>,
    disabled: bool,
    path: &str,
    element: ElementKind,
    targets: &mut Vec<FocusTarget>,
) {
    if !disabled {
        targets.push(FocusTarget {
            path: path.to_string(),
            element,
            label: Some(label.to_string()),
            action,
            disabled,
        });
    }
}

fn collect_options(
    options: &[crate::ControlOptionElement],
    path: &str,
    element: ElementKind,
    targets: &mut Vec<FocusTarget>,
) {
    for (index, option) in options.iter().enumerate() {
        collect_control(
            &option.label,
            option.action.clone(),
            option.disabled,
            &format!("{path}.options[{index}]"),
            element,
            targets,
        );
    }
}

fn collect_stack(stack: &StackElement, path: &str, targets: &mut Vec<FocusTarget>) {
    collect_children(&stack.children, path, targets);
}

fn collect_split_view(split_view: &SplitViewElement, path: &str, targets: &mut Vec<FocusTarget>) {
    collect_focus_targets(&split_view.sidebar, &format!("{path}.sidebar"), targets);
    collect_focus_targets(&split_view.main, &format!("{path}.main"), targets);
}

fn collect_children(children: &[Element], path: &str, targets: &mut Vec<FocusTarget>) {
    for (index, child) in children.iter().enumerate() {
        collect_focus_targets(child, &format!("{path}.children[{index}]"), targets);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ButtonElement, TextFieldElement};
    use stuk_style::ButtonVariant;

    #[test]
    fn collects_enabled_controls_in_tree_order() {
        let element = Element::Stack(StackElement {
            axis: stuk_layout::Axis::Vertical,
            padding: stuk_layout::EdgeInsets::default(),
            spacing: 0.0,
            children: vec![
                Element::Button(ButtonElement {
                    label: "Save".to_string(),
                    variant: ButtonVariant::Primary,
                    action: Some("document.save".to_string()),
                    disabled: false,
                }),
                Element::TextField(TextFieldElement {
                    label: Some("Search".to_string()),
                    text: String::new(),
                    placeholder: String::new(),
                    disabled: false,
                }),
                Element::Button(ButtonElement {
                    label: "Disabled".to_string(),
                    variant: ButtonVariant::Secondary,
                    action: Some("document.disabled".to_string()),
                    disabled: true,
                }),
            ],
        });

        let targets = focus_targets(&element);

        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].path, "root.children[0]");
        assert_eq!(targets[0].element, ElementKind::Button);
        assert!(targets[0].is_actionable());
        assert_eq!(targets[1].path, "root.children[1]");
        assert_eq!(targets[1].element, ElementKind::TextField);
    }

    #[test]
    fn traversal_wraps_forward_and_backward() {
        let targets = vec![
            FocusTarget {
                path: "a".to_string(),
                element: ElementKind::Button,
                label: Some("A".to_string()),
                action: Some("a.run".to_string()),
                disabled: false,
            },
            FocusTarget {
                path: "b".to_string(),
                element: ElementKind::TextField,
                label: Some("B".to_string()),
                action: None,
                disabled: false,
            },
        ];
        let mut traversal = FocusTraversal::new(targets);

        assert_eq!(traversal.current().unwrap().path, "a");
        assert_eq!(traversal.focus_next().unwrap().path, "b");
        assert_eq!(traversal.focus_next().unwrap().path, "a");
        assert_eq!(traversal.focus_previous().unwrap().path, "b");
        assert_eq!(traversal.set_current_path("a").unwrap().path, "a");
        assert!(traversal.set_current_path("missing").is_none());
    }
}
