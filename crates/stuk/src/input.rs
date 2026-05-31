use stuk_platform::{read_clipboard_text, write_clipboard_text};
use stuk_text::{TextInputState, TextSelection};

#[derive(Clone, Debug, Default)]
pub struct TextInputManager {
    focused: Option<String>,
    drag_anchor: Option<(String, usize)>,
    fallback_clipboard: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextInputAction {
    pub handled: bool,
    pub changed: bool,
}

pub trait TextInputResolver {
    fn input_mut(&mut self, id: &str) -> Option<&mut TextInputState>;
}

impl TextInputManager {
    pub fn focused(&self) -> Option<&str> {
        self.focused.as_deref()
    }

    pub fn is_focused(&self, id: &str) -> bool {
        self.focused.as_deref() == Some(id)
    }

    pub fn focus(&mut self, id: impl Into<String>) {
        self.focused = Some(id.into());
        self.drag_anchor = None;
    }

    pub fn focus_input(&mut self, id: impl Into<String>, inputs: &mut impl TextInputResolver) {
        let id = id.into();
        self.set_focus_with_inputs(&id, inputs, true);
    }

    pub fn clear_focus(&mut self) {
        self.focused = None;
        self.drag_anchor = None;
    }

    pub fn handle_action(
        &mut self,
        action_id: &str,
        inputs: &mut impl TextInputResolver,
    ) -> TextInputAction {
        if let Some(id) = action_id.strip_prefix("__stuk.input.focus.") {
            self.set_focus_with_inputs(id, inputs, true);
            return TextInputAction {
                handled: true,
                changed: false,
            };
        }

        if let Some((id, caret)) = parse_caret_action(action_id, "__stuk.input.caret_down.") {
            self.set_focus_with_inputs(&id, inputs, true);
            self.drag_anchor = Some((id.clone(), caret));
            set_selection(inputs, &id, caret, caret);
            return TextInputAction {
                handled: true,
                changed: false,
            };
        }

        if let Some((id, caret)) = parse_caret_action(action_id, "__stuk.input.caret_drag.") {
            self.set_focus_with_inputs(&id, inputs, false);
            let anchor = self
                .drag_anchor
                .as_ref()
                .filter(|(field_id, _)| field_id == &id)
                .map(|(_, anchor)| *anchor)
                .unwrap_or(caret);
            set_selection(inputs, &id, anchor, caret);
            return TextInputAction {
                handled: true,
                changed: false,
            };
        }

        if action_id.starts_with("__stuk.input.caret_up.") {
            self.drag_anchor = None;
            return TextInputAction {
                handled: true,
                changed: false,
            };
        }

        if let Some((id, caret)) = parse_caret_action(action_id, "__stuk.input.word.") {
            self.set_focus_with_inputs(&id, inputs, true);
            if let Some(input) = inputs.input_mut(&id) {
                input.select_word_at(caret);
            }
            return TextInputAction {
                handled: true,
                changed: false,
            };
        }

        let Some(focused) = self.focused.clone() else {
            return TextInputAction::default();
        };
        let Some(input) = inputs.input_mut(&focused) else {
            return TextInputAction::default();
        };

        let changed = match action_id {
            "input.key.Backspace" => input.delete_backward(),
            "input.key.Delete" => input.delete_forward(),
            "input.key.Enter" => input.insert_text("\n"),
            "input.key.ArrowLeft" | "input.move.left" => {
                input.move_left(false);
                false
            }
            "input.move.left.select" => {
                input.move_left(true);
                false
            }
            "input.key.ArrowRight" | "input.move.right" => {
                input.move_right(false);
                false
            }
            "input.move.right.select" => {
                input.move_right(true);
                false
            }
            "input.key.Home" | "input.move.line_start" => {
                input.move_to_line_start(false);
                false
            }
            "input.move.line_start.select" => {
                input.move_to_line_start(true);
                false
            }
            "input.key.End" | "input.move.line_end" => {
                input.move_to_line_end(false);
                false
            }
            "input.move.line_end.select" => {
                input.move_to_line_end(true);
                false
            }
            "input.move.word_left" => {
                input.move_word_left(false);
                false
            }
            "input.move.word_left.select" => {
                input.move_word_left(true);
                false
            }
            "input.move.word_right" => {
                input.move_word_right(false);
                false
            }
            "input.move.word_right.select" => {
                input.move_word_right(true);
                false
            }
            "input.move.start" => {
                input.move_to_start(false);
                false
            }
            "input.move.start.select" => {
                input.move_to_start(true);
                false
            }
            "input.move.end" => {
                input.move_to_end(false);
                false
            }
            "input.move.end.select" => {
                input.move_to_end(true);
                false
            }
            "input.edit.select_all" => {
                input.select_all();
                false
            }
            "input.edit.copy" => {
                if let Some(value) = input.copy_selection() {
                    let _ = write_clipboard_text(&value);
                    self.fallback_clipboard = value;
                }
                false
            }
            "input.edit.cut" => {
                if let Some(value) = input.cut_selection() {
                    let _ = write_clipboard_text(&value);
                    self.fallback_clipboard = value;
                    true
                } else {
                    false
                }
            }
            "input.edit.paste" => {
                let value =
                    read_clipboard_text().unwrap_or_else(|| self.fallback_clipboard.clone());
                input.paste_text(&value)
            }
            action if action.starts_with("input.edit.paste.") => input.paste_text(
                &decode_action_text(action.trim_start_matches("input.edit.paste.")),
            ),
            action if action.starts_with("input.key.") => {
                let value = action.trim_start_matches("input.key.");
                if value.chars().count() == 1 {
                    input.insert_text(value)
                } else {
                    return TextInputAction::default();
                }
            }
            _ => return TextInputAction::default(),
        };

        TextInputAction {
            handled: true,
            changed,
        }
    }

    fn set_focus_with_inputs(
        &mut self,
        id: &str,
        inputs: &mut impl TextInputResolver,
        clear_drag_anchor: bool,
    ) {
        if self.focused.as_deref() != Some(id)
            && let Some(previous) = self.focused.clone()
            && let Some(input) = inputs.input_mut(&previous)
        {
            let focus = input.selection().focus;
            input.set_selection(TextSelection::caret(focus));
        }
        self.focused = Some(id.to_string());
        if clear_drag_anchor {
            self.drag_anchor = None;
        }
    }
}

fn set_selection(inputs: &mut impl TextInputResolver, id: &str, anchor: usize, focus: usize) {
    if let Some(input) = inputs.input_mut(id) {
        input.set_selection(TextSelection::new(anchor, focus));
    }
}

fn parse_caret_action(action: &str, prefix: &str) -> Option<(String, usize)> {
    let value = action.strip_prefix(prefix)?;
    let (field, index) = value.rsplit_once('.')?;
    Some((field.to_string(), index.parse().ok()?))
}

fn decode_action_text(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let Ok(hex) = std::str::from_utf8(&bytes[index + 1..index + 3])
            && let Ok(byte) = u8::from_str_radix(hex, 16)
        {
            output.push(byte);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8_lossy(&output).to_string()
}
