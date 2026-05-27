use crate::TextSelection;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextEditSnapshot {
    pub text: String,
    pub selection: TextSelection,
}

impl TextEditSnapshot {
    pub fn new(text: String, selection: TextSelection) -> Self {
        Self { text, selection }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextHistory {
    undo: Vec<TextEditSnapshot>,
    redo: Vec<TextEditSnapshot>,
    limit: usize,
}

impl TextHistory {
    pub fn new(limit: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            limit: limit.max(1),
        }
    }

    pub fn push_undo(&mut self, snapshot: TextEditSnapshot) {
        self.push_undo_entry(snapshot);
        self.redo.clear();
    }

    pub fn push_undo_preserving_redo(&mut self, snapshot: TextEditSnapshot) {
        self.push_undo_entry(snapshot);
    }

    fn push_undo_entry(&mut self, snapshot: TextEditSnapshot) {
        if self.undo.last() == Some(&snapshot) {
            return;
        }
        self.undo.push(snapshot);
        if self.undo.len() > self.limit {
            self.undo.remove(0);
        }
    }

    pub fn push_redo(&mut self, snapshot: TextEditSnapshot) {
        self.redo.push(snapshot);
        if self.redo.len() > self.limit {
            self.redo.remove(0);
        }
    }

    pub fn pop_undo(&mut self) -> Option<TextEditSnapshot> {
        self.undo.pop()
    }

    pub fn pop_redo(&mut self) -> Option<TextEditSnapshot> {
        self.redo.pop()
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    pub fn redo_depth(&self) -> usize {
        self.redo.len()
    }
}

impl Default for TextHistory {
    fn default() -> Self {
        Self::new(100)
    }
}
