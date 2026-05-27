use std::{cell::RefCell, rc::Rc};

use stuk_platform::{SplitHint, StaccatoSession};

#[derive(Clone, Debug)]
pub struct StaccatoCx {
    session: Rc<RefCell<StaccatoSession>>,
}

impl StaccatoCx {
    pub(crate) fn new(session: Rc<RefCell<StaccatoSession>>) -> Self {
        Self { session }
    }

    pub fn set_tab_title(&self, title: impl Into<String>) {
        self.session.borrow_mut().set_tab_title(title);
    }

    pub fn set_document_id(&self, document_id: impl Into<String>) {
        self.session.borrow_mut().set_document_id(document_id);
    }

    pub fn set_restore_payload(&self, payload: impl Into<String>) {
        self.session.borrow_mut().set_restore_payload(payload);
    }

    pub fn set_preferred_split(&self, split: SplitHint) {
        self.session.borrow_mut().set_preferred_split(split);
    }

    pub fn snapshot(&self) -> StaccatoSession {
        self.session.borrow().clone()
    }
}

#[derive(Clone, Debug)]
pub struct SessionCx {
    session: Rc<RefCell<StaccatoSession>>,
}

impl SessionCx {
    pub(crate) fn new(session: Rc<RefCell<StaccatoSession>>) -> Self {
        Self { session }
    }

    pub fn set_document_id(&self, document_id: impl Into<String>) {
        self.session.borrow_mut().set_document_id(document_id);
    }

    pub fn set_restore_payload(&self, payload: impl Into<String>) {
        self.session.borrow_mut().set_restore_payload(payload);
    }

    pub fn snapshot(&self) -> StaccatoSession {
        self.session.borrow().clone()
    }
}
