//! Serialized latest-per-key worker queue.
//!
//! Port of packages/session-ui/src/components/markdown-worker-queue.ts
//! behaviour (upstream 18ef3cc).

use crate::markdown_worker_transport::WorkerRequest;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

/// A boxed, sendable future produced by the queue's `run` callback.
pub type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

struct QueueKeyState<R> {
    active: Option<R>,
    running: bool,
    queued: Option<R>,
    owner: u64,
    cancelled: bool,
}

impl<R> Default for QueueKeyState<R> {
    fn default() -> Self {
        QueueKeyState {
            active: None,
            running: false,
            queued: None,
            owner: 0,
            cancelled: false,
        }
    }
}

struct QueueInner<R> {
    states: Mutex<HashMap<String, QueueKeyState<R>>>,
    notify: Notify,
}

/// A queue that runs one request per key at a time, keeping only the latest.
pub struct LatestWorkerQueue<R> {
    inner: Arc<QueueInner<R>>,
    run: Arc<dyn Fn(R) -> BoxFuture + Send + Sync>,
    supersede: Arc<dyn Fn(&R) + Send + Sync>,
    dispose: Arc<dyn Fn(&str) + Send + Sync>,
}

/// Build a queue from `run`, `supersede`, and `dispose` callbacks.
pub fn create_latest_worker_queue<R, F, S, D>(
    run: F,
    supersede: S,
    dispose: D,
) -> LatestWorkerQueue<R>
where
    R: WorkerRequest + Send + 'static,
    F: Fn(R) -> BoxFuture + Send + Sync + 'static,
    S: Fn(&R) + Send + Sync + 'static,
    D: Fn(&str) + Send + Sync + 'static,
{
    LatestWorkerQueue {
        inner: Arc::new(QueueInner {
            states: Mutex::new(HashMap::new()),
            notify: Notify::new(),
        }),
        run: Arc::new(run),
        supersede: Arc::new(supersede),
        dispose: Arc::new(dispose),
    }
}

enum Step<R> {
    Run(R),
    Stop,
}

enum StartAction<R> {
    Start,
    Supersede(R),
    None,
}

impl<R: WorkerRequest + Send + 'static> LatestWorkerQueue<R> {
    /// Enqueue a request, superseding any older queued request for its key.
    pub fn highlight(&self, request: R) {
        let key = request.key().to_string();
        let action = {
            let mut states = self.inner.states.lock().unwrap();
            let state = states.entry(key.clone()).or_default();
            state.cancelled = false;
            if !state.running && state.active.is_none() {
                state.active = Some(request);
                StartAction::Start
            } else if let Some(previous) = state.queued.replace(request) {
                StartAction::Supersede(previous)
            } else {
                StartAction::None
            }
        };
        match action {
            StartAction::Start => self.spawn(&key),
            StartAction::Supersede(previous) => (self.supersede)(&previous),
            StartAction::None => {}
        }
        self.inner.notify.notify_waiters();
    }

    /// Dispose a key, superseding any pending request.
    pub fn dispose(&self, key: &str) {
        let (active, queued) = {
            let mut states = self.inner.states.lock().unwrap();
            match states.get_mut(key) {
                Some(state) => {
                    let active = state.active.take();
                    let queued = state.queued.take();
                    state.cancelled = true;
                    state.running = false;
                    (active, queued)
                }
                None => (None, None),
            }
        };
        if let Some(active) = active {
            (self.supersede)(&active);
        }
        if let Some(queued) = queued {
            (self.supersede)(&queued);
        }
        (self.dispose)(key);
        self.inner.notify.notify_waiters();
    }

    /// The number of keys with a queued request.
    pub fn pending(&self) -> usize {
        let states = self.inner.states.lock().unwrap();
        states
            .values()
            .filter(|state| state.queued.is_some())
            .count()
    }

    /// Wait until every key is idle.
    pub async fn idle(&self) {
        loop {
            let notified = self.inner.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.is_idle() {
                return;
            }
            notified.await;
        }
    }

    fn is_idle(&self) -> bool {
        let states = self.inner.states.lock().unwrap();
        states
            .values()
            .all(|state| !state.running && state.active.is_none() && state.queued.is_none())
    }

    fn spawn(&self, key: &str) {
        let token = {
            let mut states = self.inner.states.lock().unwrap();
            let state = states.entry(key.to_string()).or_default();
            state.owner += 1;
            state.owner
        };
        let inner = self.inner.clone();
        let run = self.run.clone();
        let key = key.to_string();
        tokio::spawn(async move {
            loop {
                let step = {
                    let mut states = inner.states.lock().unwrap();
                    let Some(state) = states.get_mut(&key) else {
                        return;
                    };
                    if state.owner != token {
                        return;
                    }
                    if state.cancelled {
                        state.cancelled = false;
                        state.running = false;
                        state.active = None;
                        return;
                    }
                    if state.running {
                        return;
                    }
                    match state.active.take() {
                        Some(request) => {
                            state.running = true;
                            Step::Run(request)
                        }
                        None => Step::Stop,
                    }
                };
                match step {
                    Step::Run(request) => run(request).await,
                    Step::Stop => break,
                }
                {
                    let mut states = inner.states.lock().unwrap();
                    let Some(state) = states.get_mut(&key) else {
                        return;
                    };
                    if state.owner != token {
                        return;
                    }
                    state.running = false;
                    if state.active.is_none() {
                        if let Some(next) = state.queued.take() {
                            state.active = Some(next);
                        }
                    }
                }
                inner.notify.notify_waiters();
            }
            inner.notify.notify_waiters();
        });
    }
}
