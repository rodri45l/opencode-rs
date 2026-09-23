//! Port of packages/core/test/file-mutation.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: guarded writes/creates/removes with stable results, BOM
//! preservation, appearing/disappearing targets, external targets, and missing
//! removals. Dropped (re-derived): the concurrency serialization cases, which
//! depend on Effect fibers and an instrumented filesystem layer.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use opencode_core::file_mutation::{FileMutation, FileMutationResult, LocationMutation};
use opencode_core::CoreError;

const NOTE: &str = "porting: file-mutation not implemented";

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-rs-file-mutation-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn result(operation: &str, target: &str, resource: &str, existed: bool) -> FileMutationResult {
    FileMutationResult {
        operation: operation.into(),
        target: opencode_core::path::AbsolutePath::new(target),
        resource: resource.into(),
        existed,
    }
}

#[test]
#[ignore = "porting: file-mutation not implemented"]
fn writes_an_existing_internal_file_and_returns_a_stable_result() {
    let directory = scratch();
    let target_path = directory.join("hello.txt");
    std::fs::write(&target_path, "before").unwrap();

    let mutation = LocationMutation;
    let files = FileMutation;
    let target = mutation.resolve("hello.txt").expect(NOTE);

    assert_eq!(
        files.write(&target, "after").expect(NOTE),
        result(
            "write",
            target.canonical.as_path().to_str().unwrap(),
            "hello.txt",
            true
        )
    );
    assert_eq!(std::fs::read_to_string(&target_path).unwrap(), "after");
}

#[test]
#[ignore = "porting: file-mutation not implemented"]
fn writes_a_prospective_internal_file_and_creates_parent_directories() {
    scratch();

    let mutation = LocationMutation;
    let files = FileMutation;
    let target = mutation.resolve("src/nested/hello.txt").expect(NOTE);
    let outcome = files.write(&target, "hello").expect(NOTE);

    assert_eq!(
        outcome,
        result(
            "write",
            target.canonical.as_path().to_str().unwrap(),
            "src/nested/hello.txt",
            false
        )
    );
    assert_eq!(
        std::fs::read_to_string(outcome.target.as_path()).unwrap(),
        "hello"
    );
}

#[test]
#[ignore = "porting: file-mutation not implemented"]
fn preserves_exactly_one_bom_for_text_writes_and_normalizes_created_text() {
    let directory = scratch();
    let preserved_path = directory.join("preserved.txt");
    std::fs::write(&preserved_path, "\u{feff}before").unwrap();

    let mutation = LocationMutation;
    let files = FileMutation;
    let preserved = mutation.resolve("preserved.txt").expect(NOTE);
    let created = mutation.resolve("created.txt").expect(NOTE);

    files
        .write_text_preserving_bom(&preserved, "\u{feff}after")
        .expect(NOTE);
    files
        .write_text_preserving_bom(&created, "\u{feff}\u{feff}\u{feff}created")
        .expect(NOTE);

    assert_eq!(
        std::fs::read_to_string(&preserved_path).unwrap(),
        "\u{feff}after"
    );
    assert_eq!(
        std::fs::read_to_string(created.canonical.as_path()).unwrap(),
        "\u{feff}created"
    );
}

#[test]
#[ignore = "porting: file-mutation not implemented"]
fn rejects_create_when_a_prospective_target_appears_after_resolution() {
    let directory = scratch();
    let target_path = directory.join("appeared.txt");

    let mutation = LocationMutation;
    let files = FileMutation;
    let target = mutation.resolve("appeared.txt").expect(NOTE);
    std::fs::write(&target_path, "winner").unwrap();

    let error = files.create(&target, "replacement").unwrap_err();
    assert!(matches!(error, CoreError::TargetExists(_)));
    assert_eq!(std::fs::read_to_string(&target_path).unwrap(), "winner");
}

#[test]
#[ignore = "porting: file-mutation not implemented"]
fn creates_when_an_existing_target_disappears_after_resolution() {
    let directory = scratch();
    let target_path = directory.join("removed.txt");
    std::fs::write(&target_path, "before").unwrap();

    let mutation = LocationMutation;
    let files = FileMutation;
    let target = mutation.resolve("removed.txt").expect(NOTE);
    std::fs::remove_file(&target_path).unwrap();

    assert_eq!(
        files.create(&target, "after").expect(NOTE),
        result(
            "write",
            target.canonical.as_path().to_str().unwrap(),
            "removed.txt",
            false
        )
    );
    assert_eq!(std::fs::read_to_string(&target_path).unwrap(), "after");
}

#[test]
#[ignore = "porting: file-mutation not implemented"]
fn removes_an_existing_internal_file() {
    let directory = scratch();
    let target_path = directory.join("remove.txt");
    std::fs::write(&target_path, "remove").unwrap();

    let mutation = LocationMutation;
    let files = FileMutation;
    let target = mutation.resolve("remove.txt").expect(NOTE);
    let outcome = files.remove(&target).expect(NOTE);

    assert_eq!(
        outcome,
        result(
            "remove",
            target.canonical.as_path().to_str().unwrap(),
            "remove.txt",
            true
        )
    );
    assert!(!target_path.exists());
}

#[test]
#[ignore = "porting: file-mutation not implemented"]
fn writes_an_explicitly_resolved_external_target() {
    scratch();
    let outside = scratch();
    let target_path = outside.join("external.txt");

    let mutation = LocationMutation;
    let files = FileMutation;
    let target = mutation.resolve(target_path.to_str().unwrap()).expect(NOTE);
    let outcome = files.write(&target, "external").expect(NOTE);

    assert_eq!(
        outcome,
        result(
            "write",
            target.canonical.as_path().to_str().unwrap(),
            &target.resource,
            false
        )
    );
    assert_eq!(std::fs::read_to_string(&target_path).unwrap(), "external");
}

#[test]
#[ignore = "porting: file-mutation not implemented"]
fn removes_an_explicitly_resolved_external_target() {
    scratch();
    let outside = scratch();
    let target_path = outside.join("external.txt");
    std::fs::write(&target_path, "external").unwrap();

    let mutation = LocationMutation;
    let files = FileMutation;
    let target = mutation.resolve(target_path.to_str().unwrap()).expect(NOTE);
    let outcome = files.remove(&target).expect(NOTE);

    assert_eq!(
        outcome,
        result(
            "remove",
            target.canonical.as_path().to_str().unwrap(),
            &target.resource,
            true
        )
    );
    assert!(!target_path.exists());
}

#[test]
#[ignore = "porting: file-mutation not implemented"]
fn reports_a_missing_target_as_not_removed() {
    scratch();

    let mutation = LocationMutation;
    let files = FileMutation;
    let target = mutation.resolve("missing.txt").expect(NOTE);

    assert_eq!(
        files.remove(&target).expect(NOTE),
        result(
            "remove",
            target.canonical.as_path().to_str().unwrap(),
            "missing.txt",
            false
        )
    );
}

#[test]
#[ignore = "porting: file-mutation not implemented"]
fn rejects_a_conditional_write_when_target_content_is_already_stale() {
    let directory = scratch();
    let target_path = directory.join("stale.txt");
    std::fs::write(&target_path, "current").unwrap();

    let mutation = LocationMutation;
    let files = FileMutation;
    let target = mutation.resolve("stale.txt").expect(NOTE);

    let error = files
        .write_if_unchanged(&target, b"older", "replacement")
        .unwrap_err();
    assert!(matches!(error, CoreError::StaleContent(_)));
    assert_eq!(std::fs::read_to_string(&target_path).unwrap(), "current");
}
