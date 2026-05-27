use stuk_core::{Element, Mutation, MutationState, Resource, ResourceState};

use crate::{EmptyState, ErrorView, Spinner};

type EmptyPredicate<T> = Box<dyn Fn(&T) -> bool>;
type EmptyBuilder = Box<dyn Fn() -> Element>;
type DataBuilder<T> = Box<dyn Fn(&T) -> Element>;
type ErrorBuilder<E> = Box<dyn Fn(&E) -> Element>;
type StateBuilder<T> = Box<dyn Fn(&T) -> Element>;

pub struct ResourceView<T, E> {
    resource: Resource<T, E>,
    loading: Option<EmptyBuilder>,
    empty: Option<EmptyBuilder>,
    empty_when: Option<EmptyPredicate<T>>,
    error: Option<ErrorBuilder<E>>,
    data: Option<DataBuilder<T>>,
}

impl<T, E> ResourceView<T, E> {
    pub fn new(resource: Resource<T, E>) -> Self {
        Self {
            resource,
            loading: None,
            empty: None,
            empty_when: None,
            error: None,
            data: None,
        }
    }

    pub fn loading<V>(mut self, view: impl Fn() -> V + 'static) -> Self
    where
        V: Into<Element> + 'static,
    {
        self.loading = Some(Box::new(move || view().into()));
        self
    }

    pub fn empty<V>(mut self, view: impl Fn() -> V + 'static) -> Self
    where
        V: Into<Element> + 'static,
    {
        self.empty = Some(Box::new(move || view().into()));
        self
    }

    pub fn empty_when(mut self, predicate: impl Fn(&T) -> bool + 'static) -> Self {
        self.empty_when = Some(Box::new(predicate));
        self
    }

    pub fn error<V>(mut self, view: impl Fn(&E) -> V + 'static) -> Self
    where
        V: Into<Element> + 'static,
    {
        self.error = Some(Box::new(move |error| view(error).into()));
        self
    }

    pub fn data<V>(mut self, view: impl Fn(&T) -> V + 'static) -> Self
    where
        V: Into<Element> + 'static,
    {
        self.data = Some(Box::new(move |data| view(data).into()));
        self
    }
}

impl<T, E> From<ResourceView<T, E>> for Element {
    fn from(view: ResourceView<T, E>) -> Self {
        let ResourceView {
            resource,
            loading,
            empty,
            empty_when,
            error,
            data,
        } = view;

        resource.with_state(|state| match state {
            ResourceState::Loading => loading
                .as_ref()
                .map(|view| view())
                .unwrap_or_else(|| Spinner::new("Loading").into()),
            ResourceState::Error(error_value) => error
                .as_ref()
                .map(|view| view(error_value))
                .unwrap_or_else(|| ErrorView::new("Resource failed").into()),
            ResourceState::Ready(value) => {
                if empty_when.as_ref().is_some_and(|is_empty| is_empty(value)) {
                    return empty
                        .as_ref()
                        .map(|view| view())
                        .unwrap_or_else(|| EmptyState::new("No data").into());
                }

                data.as_ref()
                    .map(|view| view(value))
                    .unwrap_or(Element::Empty)
            }
        })
    }
}

pub struct MutationView<I, T, E> {
    mutation: Mutation<I, T, E>,
    idle: Option<EmptyBuilder>,
    pending: Option<EmptyBuilder>,
    success: Option<StateBuilder<T>>,
    error: Option<ErrorBuilder<E>>,
}

impl<I, T, E> MutationView<I, T, E> {
    pub fn new(mutation: Mutation<I, T, E>) -> Self {
        Self {
            mutation,
            idle: None,
            pending: None,
            success: None,
            error: None,
        }
    }

    pub fn idle<V>(mut self, view: impl Fn() -> V + 'static) -> Self
    where
        V: Into<Element> + 'static,
    {
        self.idle = Some(Box::new(move || view().into()));
        self
    }

    pub fn pending<V>(mut self, view: impl Fn() -> V + 'static) -> Self
    where
        V: Into<Element> + 'static,
    {
        self.pending = Some(Box::new(move || view().into()));
        self
    }

    pub fn success<V>(mut self, view: impl Fn(&T) -> V + 'static) -> Self
    where
        V: Into<Element> + 'static,
    {
        self.success = Some(Box::new(move |data| view(data).into()));
        self
    }

    pub fn error<V>(mut self, view: impl Fn(&E) -> V + 'static) -> Self
    where
        V: Into<Element> + 'static,
    {
        self.error = Some(Box::new(move |error| view(error).into()));
        self
    }
}

impl<I, T, E> From<MutationView<I, T, E>> for Element {
    fn from(view: MutationView<I, T, E>) -> Self {
        let MutationView {
            mutation,
            idle,
            pending,
            success,
            error,
        } = view;

        mutation.with_state(|state| match state {
            MutationState::Idle => idle.as_ref().map(|view| view()).unwrap_or(Element::Empty),
            MutationState::Pending => pending
                .as_ref()
                .map(|view| view())
                .unwrap_or_else(|| Spinner::new("Working").into()),
            MutationState::Success(value) => success
                .as_ref()
                .map(|view| view(value))
                .unwrap_or(Element::Empty),
            MutationState::Error(error_value) => error
                .as_ref()
                .map(|view| view(error_value))
                .unwrap_or_else(|| ErrorView::new("Action failed").into()),
        })
    }
}
