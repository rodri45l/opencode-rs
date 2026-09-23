//! Latest-per-key worker transport.
//!
//! Port of packages/session-ui/src/components/markdown-worker-transport.ts
//! behaviour (upstream 18ef3cc).

use std::collections::HashMap;
use std::sync::Mutex;

/// A worker request identified by a stable key.
pub trait WorkerRequest {
    fn key(&self) -> &str;
    fn id(&self) -> u64;
}

struct TransportKey<R> {
    inflight: Option<u64>,
    queued: Option<R>,
}

enum Action<R> {
    Post(R),
    Supersede(R),
    None,
}

/// A transport that keeps at most one queued request per key.
pub struct WorkerTransport<R> {
    state: Mutex<HashMap<String, TransportKey<R>>>,
    post: Box<dyn Fn(&R) + Send + Sync>,
    supersede: Box<dyn Fn(&R) + Send + Sync>,
}

/// Build a transport from `post` and `supersede` callbacks.
pub fn create_worker_transport<R, P, S>(post: P, supersede: S) -> WorkerTransport<R>
where
    R: WorkerRequest,
    P: Fn(&R) + Send + Sync + 'static,
    S: Fn(&R) + Send + Sync + 'static,
{
    WorkerTransport {
        state: Mutex::new(HashMap::new()),
        post: Box::new(post),
        supersede: Box::new(supersede),
    }
}

impl<R: WorkerRequest> WorkerTransport<R> {
    /// Send a request, superseding any older queued request for its key.
    pub fn send(&self, request: R) {
        let key = request.key().to_string();
        let action = {
            let mut state = self.state.lock().unwrap();
            let entry = state.entry(key).or_insert_with(|| TransportKey {
                inflight: None,
                queued: None,
            });
            if entry.inflight.is_none() {
                entry.inflight = Some(request.id());
                Action::Post(request)
            } else if let Some(previous) = entry.queued.replace(request) {
                Action::Supersede(previous)
            } else {
                Action::None
            }
        };
        match action {
            Action::Post(request) => (self.post)(&request),
            Action::Supersede(previous) => (self.supersede)(&previous),
            Action::None => {}
        }
    }

    /// Complete the in-flight request and post the queued successor.
    pub fn complete(&self, key: &str, id: u64) {
        let mut state = self.state.lock().unwrap();
        if let Some(entry) = state.get_mut(key) {
            if entry.inflight == Some(id) {
                entry.inflight = None;
                if let Some(next) = entry.queued.take() {
                    entry.inflight = Some(next.id());
                    drop(state);
                    (self.post)(&next);
                }
            }
        }
    }

    /// Drop any queued request for a key and clear its in-flight marker.
    pub fn dispose(&self, key: &str) {
        let queued = {
            let mut state = self.state.lock().unwrap();
            state.remove(key).and_then(|mut entry| entry.queued.take())
        };
        if let Some(queued) = queued {
            (self.supersede)(&queued);
        }
    }

    /// The number of keys with a queued request.
    pub fn queued(&self) -> usize {
        let state = self.state.lock().unwrap();
        state
            .values()
            .filter(|entry| entry.queued.is_some())
            .count()
    }
}
