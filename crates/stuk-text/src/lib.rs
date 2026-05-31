mod history;
mod movement;
#[cfg(test)]
mod tests;

use history::{TextEditSnapshot, TextHistory};
use movement::{
    char_len, char_to_byte_index, line_end_boundary, line_start_boundary, next_word_boundary,
    previous_word_boundary,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextSelection {
    pub anchor: usize,
    pub focus: usize,
}

impl TextSelection {
    pub fn new(anchor: usize, focus: usize) -> Self {
        Self { anchor, focus }
    }

    pub fn caret(offset: usize) -> Self {
        Self {
            anchor: offset,
            focus: offset,
        }
    }

    pub fn is_collapsed(self) -> bool {
        self.anchor == self.focus
    }

    pub fn range(self) -> TextRange {
        TextRange::new(self.anchor.min(self.focus), self.anchor.max(self.focus))
    }

    pub fn collapse_to_start(self) -> Self {
        Self::caret(self.range().start)
    }

    pub fn collapse_to_end(self) -> Self {
        Self::caret(self.range().end)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

impl TextRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: start.min(end),
            end: start.max(end),
        }
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextInputState {
    text: String,
    selection: TextSelection,
    composition: Option<TextComposition>,
    placeholder: String,
    secure: bool,
    disabled: bool,
    history: TextHistory,
}

impl TextInputState {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let selection = TextSelection::caret(char_len(&text));
        Self {
            text,
            selection,
            composition: None,
            placeholder: String::new(),
            secure: false,
            disabled: false,
            history: TextHistory::default(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn selection(&self) -> TextSelection {
        self.selection
    }

    pub fn composition(&self) -> Option<&TextComposition> {
        self.composition.as_ref()
    }

    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    pub fn is_secure(&self) -> bool {
        self.secure
    }

    pub fn set_secure(&mut self, secure: bool) {
        self.secure = secure;
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        let end = char_len(&self.text);
        self.selection = TextSelection::caret(end);
        self.composition = None;
        self.history.clear();
    }

    pub fn set_selection(&mut self, selection: TextSelection) {
        self.selection = self.clamp_selection(selection);
    }

    pub fn select_all(&mut self) {
        self.selection = TextSelection::new(0, char_len(&self.text));
    }

    pub fn select_word_at(&mut self, offset: usize) {
        let offset = offset.min(char_len(&self.text));
        let start = previous_word_boundary(&self.text, offset);
        let end = next_word_boundary(&self.text, offset);
        self.selection = if start == end {
            TextSelection::caret(offset)
        } else {
            TextSelection::new(start, end)
        };
    }

    pub fn selected_text(&self) -> &str {
        let range = self.selection.range();
        self.text_in_range(range)
    }

    pub fn copy_selection(&self) -> Option<String> {
        if self.secure || self.selection.is_collapsed() {
            return None;
        }
        Some(self.selected_text().to_string())
    }

    pub fn cut_selection(&mut self) -> Option<String> {
        if self.disabled || self.secure || self.selection.is_collapsed() {
            return None;
        }
        let selected = self.selected_text().to_string();
        self.replace_selection("");
        Some(selected)
    }

    pub fn paste_text(&mut self, text: &str) -> bool {
        self.insert_text(text)
    }

    pub fn display_text(&self) -> String {
        if self.text.is_empty() {
            return self.placeholder.clone();
        }
        if self.secure {
            return "*".repeat(char_len(&self.text));
        }
        self.text.clone()
    }

    pub fn insert_text(&mut self, text: &str) -> bool {
        if self.disabled || text.is_empty() {
            return false;
        }
        self.replace_selection(text)
    }

    pub fn delete_backward(&mut self) -> bool {
        if self.disabled {
            return false;
        }
        if !self.selection.is_collapsed() {
            return self.replace_selection("");
        }

        let caret = self.selection.focus;
        if caret == 0 {
            return false;
        }
        self.apply_replace_range(TextRange::new(caret - 1, caret), "")
    }

    pub fn delete_forward(&mut self) -> bool {
        if self.disabled {
            return false;
        }
        if !self.selection.is_collapsed() {
            return self.replace_selection("");
        }

        let caret = self.selection.focus;
        if caret >= char_len(&self.text) {
            return false;
        }
        self.apply_replace_range(TextRange::new(caret, caret + 1), "")
    }

    pub fn delete_word_backward(&mut self) -> bool {
        if self.disabled {
            return false;
        }
        if !self.selection.is_collapsed() {
            return self.replace_selection("");
        }

        let caret = self.selection.focus;
        let start = previous_word_boundary(&self.text, caret);
        if start == caret {
            return false;
        }
        self.apply_replace_range(TextRange::new(start, caret), "")
    }

    pub fn delete_word_forward(&mut self) -> bool {
        if self.disabled {
            return false;
        }
        if !self.selection.is_collapsed() {
            return self.replace_selection("");
        }

        let caret = self.selection.focus;
        let end = next_word_boundary(&self.text, caret);
        if end == caret {
            return false;
        }
        self.apply_replace_range(TextRange::new(caret, end), "")
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn undo_depth(&self) -> usize {
        self.history.undo_depth()
    }

    pub fn redo_depth(&self) -> usize {
        self.history.redo_depth()
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    pub fn undo(&mut self) -> bool {
        if self.disabled {
            return false;
        }
        let Some(snapshot) = self.history.pop_undo() else {
            return false;
        };
        let current = self.snapshot();
        self.apply_snapshot(snapshot);
        self.history.push_redo(current);
        true
    }

    pub fn redo(&mut self) -> bool {
        if self.disabled {
            return false;
        }
        let Some(snapshot) = self.history.pop_redo() else {
            return false;
        };
        let current = self.snapshot();
        self.apply_snapshot(snapshot);
        self.history.push_undo_preserving_redo(current);
        true
    }

    pub fn move_left(&mut self, extend_selection: bool) {
        if !extend_selection && !self.selection.is_collapsed() {
            self.selection = self.selection.collapse_to_start();
            return;
        }

        let focus = self.selection.focus.saturating_sub(1);
        self.move_focus(focus, extend_selection);
    }

    pub fn move_right(&mut self, extend_selection: bool) {
        if !extend_selection && !self.selection.is_collapsed() {
            self.selection = self.selection.collapse_to_end();
            return;
        }

        let focus = (self.selection.focus + 1).min(char_len(&self.text));
        self.move_focus(focus, extend_selection);
    }

    pub fn move_word_left(&mut self, extend_selection: bool) {
        if !extend_selection && !self.selection.is_collapsed() {
            self.selection = self.selection.collapse_to_start();
            return;
        }

        self.move_focus(
            previous_word_boundary(&self.text, self.selection.focus),
            extend_selection,
        );
    }

    pub fn move_word_right(&mut self, extend_selection: bool) {
        if !extend_selection && !self.selection.is_collapsed() {
            self.selection = self.selection.collapse_to_end();
            return;
        }

        self.move_focus(
            next_word_boundary(&self.text, self.selection.focus),
            extend_selection,
        );
    }

    pub fn move_to_start(&mut self, extend_selection: bool) {
        self.move_focus(0, extend_selection);
    }

    pub fn move_to_end(&mut self, extend_selection: bool) {
        self.move_focus(char_len(&self.text), extend_selection);
    }

    pub fn move_to_line_start(&mut self, extend_selection: bool) {
        self.move_focus(
            line_start_boundary(&self.text, self.selection.focus),
            extend_selection,
        );
    }

    pub fn move_to_line_end(&mut self, extend_selection: bool) {
        self.move_focus(
            line_end_boundary(&self.text, self.selection.focus),
            extend_selection,
        );
    }

    pub fn start_composition(&mut self, text: impl Into<String>) -> bool {
        if self.disabled {
            return false;
        }
        self.composition = Some(TextComposition {
            text: text.into(),
            range: self.selection.range(),
        });
        true
    }

    pub fn update_composition(&mut self, text: impl Into<String>) -> bool {
        if self.disabled {
            return false;
        }
        if let Some(composition) = &mut self.composition {
            composition.text = text.into();
        } else {
            self.start_composition(text);
        }
        true
    }

    pub fn commit_composition(&mut self) -> bool {
        if self.disabled {
            return false;
        }
        let Some(composition) = self.composition.take() else {
            return false;
        };
        self.apply_replace_range(composition.range, &composition.text);
        true
    }

    pub fn cancel_composition(&mut self) -> bool {
        self.composition.take().is_some()
    }

    fn replace_selection(&mut self, replacement: &str) -> bool {
        self.apply_replace_range(self.selection.range(), replacement)
    }

    fn apply_replace_range(&mut self, range: TextRange, replacement: &str) -> bool {
        let range = TextRange::new(
            range.start.min(char_len(&self.text)),
            range.end.min(char_len(&self.text)),
        );
        if self.text_in_range(range) == replacement {
            return false;
        }
        self.history.push_undo(self.snapshot());
        self.replace_range(range, replacement);
        true
    }

    fn replace_range(&mut self, range: TextRange, replacement: &str) {
        let start = char_to_byte_index(&self.text, range.start);
        let end = char_to_byte_index(&self.text, range.end);
        self.text.replace_range(start..end, replacement);
        self.selection = TextSelection::caret(range.start + char_len(replacement));
        self.composition = None;
    }

    fn move_focus(&mut self, focus: usize, extend_selection: bool) {
        let focus = focus.min(char_len(&self.text));
        self.selection = if extend_selection {
            TextSelection::new(self.selection.anchor, focus)
        } else {
            TextSelection::caret(focus)
        };
    }

    fn clamp_selection(&self, selection: TextSelection) -> TextSelection {
        let end = char_len(&self.text);
        TextSelection::new(selection.anchor.min(end), selection.focus.min(end))
    }

    fn text_in_range(&self, range: TextRange) -> &str {
        let start = char_to_byte_index(&self.text, range.start);
        let end = char_to_byte_index(&self.text, range.end);
        &self.text[start..end]
    }

    fn snapshot(&self) -> TextEditSnapshot {
        TextEditSnapshot::new(self.text.clone(), self.selection)
    }

    fn apply_snapshot(&mut self, snapshot: TextEditSnapshot) {
        self.text = snapshot.text;
        self.selection = self.clamp_selection(snapshot.selection);
        self.composition = None;
    }
}

impl Default for TextInputState {
    fn default() -> Self {
        Self::new("")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextComposition {
    pub text: String,
    pub range: TextRange,
}
