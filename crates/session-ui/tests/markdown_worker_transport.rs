//! Port of packages/session-ui/src/components/markdown-worker-transport.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/markdown-worker-transport.ts; see docs/TEST-PORT.md.

use opencode_session_ui::markdown_worker_transport::{create_worker_transport, WorkerRequest};
use std::sync::{Arc, Mutex};

struct Req {
    id: u64,
    key: String,
}

impl WorkerRequest for Req {
    fn key(&self) -> &str {
        &self.key
    }
    fn id(&self) -> u64 {
        self.id
    }
}

fn req(id: u64, key: &str) -> Req {
    Req {
        id,
        key: key.to_string(),
    }
}

#[test]
fn posts_one_request_and_retains_only_the_latest_queued_snapshot_per_key() {
    let posted = Arc::new(Mutex::new(Vec::new()));
    let superseded = Arc::new(Mutex::new(Vec::new()));
    let (p, s) = (posted.clone(), superseded.clone());
    let transport = create_worker_transport(
        move |request: &Req| p.lock().unwrap().push(request.id),
        move |request: &Req| s.lock().unwrap().push(request.id),
    );

    transport.send(req(1, "code"));
    transport.send(req(2, "code"));
    transport.send(req(3, "code"));

    assert_eq!(*posted.lock().unwrap(), vec![1]);
    assert_eq!(*superseded.lock().unwrap(), vec![2]);
    assert_eq!(transport.queued(), 1);
    transport.complete("code", 1);
    assert_eq!(*posted.lock().unwrap(), vec![1, 3]);
    assert_eq!(transport.queued(), 0);
}

#[test]
fn ignores_a_disposed_request_response_after_the_key_is_reused() {
    let posted = Arc::new(Mutex::new(Vec::new()));
    let p = posted.clone();
    let transport = create_worker_transport(
        move |request: &Req| p.lock().unwrap().push(request.id),
        |_: &Req| {},
    );

    transport.send(req(1, "code"));
    transport.dispose("code");
    transport.send(req(2, "code"));
    transport.send(req(3, "code"));
    transport.complete("code", 1);

    assert_eq!(*posted.lock().unwrap(), vec![1, 2]);
    assert_eq!(transport.queued(), 1);
    transport.complete("code", 2);
    assert_eq!(*posted.lock().unwrap(), vec![1, 2, 3]);
}

#[test]
fn drops_queued_snapshots_when_a_key_is_disposed() {
    let superseded = Arc::new(Mutex::new(Vec::new()));
    let s = superseded.clone();
    let transport = create_worker_transport(
        |_: &Req| {},
        move |request: &Req| s.lock().unwrap().push(request.id),
    );

    transport.send(req(1, "code"));
    transport.send(req(2, "code"));
    transport.dispose("code");

    assert_eq!(*superseded.lock().unwrap(), vec![2]);
    assert_eq!(transport.queued(), 0);
}
