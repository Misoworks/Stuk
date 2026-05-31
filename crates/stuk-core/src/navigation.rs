use std::fmt;

use crate::{Cx, Element};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageId(String);

impl PageId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for PageId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for PageId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for PageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

pub trait Screen {
    fn id(&self) -> PageId;
    fn title(&self) -> String;
    fn view(&self, cx: &mut Cx) -> Element;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteState<R> {
    route: R,
}

impl<R> RouteState<R> {
    pub fn new(route: R) -> Self {
        Self { route }
    }

    pub fn route(&self) -> &R {
        &self.route
    }

    pub fn set(&mut self, route: R) {
        self.route = route;
    }

    pub fn replace(&mut self, route: R) -> R {
        std::mem::replace(&mut self.route, route)
    }

    pub fn map<T>(&self, map: impl FnOnce(&R) -> T) -> T {
        map(&self.route)
    }
}

impl<R: Clone> RouteState<R> {
    pub fn current(&self) -> R {
        self.route.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationStack<R> {
    entries: Vec<R>,
}

impl<R> NavigationStack<R> {
    pub fn new(root: R) -> Self {
        Self {
            entries: vec![root],
        }
    }

    pub fn from_entries(entries: impl IntoIterator<Item = R>) -> Option<Self> {
        let entries = entries.into_iter().collect::<Vec<_>>();
        (!entries.is_empty()).then_some(Self { entries })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn can_go_back(&self) -> bool {
        self.entries.len() > 1
    }

    pub fn entries(&self) -> &[R] {
        &self.entries
    }

    pub fn push(&mut self, route: R) {
        self.entries.push(route);
    }

    pub fn replace(&mut self, route: R) -> Option<R> {
        self.entries
            .last_mut()
            .map(|current| std::mem::replace(current, route))
    }

    pub fn pop(&mut self) -> Option<R> {
        if self.can_go_back() {
            self.entries.pop()
        } else {
            None
        }
    }

    pub fn clear_to_root(&mut self) {
        self.entries.truncate(1);
    }
}

impl<R: Clone> NavigationStack<R> {
    pub fn current(&self) -> R {
        self.entries
            .last()
            .expect("navigation stack always has a root")
            .clone()
    }

    pub fn root(&self) -> R {
        self.entries
            .first()
            .expect("navigation stack always has a root")
            .clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationSplitState<S, D> {
    sidebar: S,
    detail: D,
}

impl<S, D> NavigationSplitState<S, D> {
    pub fn new(sidebar: S, detail: D) -> Self {
        Self { sidebar, detail }
    }

    pub fn sidebar(&self) -> &S {
        &self.sidebar
    }

    pub fn detail(&self) -> &D {
        &self.detail
    }

    pub fn set_sidebar(&mut self, route: S) {
        self.sidebar = route;
    }

    pub fn set_detail(&mut self, route: D) {
        self.detail = route;
    }
}
