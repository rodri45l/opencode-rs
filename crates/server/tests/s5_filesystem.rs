//! Port of packages/opencode/test/filesystem/filesystem.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `FSUtil` directory/file predicates, JSON round-trip,
//! `ensureDir`, `findUp`, built-in `exists`/`remove` passthrough, and the pure
//! `mimeType`/`contains`/`overlaps` helpers.
//!
//! The reference file's `glob`/`globUp`/`up`/binary `writeWithDirs` suites are
//! tracked as skipped (see PORT-STATUS.s5.json).
#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use opencode_server::fs_util;

fn tmpdir() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("oc_fs_{}_{}_{}", std::process::id(), nanos, n));
    fs::create_dir_all(&dir).expect("tmpdir");
    dir
}

#[test]
fn is_dir_returns_true_for_directories() {
    let tmp = tmpdir();
    assert!(fs_util::is_dir(&tmp));
}

#[test]
fn is_dir_returns_false_for_files() {
    let tmp = tmpdir();
    let file = tmp.join("test.txt");
    fs::write(&file, "hello").unwrap();
    assert!(!fs_util::is_dir(&file));
}

#[test]
fn is_dir_returns_false_for_non_existent_paths() {
    let tmp = tmpdir();
    let missing = tmp.join("does-not-exist");
    assert!(!fs_util::is_dir(&missing));
}

#[test]
fn is_file_returns_true_for_files() {
    let tmp = tmpdir();
    let file = tmp.join("test.txt");
    fs::write(&file, "hello").unwrap();
    assert!(fs_util::is_file(&file));
}

#[test]
fn is_file_returns_false_for_directories() {
    let tmp = tmpdir();
    assert!(!fs_util::is_file(&tmp));
}

#[test]
fn read_json_and_write_json_round_trip() {
    let tmp = tmpdir();
    let file = tmp.join("data.json");
    let data = serde_json::json!({ "name": "test", "count": 42, "nested": { "ok": true } });

    fs_util::write_json(&file, &data).unwrap();
    let result = fs_util::read_json(&file).unwrap();

    assert_eq!(result, data);
}

#[test]
fn ensure_dir_creates_nested_directories() {
    let tmp = tmpdir();
    let nested = tmp.join("a").join("b").join("c");

    fs_util::ensure_dir(&nested).unwrap();

    assert!(nested.is_dir());
}

#[test]
fn ensure_dir_is_idempotent() {
    let tmp = tmpdir();
    let dir = tmp.join("existing");
    fs::create_dir(&dir).unwrap();

    fs_util::ensure_dir(&dir).unwrap();

    assert!(dir.is_dir());
}

#[test]
fn find_up_finds_target_in_start_directory() {
    let tmp = tmpdir();
    fs::write(tmp.join("target.txt"), "found").unwrap();

    let result = fs_util::find_up(&["target.txt"], &tmp, &tmp);
    assert_eq!(result, vec![tmp.join("target.txt")]);
}

#[test]
fn find_up_finds_target_in_parent_directories() {
    let tmp = tmpdir();
    fs::write(tmp.join("marker"), "root").unwrap();
    let child = tmp.join("a").join("b");
    fs::create_dir_all(&child).unwrap();

    let result = fs_util::find_up(&["marker"], &child, &tmp);
    assert_eq!(result, vec![tmp.join("marker")]);
}

#[test]
fn find_up_returns_empty_array_when_not_found() {
    let tmp = tmpdir();
    let result = fs_util::find_up(&["nonexistent"], &tmp, &tmp);
    assert!(result.is_empty());
}

#[test]
fn built_in_exists_works() {
    let tmp = tmpdir();
    let file = tmp.join("exists.txt");
    fs::write(&file, "yes").unwrap();

    assert!(fs_util::exists(&file));
    let missing = PathBuf::from(format!("{}.nope", file.to_string_lossy()));
    assert!(!fs_util::exists(&missing));
}

#[test]
fn built_in_remove_works() {
    let tmp = tmpdir();
    let file = tmp.join("delete-me.txt");
    fs::write(&file, "bye").unwrap();

    fs_util::remove(&file).unwrap();

    assert!(!fs_util::exists(&file));
}

#[test]
fn pure_helpers_mime_type_returns_correct_types() {
    assert_eq!(fs_util::mime_type("file.json"), "application/json");
    assert_eq!(fs_util::mime_type("image.png"), "image/png");
    assert_eq!(
        fs_util::mime_type("unknown.qzx"),
        "application/octet-stream"
    );
}

#[test]
fn pure_helpers_contains_checks_path_containment() {
    assert!(fs_util::contains("/a/b", "/a/b/c"));
    assert!(!fs_util::contains("/a/b", "/a/c"));
}

#[test]
fn pure_helpers_overlaps_detects_overlapping_paths() {
    assert!(fs_util::overlaps("/a/b", "/a/b/c"));
    assert!(fs_util::overlaps("/a/b/c", "/a/b"));
    assert!(!fs_util::overlaps("/a", "/b"));
}
