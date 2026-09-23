//! Port of packages/core/test/location-mutation.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: relative existing and prospective targets resolve to
//! canonical paths and resources, relative lexical escapes are rejected, explicit
//! absolute in-location targets need no external approval, explicit external
//! absolute targets require external-directory authorization, and unknown input
//! fields are ignored. Dropped: the symlink cases, which need platform-specific
//! link setup.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use opencode_core::location_mutation::{LocationMutation, ResolveInput};
use opencode_core::path::AbsolutePath;
use serde_json::json;

const NOTE: &str = "porting: location mutation not implemented";

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-rs-locmut-{tag}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::canonicalize(&dir).unwrap()
}

fn input(path: &str) -> ResolveInput {
    ResolveInput {
        path: path.into(),
        kind: None,
    }
}

#[test]
#[ignore = "porting: location mutation not implemented"]
fn resolves_an_active_relative_existing_file_target() {
    let directory = scratch("relative-existing");
    let target_path = directory.join("hello.txt");
    std::fs::write(&target_path, "hello").unwrap();

    let target =
        LocationMutation::resolve(&AbsolutePath::new(directory.clone()), &input("hello.txt"))
            .expect(NOTE);

    assert_eq!(target.canonical, AbsolutePath::new(target_path.clone()));
    assert_eq!(target.resource, "hello.txt");
    assert_eq!(target.external_directory, None);
}

#[test]
#[ignore = "porting: location mutation not implemented"]
fn resolves_an_active_relative_prospective_file_target() {
    let directory = scratch("relative-prospective");
    std::fs::create_dir(directory.join("src")).unwrap();

    let target =
        LocationMutation::resolve(&AbsolutePath::new(directory.clone()), &input("src/new.txt"))
            .expect(NOTE);

    assert_eq!(
        target.canonical,
        AbsolutePath::new(directory.join("src").join("new.txt"))
    );
    assert_eq!(target.resource, "src/new.txt");
}

#[test]
#[ignore = "porting: location mutation not implemented"]
fn rejects_a_relative_lexical_escape() {
    let directory = scratch("relative-escape");
    assert!(LocationMutation::resolve(
        &AbsolutePath::new(directory.clone()),
        &input("../outside.txt")
    )
    .is_err());
}

#[test]
#[ignore = "porting: location mutation not implemented"]
fn accepts_an_explicit_absolute_in_location_target() {
    let directory = scratch("absolute-inside");
    let target = LocationMutation::resolve(
        &AbsolutePath::new(directory.clone()),
        &input(&directory.join("new.txt").to_string_lossy()),
    )
    .expect(NOTE);

    assert_eq!(
        target.canonical,
        AbsolutePath::new(directory.join("new.txt"))
    );
    assert_eq!(target.resource, "new.txt");
    assert_eq!(target.external_directory, None);
}

#[test]
#[ignore = "porting: location mutation not implemented"]
fn requires_external_directory_authorization_for_an_explicit_external_absolute_target() {
    let directory = scratch("external-inside");
    let outside = scratch("external-outside");

    let target = LocationMutation::resolve(
        &AbsolutePath::new(directory.clone()),
        &input(&outside.join("new.txt").to_string_lossy()),
    )
    .expect(NOTE);

    let root = outside.to_string_lossy().replace('\\', "/");
    assert_eq!(target.canonical, AbsolutePath::new(outside.join("new.txt")));
    assert_eq!(target.resource, format!("{root}/new.txt"));
    let external = target.external_directory.expect(NOTE);
    assert_eq!(external.directory, AbsolutePath::new(outside.clone()));
    assert_eq!(external.resource, format!("{root}/*"));
}

#[test]
#[ignore = "porting: location mutation not implemented"]
fn ignores_unknown_mutation_input_fields() {
    let decoded =
        ResolveInput::decode(&json!({ "path": "README.md", "reference": "docs" })).expect(NOTE);
    assert_eq!(decoded.path, "README.md");
    assert_eq!(decoded.kind, None);
}
