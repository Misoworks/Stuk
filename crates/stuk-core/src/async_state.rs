use std::{
    convert::Infallible,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::{TaskHandle, spawn_task};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceState<T, E> {
    Loading,
    Ready(T),
    Error(E),
}

impl<T, E> ResourceState<T, E> {
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    pub fn data(&self) -> Option<&T> {
        match self {
            Self::Ready(data) => Some(data),
            Self::Loading | Self::Error(_) => None,
        }
    }

    pub fn error(&self) -> Option<&E> {
        match self {
            Self::Error(error) => Some(error),
            Self::Loading | Self::Ready(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct Resource<T, E> {
    id: String,
    state: Arc<Mutex<ResourceState<T, E>>>,
    task: TaskHandle,
}

impl<T, E> Clone for Resource<T, E> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            state: Arc::clone(&self.state),
            task: self.task.clone(),
        }
    }
}

impl<T, E> Resource<T, E> {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn task(&self) -> TaskHandle {
        self.task.clone()
    }

    pub fn with_state<R>(&self, read: impl FnOnce(&ResourceState<T, E>) -> R) -> R {
        read(&self.state.lock().expect("resource state mutex poisoned"))
    }

    pub fn is_loading(&self) -> bool {
        self.with_state(ResourceState::is_loading)
    }

    pub fn is_ready(&self) -> bool {
        self.with_state(ResourceState::is_ready)
    }

    pub fn is_error(&self) -> bool {
        self.with_state(ResourceState::is_error)
    }
}

impl<T: Clone, E: Clone> Resource<T, E> {
    pub fn state(&self) -> ResourceState<T, E> {
        self.with_state(Clone::clone)
    }

    pub fn data(&self) -> Option<T> {
        self.with_state(|state| state.data().cloned())
    }

    pub fn error(&self) -> Option<E> {
        self.with_state(|state| state.error().cloned())
    }
}

pub fn resource<T, E, F, Fut>(id: impl Into<String>, load: F) -> Resource<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = std::result::Result<T, E>> + Send + 'static,
{
    let state = Arc::new(Mutex::new(ResourceState::Loading));
    let state_for_task = Arc::clone(&state);
    let task = spawn_task(async move {
        let next = match load().await {
            Ok(data) => ResourceState::Ready(data),
            Err(error) => ResourceState::Error(error),
        };
        *state_for_task
            .lock()
            .expect("resource state mutex poisoned") = next;
    });

    Resource {
        id: id.into(),
        state,
        task,
    }
}

pub fn resource_value<T, F, Fut>(id: impl Into<String>, load: F) -> Resource<T, Infallible>
where
    T: Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = T> + Send + 'static,
{
    resource(id, || async move { Ok::<_, Infallible>(load().await) })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MutationState<T, E> {
    Idle,
    Pending,
    Success(T),
    Error(E),
}

impl<T, E> MutationState<T, E> {
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    pub fn data(&self) -> Option<&T> {
        match self {
            Self::Success(data) => Some(data),
            Self::Idle | Self::Pending | Self::Error(_) => None,
        }
    }

    pub fn error(&self) -> Option<&E> {
        match self {
            Self::Error(error) => Some(error),
            Self::Idle | Self::Pending | Self::Success(_) => None,
        }
    }
}

type MutationFuture<T, E> = Pin<Box<dyn Future<Output = std::result::Result<T, E>> + Send>>;
type MutationRunner<I, T, E> = Arc<dyn Fn(I) -> MutationFuture<T, E> + Send + Sync>;

pub struct Mutation<I, T, E> {
    id: String,
    state: Arc<Mutex<MutationState<T, E>>>,
    runner: MutationRunner<I, T, E>,
    last_task: Arc<Mutex<Option<TaskHandle>>>,
    version: Arc<AtomicU64>,
    _input: PhantomData<fn(I)>,
}

impl<I, T, E> Clone for Mutation<I, T, E> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            state: Arc::clone(&self.state),
            runner: Arc::clone(&self.runner),
            last_task: Arc::clone(&self.last_task),
            version: Arc::clone(&self.version),
            _input: PhantomData,
        }
    }
}

impl<I, T, E> Mutation<I, T, E> {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn with_state<R>(&self, read: impl FnOnce(&MutationState<T, E>) -> R) -> R {
        read(&self.state.lock().expect("mutation state mutex poisoned"))
    }

    pub fn is_idle(&self) -> bool {
        self.with_state(MutationState::is_idle)
    }

    pub fn is_pending(&self) -> bool {
        self.with_state(MutationState::is_pending)
    }

    pub fn is_success(&self) -> bool {
        self.with_state(MutationState::is_success)
    }

    pub fn is_error(&self) -> bool {
        self.with_state(MutationState::is_error)
    }

    pub fn last_task(&self) -> Option<TaskHandle> {
        self.last_task
            .lock()
            .expect("mutation task mutex poisoned")
            .clone()
    }

    pub fn reset(&self) {
        self.version.fetch_add(1, Ordering::AcqRel);
        if let Some(task) = self
            .last_task
            .lock()
            .expect("mutation task mutex poisoned")
            .take()
        {
            task.cancel();
        }
        *self.state.lock().expect("mutation state mutex poisoned") = MutationState::Idle;
    }
}

impl<I, T, E> Mutation<I, T, E>
where
    I: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
{
    pub fn submit(&self, input: I) -> TaskHandle {
        let version = self.version.fetch_add(1, Ordering::AcqRel) + 1;
        *self.state.lock().expect("mutation state mutex poisoned") = MutationState::Pending;
        let state = Arc::clone(&self.state);
        let runner = Arc::clone(&self.runner);
        let current_version = Arc::clone(&self.version);
        let task = spawn_task(async move {
            let next = match runner(input).await {
                Ok(data) => MutationState::Success(data),
                Err(error) => MutationState::Error(error),
            };
            if current_version.load(Ordering::Acquire) == version {
                *state.lock().expect("mutation state mutex poisoned") = next;
            }
        });

        if let Some(previous) = self
            .last_task
            .lock()
            .expect("mutation task mutex poisoned")
            .replace(task.clone())
        {
            previous.cancel();
        }
        task
    }
}

impl<I, T, E> Mutation<I, T, E>
where
    I: Send + 'static,
    T: Send + Clone + 'static,
    E: Send + Clone + 'static,
{
    pub fn state(&self) -> MutationState<T, E> {
        self.with_state(Clone::clone)
    }

    pub fn data(&self) -> Option<T> {
        self.with_state(|state| state.data().cloned())
    }

    pub fn error(&self) -> Option<E> {
        self.with_state(|state| state.error().cloned())
    }
}

pub fn mutation<I, T, E, F, Fut>(id: impl Into<String>, run: F) -> Mutation<I, T, E>
where
    I: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
    F: Fn(I) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = std::result::Result<T, E>> + Send + 'static,
{
    Mutation {
        id: id.into(),
        state: Arc::new(Mutex::new(MutationState::Idle)),
        runner: Arc::new(move |input| Box::pin(run(input))),
        last_task: Arc::new(Mutex::new(None)),
        version: Arc::new(AtomicU64::new(0)),
        _input: PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };

    #[test]
    fn resource_tracks_loading_and_ready_states() {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let notes = resource_value("notes.load", move || async move {
            started_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(1)).unwrap();
            vec!["Inbox".to_string()]
        });

        assert_eq!(notes.id(), "notes.load");
        started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(notes.is_loading());
        release_tx.send(()).unwrap();
        wait_until(|| notes.is_ready());

        assert_eq!(notes.data(), Some(vec!["Inbox".to_string()]));
    }

    #[test]
    fn resource_tracks_error_states() {
        let notes = resource("notes.load", || async {
            Err::<Vec<String>, _>("offline".to_string())
        });

        wait_until(|| notes.is_error());

        assert_eq!(notes.error(), Some("offline".to_string()));
    }

    #[test]
    fn mutation_tracks_latest_submission() {
        let save = mutation("notes.save", |value: u32| async move {
            if value == 1 {
                thread::sleep(Duration::from_millis(25));
            }
            Ok::<_, String>(value)
        });

        assert!(save.is_idle());
        save.submit(1);
        let latest = save.submit(2);

        wait_until(|| latest.is_finished());
        wait_until(|| save.is_success());

        assert_eq!(save.data(), Some(2));
        assert!(save.last_task().is_some());
    }

    #[test]
    fn mutation_reset_cancels_pending_task_and_clears_state() {
        let save = mutation("notes.save", |value: u32| async move {
            thread::sleep(Duration::from_millis(50));
            Ok::<_, String>(value)
        });

        let task = save.submit(1);
        assert!(save.is_pending());
        save.reset();

        assert!(task.is_cancelled());
        assert_eq!(save.state(), MutationState::Idle);
    }

    fn wait_until(mut done: impl FnMut() -> bool) {
        let start = Instant::now();
        while !done() && start.elapsed() < Duration::from_secs(1) {
            thread::sleep(Duration::from_millis(1));
        }
        assert!(done());
    }
}
