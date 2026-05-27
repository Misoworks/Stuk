#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitHint {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StaccatoSession {
    pub tab_title: Option<String>,
    pub document_id: Option<String>,
    pub restore_payload: Option<String>,
    pub preferred_split: Option<SplitHint>,
}

impl StaccatoSession {
    pub fn set_tab_title(&mut self, title: impl Into<String>) {
        self.tab_title = Some(title.into());
    }

    pub fn set_document_id(&mut self, document_id: impl Into<String>) {
        self.document_id = Some(document_id.into());
    }

    pub fn set_restore_payload(&mut self, payload: impl Into<String>) {
        self.restore_payload = Some(payload.into());
    }

    pub fn set_preferred_split(&mut self, split: SplitHint) {
        self.preferred_split = Some(split);
    }
}
