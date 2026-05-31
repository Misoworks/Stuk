use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::{TaskHandle, spawn_task};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageCursor(pub String);

impl PageCursor {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page<T, C = PageCursor> {
    pub items: Vec<T>,
    pub next_cursor: Option<C>,
    pub total: Option<u64>,
}

impl<T, C> Page<T, C> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            next_cursor: None,
            total: None,
        }
    }

    pub fn next_cursor(mut self, cursor: impl Into<Option<C>>) -> Self {
        self.next_cursor = cursor.into();
        self
    }

    pub fn total(mut self, total: u64) -> Self {
        self.total = Some(total);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaginationMode {
    Cursor,
    Offset,
    PageNumber,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaginatedResourcePhase {
    InitialLoading,
    Loaded,
    Empty,
    LoadingMore,
    Refreshing,
    ErrorInitial,
    ErrorNextPage,
    EndReached,
    Stale,
}

impl PaginatedResourcePhase {
    pub fn is_loading(self) -> bool {
        matches!(
            self,
            Self::InitialLoading | Self::LoadingMore | Self::Refreshing
        )
    }

    pub fn is_error(self) -> bool {
        matches!(self, Self::ErrorInitial | Self::ErrorNextPage)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaginatedResourceSnapshot<T, E, C = PageCursor> {
    pub phase: PaginatedResourcePhase,
    pub items: Vec<T>,
    pub next_cursor: Option<C>,
    pub total: Option<u64>,
    pub error: Option<E>,
    pub stale: bool,
}

impl<T, E, C> PaginatedResourceSnapshot<T, E, C> {
    pub fn is_loading(&self) -> bool {
        self.phase.is_loading()
    }

    pub fn is_error(&self) -> bool {
        self.phase.is_error()
    }

    pub fn has_next_page(&self) -> bool {
        self.next_cursor.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
            && matches!(
                self.phase,
                PaginatedResourcePhase::Empty | PaginatedResourcePhase::EndReached
            )
    }
}

type PageFuture<T, E, C> = Pin<Box<dyn Future<Output = Result<Page<T, C>, E>> + Send>>;
type PageLoader<T, E, C> = Arc<dyn Fn(Option<C>) -> PageFuture<T, E, C> + Send + Sync>;

pub struct PaginatedResource<T, E, C = PageCursor> {
    id: String,
    mode: PaginationMode,
    state: Arc<Mutex<PaginatedResourceSnapshot<T, E, C>>>,
    loader: PageLoader<T, E, C>,
    last_task: Arc<Mutex<Option<TaskHandle>>>,
    version: Arc<AtomicU64>,
}

impl<T, E, C> Clone for PaginatedResource<T, E, C> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            mode: self.mode,
            state: Arc::clone(&self.state),
            loader: Arc::clone(&self.loader),
            last_task: Arc::clone(&self.last_task),
            version: Arc::clone(&self.version),
        }
    }
}

impl<T, E, C> PaginatedResource<T, E, C> {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn mode(&self) -> PaginationMode {
        self.mode
    }

    pub fn with_snapshot<R>(
        &self,
        read: impl FnOnce(&PaginatedResourceSnapshot<T, E, C>) -> R,
    ) -> R {
        read(
            &self
                .state
                .lock()
                .expect("paginated resource state mutex poisoned"),
        )
    }

    pub fn phase(&self) -> PaginatedResourcePhase {
        self.with_snapshot(|snapshot| snapshot.phase)
    }

    pub fn is_loading(&self) -> bool {
        self.phase().is_loading()
    }

    pub fn is_initial_loading(&self) -> bool {
        self.phase() == PaginatedResourcePhase::InitialLoading
    }

    pub fn is_loading_next_page(&self) -> bool {
        self.phase() == PaginatedResourcePhase::LoadingMore
    }

    pub fn is_refreshing(&self) -> bool {
        self.phase() == PaginatedResourcePhase::Refreshing
    }

    pub fn is_error(&self) -> bool {
        self.phase().is_error()
    }

    pub fn last_task(&self) -> Option<TaskHandle> {
        self.last_task
            .lock()
            .expect("paginated resource task mutex poisoned")
            .clone()
    }
}

impl<T, E, C> PaginatedResource<T, E, C>
where
    T: Clone,
    E: Clone,
    C: Clone,
{
    pub fn snapshot(&self) -> PaginatedResourceSnapshot<T, E, C> {
        self.with_snapshot(Clone::clone)
    }

    pub fn items(&self) -> Vec<T> {
        self.with_snapshot(|snapshot| snapshot.items.clone())
    }

    pub fn error(&self) -> Option<E> {
        self.with_snapshot(|snapshot| snapshot.error.clone())
    }

    pub fn next_cursor(&self) -> Option<C> {
        self.with_snapshot(|snapshot| snapshot.next_cursor.clone())
    }

    pub fn has_next_page(&self) -> bool {
        self.with_snapshot(PaginatedResourceSnapshot::has_next_page)
    }
}

impl<T, E, C> PaginatedResource<T, E, C>
where
    T: Clone + Send + 'static,
    E: Send + 'static,
    C: Clone + Send + 'static,
{
    pub fn load_next(&self) -> Option<TaskHandle> {
        let cursor = self.with_snapshot(|snapshot| {
            if snapshot.phase.is_loading() {
                None
            } else {
                snapshot.next_cursor.clone()
            }
        })?;

        Some(self.load(Some(cursor), LoadKind::NextPage))
    }

    pub fn refresh(&self) -> TaskHandle {
        self.load(None, LoadKind::Refresh)
    }

    pub fn reset(&self) -> TaskHandle {
        self.version.fetch_add(1, Ordering::AcqRel);
        if let Some(task) = self
            .last_task
            .lock()
            .expect("paginated resource task mutex poisoned")
            .take()
        {
            task.cancel();
        }
        self.load(None, LoadKind::Initial)
    }

    fn load(&self, cursor: Option<C>, kind: LoadKind) -> TaskHandle {
        let version = self.version.fetch_add(1, Ordering::AcqRel) + 1;
        {
            let mut state = self
                .state
                .lock()
                .expect("paginated resource state mutex poisoned");
            state.error = None;
            state.stale = false;
            state.phase = match kind {
                LoadKind::Initial => PaginatedResourcePhase::InitialLoading,
                LoadKind::NextPage => PaginatedResourcePhase::LoadingMore,
                LoadKind::Refresh => PaginatedResourcePhase::Refreshing,
            };
        }

        let state = Arc::clone(&self.state);
        let loader = Arc::clone(&self.loader);
        let current_version = Arc::clone(&self.version);
        let task = spawn_task(async move {
            let result = loader(cursor).await;
            if current_version.load(Ordering::Acquire) != version {
                return;
            }
            let mut snapshot = state
                .lock()
                .expect("paginated resource state mutex poisoned");
            apply_page_result(&mut snapshot, result, kind);
        });

        if let Some(previous) = self
            .last_task
            .lock()
            .expect("paginated resource task mutex poisoned")
            .replace(task.clone())
        {
            previous.cancel();
        }
        task
    }
}

#[derive(Clone, Copy)]
enum LoadKind {
    Initial,
    NextPage,
    Refresh,
}

pub fn paginated_resource<T, E, C, F, Fut>(
    id: impl Into<String>,
    mode: PaginationMode,
    load: F,
) -> PaginatedResource<T, E, C>
where
    T: Clone + Send + 'static,
    E: Send + 'static,
    C: Clone + Send + 'static,
    F: Fn(Option<C>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Page<T, C>, E>> + Send + 'static,
{
    let resource = PaginatedResource {
        id: id.into(),
        mode,
        state: Arc::new(Mutex::new(PaginatedResourceSnapshot {
            phase: PaginatedResourcePhase::InitialLoading,
            items: Vec::new(),
            next_cursor: None,
            total: None,
            error: None,
            stale: false,
        })),
        loader: Arc::new(move |cursor| Box::pin(load(cursor))),
        last_task: Arc::new(Mutex::new(None)),
        version: Arc::new(AtomicU64::new(0)),
    };
    resource.load(None, LoadKind::Initial);
    resource
}

pub fn cursor_resource<T, E, F, Fut>(
    id: impl Into<String>,
    load: F,
) -> PaginatedResource<T, E, PageCursor>
where
    T: Clone + Send + 'static,
    E: Send + 'static,
    F: Fn(Option<PageCursor>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Page<T, PageCursor>, E>> + Send + 'static,
{
    paginated_resource(id, PaginationMode::Cursor, load)
}

fn apply_page_result<T, E, C>(
    snapshot: &mut PaginatedResourceSnapshot<T, E, C>,
    result: Result<Page<T, C>, E>,
    kind: LoadKind,
) {
    match result {
        Ok(page) => {
            match kind {
                LoadKind::Initial | LoadKind::Refresh => snapshot.items = page.items,
                LoadKind::NextPage => snapshot.items.extend(page.items),
            }
            snapshot.next_cursor = page.next_cursor;
            snapshot.total = page.total;
            snapshot.error = None;
            snapshot.stale = false;
            snapshot.phase = if snapshot.items.is_empty() {
                PaginatedResourcePhase::Empty
            } else if snapshot.next_cursor.is_none() {
                PaginatedResourcePhase::EndReached
            } else {
                PaginatedResourcePhase::Loaded
            };
        }
        Err(error) => {
            snapshot.error = Some(error);
            snapshot.stale = matches!(kind, LoadKind::Refresh | LoadKind::NextPage);
            snapshot.phase = if snapshot.items.is_empty() {
                PaginatedResourcePhase::ErrorInitial
            } else {
                PaginatedResourcePhase::ErrorNextPage
            };
        }
    }
}

pub trait PaginationCxExt {
    fn paginated_resource<T, E, C, F, Fut>(
        &self,
        id: impl Into<String>,
        mode: PaginationMode,
        load: F,
    ) -> PaginatedResource<T, E, C>
    where
        T: Clone + Send + 'static,
        E: Send + 'static,
        C: Clone + Send + 'static,
        F: Fn(Option<C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Page<T, C>, E>> + Send + 'static;

    fn cursor_resource<T, E, F, Fut>(
        &self,
        id: impl Into<String>,
        load: F,
    ) -> PaginatedResource<T, E, PageCursor>
    where
        T: Clone + Send + 'static,
        E: Send + 'static,
        F: Fn(Option<PageCursor>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Page<T, PageCursor>, E>> + Send + 'static;
}

impl PaginationCxExt for crate::Cx {
    fn paginated_resource<T, E, C, F, Fut>(
        &self,
        id: impl Into<String>,
        mode: PaginationMode,
        load: F,
    ) -> PaginatedResource<T, E, C>
    where
        T: Clone + Send + 'static,
        E: Send + 'static,
        C: Clone + Send + 'static,
        F: Fn(Option<C>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Page<T, C>, E>> + Send + 'static,
    {
        paginated_resource(id, mode, load)
    }

    fn cursor_resource<T, E, F, Fut>(
        &self,
        id: impl Into<String>,
        load: F,
    ) -> PaginatedResource<T, E, PageCursor>
    where
        T: Clone + Send + 'static,
        E: Send + 'static,
        F: Fn(Option<PageCursor>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Page<T, PageCursor>, E>> + Send + 'static,
    {
        cursor_resource(id, load)
    }
}
