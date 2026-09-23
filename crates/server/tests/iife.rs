//! Port of packages/opencode/test/util/iife.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `iife` runs its thunk immediately and returns the result,
//! including async thunks and thunks that return no value.

#![allow(dead_code)]

#[derive(Debug, PartialEq, Eq)]
enum PortError {
    NotImplemented(&'static str),
}

type PortResult<T> = Result<T, PortError>;

fn iife<T>(_f: impl FnOnce() -> T) -> PortResult<T> {
    Err(PortError::NotImplemented("iife"))
}

#[test]
#[ignore = "porting: iife not implemented"]
fn should_execute_function_immediately_and_return_result() {
    let mut called = false;
    let result = iife(|| {
        called = true;
        42
    })
    .unwrap();

    assert!(called);
    assert_eq!(result, 42);
}

#[tokio::test]
#[ignore = "porting: iife not implemented"]
async fn should_work_with_async_functions() {
    let called = std::cell::Cell::new(false);
    let result = iife(|| async {
        called.set(true);
        "async result"
    })
    .unwrap()
    .await;

    assert!(called.get());
    assert_eq!(result, "async result");
}

#[test]
#[ignore = "porting: iife not implemented"]
fn should_handle_functions_with_no_return_value() {
    let mut called = false;
    iife(|| {
        called = true;
    })
    .unwrap();

    assert!(called);
}
