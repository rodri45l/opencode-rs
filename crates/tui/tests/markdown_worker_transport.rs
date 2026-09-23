//! Port of packages/session-ui/src/components/markdown-worker-transport.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/markdown-worker-transport.ts:
//! one request is posted per key and only the latest queued snapshot is retained;
//! responses for a disposed request are ignored; disposing drops queued snapshots.
//! Red-first: the worker transport queue is not implemented.

#[allow(dead_code)]
mod worker_transport {
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: session-ui markdown worker transport not implemented";

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct WorkerRequest {
        pub id: i64,
        pub key: String,
    }

    pub struct WorkerTransport {
        post: Box<dyn FnMut(&WorkerRequest)>,
        supersede: Box<dyn FnMut(&WorkerRequest)>,
    }

    impl WorkerTransport {
        pub fn new(
            post: impl FnMut(&WorkerRequest) + 'static,
            supersede: impl FnMut(&WorkerRequest) + 'static,
        ) -> Self {
            Self {
                post: Box::new(post),
                supersede: Box::new(supersede),
            }
        }

        pub fn send(&mut self, _request: WorkerRequest) -> PortResult<()> {
            Err(NotImplemented(NOTE))
        }

        pub fn complete(&mut self, _key: &str, _id: i64) -> PortResult<()> {
            Err(NotImplemented(NOTE))
        }

        pub fn dispose(&mut self, _key: &str) -> PortResult<()> {
            Err(NotImplemented(NOTE))
        }

        pub fn queued(&self) -> PortResult<usize> {
            Err(NotImplemented(NOTE))
        }
    }
}

use std::cell::RefCell;
use std::rc::Rc;

use worker_transport::{WorkerRequest, WorkerTransport, NOTE};

fn request(id: i64, key: &str) -> WorkerRequest {
    WorkerRequest {
        id,
        key: key.into(),
    }
}

#[test]
#[ignore = "porting: session-ui markdown worker transport not implemented"]
fn posts_one_request_and_retains_only_the_latest_queued_snapshot_per_key() {
    let posted = Rc::new(RefCell::new(Vec::new()));
    let superseded = Rc::new(RefCell::new(Vec::new()));
    let mut transport = WorkerTransport::new(
        {
            let posted = Rc::clone(&posted);
            move |request: &WorkerRequest| posted.borrow_mut().push(request.id)
        },
        {
            let superseded = Rc::clone(&superseded);
            move |request: &WorkerRequest| superseded.borrow_mut().push(request.id)
        },
    );

    transport.send(request(1, "code")).expect(NOTE);
    transport.send(request(2, "code")).expect(NOTE);
    transport.send(request(3, "code")).expect(NOTE);

    assert_eq!(*posted.borrow(), vec![1]);
    assert_eq!(*superseded.borrow(), vec![2]);
    assert_eq!(transport.queued().expect(NOTE), 1);
    transport.complete("code", 1).expect(NOTE);
    assert_eq!(*posted.borrow(), vec![1, 3]);
    assert_eq!(transport.queued().expect(NOTE), 0);
}

#[test]
#[ignore = "porting: session-ui markdown worker transport not implemented"]
fn ignores_a_disposed_request_response_after_the_key_is_reused() {
    let posted = Rc::new(RefCell::new(Vec::new()));
    let mut transport = WorkerTransport::new(
        {
            let posted = Rc::clone(&posted);
            move |request: &WorkerRequest| posted.borrow_mut().push(request.id)
        },
        |_request: &WorkerRequest| {},
    );

    transport.send(request(1, "code")).expect(NOTE);
    transport.dispose("code").expect(NOTE);
    transport.send(request(2, "code")).expect(NOTE);
    transport.send(request(3, "code")).expect(NOTE);
    transport.complete("code", 1).expect(NOTE);

    assert_eq!(*posted.borrow(), vec![1, 2]);
    assert_eq!(transport.queued().expect(NOTE), 1);
    transport.complete("code", 2).expect(NOTE);
    assert_eq!(*posted.borrow(), vec![1, 2, 3]);
}

#[test]
#[ignore = "porting: session-ui markdown worker transport not implemented"]
fn drops_queued_snapshots_when_a_key_is_disposed() {
    let superseded = Rc::new(RefCell::new(Vec::new()));
    let mut transport = WorkerTransport::new(|_request: &WorkerRequest| {}, {
        let superseded = Rc::clone(&superseded);
        move |request: &WorkerRequest| superseded.borrow_mut().push(request.id)
    });

    transport.send(request(1, "code")).expect(NOTE);
    transport.send(request(2, "code")).expect(NOTE);
    transport.dispose("code").expect(NOTE);

    assert_eq!(*superseded.borrow(), vec![2]);
    assert_eq!(transport.queued().expect(NOTE), 0);
}
