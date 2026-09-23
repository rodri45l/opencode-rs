//! Port of packages/session-ui/src/components/markdown-worker-queue.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/markdown-worker-queue.ts; see docs/TEST-PORT.md.

use opencode_session_ui::markdown_worker_queue::{create_latest_worker_queue, BoxFuture};
use opencode_session_ui::markdown_worker_transport::WorkerRequest;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

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

#[tokio::test]
async fn keeps_only_the_latest_queued_request_for_each_key() {
    let processed = Arc::new(Mutex::new(Vec::new()));
    let superseded = Arc::new(Mutex::new(Vec::new()));
    let gate = Arc::new(Notify::new());
    let (p, s, g) = (processed.clone(), superseded.clone(), gate.clone());

    let queue = create_latest_worker_queue(
        move |request: Req| -> BoxFuture {
            let p = p.clone();
            let g = g.clone();
            Box::pin(async move {
                p.lock().unwrap().push(request.id);
                if request.id == 1 {
                    g.notified().await;
                }
            })
        },
        move |request: &Req| s.lock().unwrap().push(request.id),
        |_key: &str| {},
    );

    queue.highlight(req(1, "code"));
    tokio::task::yield_now().await;
    queue.highlight(req(2, "code"));
    queue.highlight(req(3, "code"));
    queue.highlight(req(4, "code"));

    assert_eq!(queue.pending(), 1);
    assert_eq!(*superseded.lock().unwrap(), vec![2, 3]);
    gate.notify_one();
    queue.idle().await;
    assert_eq!(*processed.lock().unwrap(), vec![1, 4]);
}

#[tokio::test]
async fn serializes_disposal_before_a_later_request_for_the_same_key() {
    let events = Arc::new(Mutex::new(Vec::<String>::new()));
    let (e1, e2, e3) = (events.clone(), events.clone(), events.clone());

    let queue = create_latest_worker_queue(
        move |request: Req| -> BoxFuture {
            let e = e1.clone();
            Box::pin(async move {
                e.lock().unwrap().push(format!("highlight:{}", request.id));
            })
        },
        move |request: &Req| e2.lock().unwrap().push(format!("supersede:{}", request.id)),
        move |key: &str| e3.lock().unwrap().push(format!("dispose:{key}")),
    );

    queue.highlight(req(1, "code"));
    queue.dispose("code");
    queue.highlight(req(2, "code"));
    queue.idle().await;

    assert_eq!(
        *events.lock().unwrap(),
        vec!["supersede:1", "dispose:code", "highlight:2"]
    );
}
