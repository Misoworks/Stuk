use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
};

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[derive(Clone, Debug)]
pub struct TaskHandle {
    id: u64,
    finished: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    owners: Arc<()>,
}

impl TaskHandle {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Acquire)
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub fn cancellation_token(&self) -> CancellationToken {
        CancellationToken {
            cancelled: Arc::clone(&self.cancelled),
        }
    }
}

impl Drop for TaskHandle {
    fn drop(&mut self) {
        if Arc::strong_count(&self.owners) == 1 {
            self.cancel();
        }
    }
}

pub fn spawn_task(future: impl Future<Output = ()> + Send + 'static) -> TaskHandle {
    spawn_cancellable_task(|_| future)
}

pub fn spawn_cancellable_task<F, Fut>(task: F) -> TaskHandle
where
    F: FnOnce(CancellationToken) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let cancelled = Arc::new(AtomicBool::new(false));
    let handle = TaskHandle {
        id: NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed),
        finished: Arc::new(AtomicBool::new(false)),
        cancelled: Arc::clone(&cancelled),
        owners: Arc::new(()),
    };
    let finished = Arc::clone(&handle.finished);
    let token = CancellationToken { cancelled };

    thread::spawn(move || {
        pollster::block_on(task(token));
        finished.store(true, Ordering::Release);
    });

    handle
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::mpsc,
        time::{Duration, Instant},
    };

    #[test]
    fn spawn_task_runs_future_and_marks_completion() {
        let (tx, rx) = mpsc::channel();
        let handle = spawn_task(async move {
            tx.send("done").unwrap();
        });

        assert!(handle.id() > 0);
        assert_eq!(rx.recv_timeout(Duration::from_secs(1)).unwrap(), "done");

        let start = Instant::now();
        while !handle.is_finished() && start.elapsed() < Duration::from_secs(1) {
            thread::sleep(Duration::from_millis(1));
        }

        assert!(handle.is_finished());
    }

    #[test]
    fn cancellable_task_exposes_cooperative_token() {
        let (tx, rx) = mpsc::channel();
        let handle = spawn_cancellable_task(move |token| async move {
            tx.send(token.clone()).unwrap();
            while !token.is_cancelled() {
                thread::sleep(Duration::from_millis(1));
            }
        });

        let token = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(!token.is_cancelled());

        handle.cancel();

        let start = Instant::now();
        while !handle.is_finished() && start.elapsed() < Duration::from_secs(1) {
            thread::sleep(Duration::from_millis(1));
        }

        assert!(token.is_cancelled());
        assert!(handle.is_cancelled());
        assert!(handle.is_finished());
    }

    #[test]
    fn dropping_last_handle_cancels_owned_task() {
        let (tx, rx) = mpsc::channel();
        let handle = spawn_cancellable_task(move |token| async move {
            tx.send(token.clone()).unwrap();
            while !token.is_cancelled() {
                thread::sleep(Duration::from_millis(1));
            }
        });

        let token = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let clone = handle.clone();
        drop(handle);
        assert!(!token.is_cancelled());
        drop(clone);

        let start = Instant::now();
        while !token.is_cancelled() && start.elapsed() < Duration::from_secs(1) {
            thread::sleep(Duration::from_millis(1));
        }

        assert!(token.is_cancelled());
    }
}
