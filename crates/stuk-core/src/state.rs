use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::{Cx, Element};

#[derive(Debug)]
pub struct Signal<T> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T> Signal<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(SignalInner { value, revision: 0 })),
        }
    }

    pub fn with<R>(&self, read: impl FnOnce(&T) -> R) -> R {
        read(&self.inner.borrow().value)
    }

    pub fn set(&self, value: T) {
        let mut inner = self.inner.borrow_mut();
        inner.value = value;
        inner.revision += 1;
    }

    pub fn update(&self, update: impl FnOnce(&mut T)) {
        let mut inner = self.inner.borrow_mut();
        update(&mut inner.value);
        inner.revision += 1;
    }

    pub fn replace(&self, value: T) -> T {
        let mut inner = self.inner.borrow_mut();
        inner.revision += 1;
        std::mem::replace(&mut inner.value, value)
    }

    pub fn revision(&self) -> u64 {
        self.inner.borrow().revision
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<T: Clone> Signal<T> {
    pub fn get(&self) -> T {
        self.inner.borrow().value.clone()
    }
}

#[derive(Debug)]
struct SignalInner<T> {
    value: T,
    revision: u64,
}

pub fn signal<T>(value: T) -> Signal<T> {
    Signal::new(value)
}

pub trait Component {
    type State;
    type Action;

    fn init(cx: &mut Cx) -> Self::State;
    fn update(state: &mut Self::State, action: Self::Action, cx: &mut Cx);
    fn view(state: &Self::State, cx: &mut Cx) -> Element;
}

#[derive(Debug)]
pub struct ComponentState<C: Component> {
    state: C::State,
    _component: PhantomData<C>,
}

impl<C: Component> ComponentState<C> {
    pub fn init(cx: &mut Cx) -> Self {
        Self::from_state(C::init(cx))
    }

    pub fn from_state(state: C::State) -> Self {
        Self {
            state,
            _component: PhantomData,
        }
    }

    pub fn state(&self) -> &C::State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut C::State {
        &mut self.state
    }

    pub fn dispatch(&mut self, action: C::Action, cx: &mut Cx) {
        C::update(&mut self.state, action, cx);
    }

    pub fn view(&self, cx: &mut Cx) -> Element {
        C::view(&self.state, cx)
    }

    pub fn into_state(self) -> C::State {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ElementKind;
    use stuk_settings::{SettingDefinition, SettingsSchema, SettingsStore};

    struct Counter;

    impl Component for Counter {
        type State = u32;
        type Action = u32;

        fn init(_cx: &mut Cx) -> Self::State {
            1
        }

        fn update(state: &mut Self::State, action: Self::Action, _cx: &mut Cx) {
            *state += action;
        }

        fn view(state: &Self::State, _cx: &mut Cx) -> Element {
            crate::TextElement {
                text: format!("Count {state}"),
                size: 14.0,
                line_height: 20.0,
                color: stuk_style::Color::TEXT,
            }
            .into()
        }
    }

    #[test]
    fn signal_tracks_value_and_revision() {
        let name = signal("Draft".to_string());
        let clone = name.clone();

        assert!(name.ptr_eq(&clone));
        assert_eq!(name.get(), "Draft");
        assert_eq!(name.revision(), 0);

        clone.update(|value| value.push_str(" Note"));

        assert_eq!(name.get(), "Draft Note");
        assert_eq!(name.revision(), 1);
        assert_eq!(name.replace("Saved".to_string()), "Draft Note");
        assert_eq!(clone.get(), "Saved");
        assert_eq!(clone.revision(), 2);
    }

    #[test]
    fn component_state_initializes_updates_and_views() {
        let mut schema = SettingsSchema::new();
        schema
            .insert(SettingDefinition::boolean("sync.enabled", "Sync", false))
            .unwrap();
        let schema = Rc::new(schema);
        let store = Rc::new(RefCell::new(SettingsStore::from_schema(schema.as_ref())));
        let mut cx = Cx::with_settings("dev.stuk.test", "Test", schema, store);

        let mut counter = ComponentState::<Counter>::init(&mut cx);
        counter.dispatch(2, &mut cx);
        let element = counter.view(&mut cx);

        assert_eq!(*counter.state(), 3);
        assert_eq!(element.kind(), ElementKind::Text);
    }
}
