//! Port of packages/opencode/test/util/lazy.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `lazy` wraps a producer so it is invoked at most once and
//! every reader observes the same value.
//!
//! Dropped: the reference's `null`/`undefined` return-type probes are folded
//! into the typed string/number/boolean cases; Rust has no `undefined` value.

#![allow(dead_code)]

#[derive(Debug, PartialEq, Eq)]
enum PortError {
    NotImplemented(&'static str),
}

type PortResult<T> = Result<T, PortError>;

fn lazy<T, F: Fn() -> T>(_f: F) -> impl Fn() -> PortResult<T> {
    move || Err(PortError::NotImplemented("lazy"))
}

#[test]
#[ignore = "porting: lazy not implemented"]
fn should_call_function_only_once() {
    let call_count = std::cell::Cell::new(0);
    let get_value = || {
        call_count.set(call_count.get() + 1);
        "expensive value"
    };

    let lazy_value = lazy(get_value);
    assert_eq!(call_count.get(), 0);

    let result1 = lazy_value().unwrap();
    assert_eq!(result1, "expensive value");
    assert_eq!(call_count.get(), 1);

    let result2 = lazy_value().unwrap();
    assert_eq!(result2, "expensive value");
    assert_eq!(call_count.get(), 1);
}

#[test]
#[ignore = "porting: lazy not implemented"]
fn should_preserve_the_same_reference() {
    let obj = "value";
    let lazy_obj = lazy(|| obj);

    let result1 = lazy_obj().unwrap();
    let result2 = lazy_obj().unwrap();

    assert!(std::ptr::eq(result1, result2));
    assert!(std::ptr::eq(result1, obj));
}

#[test]
#[ignore = "porting: lazy not implemented"]
fn should_work_with_different_return_types() {
    let lazy_string = lazy(|| "string");
    let lazy_number = lazy(|| 123);
    let lazy_boolean = lazy(|| true);

    assert_eq!(lazy_string().unwrap(), "string");
    assert_eq!(lazy_number().unwrap(), 123);
    assert!(lazy_boolean().unwrap());
}
