//! Port of packages/enterprise/test/core/storage.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/enterprise/src/core/storage.ts. The reference
//! global async store is re-derived as an owned in-memory store; each test
//! seeds its own fixtures.

use opencode_integrations::{ListOptions, Storage};

fn p(segments: &[&str]) -> Vec<String> {
    segments
        .iter()
        .map(|segment| (*segment).to_string())
        .collect()
}

fn seeded() -> Storage {
    let mut storage = Storage::new();
    for name in ["user1", "user2", "user3", "user4", "user5"] {
        storage.write(
            &["test", "users", name],
            serde_json::json!({ "name": name }),
        );
    }
    storage
}

#[test]
fn lists_files_with_after_and_before_range() {
    let storage = seeded();
    let result = storage.list(&ListOptions {
        prefix: p(&["test", "users"]),
        after: Some("user2".to_string()),
        before: Some("user4".to_string()),
        limit: None,
    });
    assert_eq!(result, vec![p(&["test", "users", "user3"])]);
}

#[test]
fn lists_files_with_after_only() {
    let storage = seeded();
    let result = storage.list(&ListOptions {
        prefix: p(&["test", "users"]),
        after: Some("user3".to_string()),
        before: None,
        limit: None,
    });
    assert_eq!(
        result,
        vec![
            p(&["test", "users", "user4"]),
            p(&["test", "users", "user5"]),
        ]
    );
}

#[test]
fn lists_files_with_limit() {
    let storage = seeded();
    let result = storage.list(&ListOptions {
        prefix: p(&["test", "users"]),
        after: None,
        before: None,
        limit: Some(3),
    });
    assert_eq!(
        result,
        vec![
            p(&["test", "users", "user1"]),
            p(&["test", "users", "user2"]),
            p(&["test", "users", "user3"]),
        ]
    );
}

#[test]
fn lists_all_files_without_prefix() {
    let storage = seeded();
    assert!(!storage.list(&ListOptions::default()).is_empty());
}

#[test]
fn lists_all_files_with_prefix() {
    let storage = seeded();
    let result = storage.list(&ListOptions {
        prefix: p(&["test", "users"]),
        after: None,
        before: None,
        limit: None,
    });
    assert_eq!(
        result,
        vec![
            p(&["test", "users", "user1"]),
            p(&["test", "users", "user2"]),
            p(&["test", "users", "user3"]),
            p(&["test", "users", "user4"]),
            p(&["test", "users", "user5"]),
        ]
    );
}

#[test]
fn removes_seeded_files_and_leaves_the_prefix_empty() {
    let mut storage = seeded();
    let test_files = storage.list(&ListOptions {
        prefix: p(&["test"]),
        ..Default::default()
    });
    for file in test_files {
        let refs: Vec<&str> = file.iter().map(String::as_str).collect();
        storage.remove(&refs);
    }
    let remaining = storage.list(&ListOptions {
        prefix: p(&["test"]),
        ..Default::default()
    });
    assert!(remaining.is_empty());
}
