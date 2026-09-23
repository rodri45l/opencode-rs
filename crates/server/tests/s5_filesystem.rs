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

// Fast-wave local stubs: `filesystem::fs_util` is not implemented yet.
mod fs_util {
    use std::path::{Path, PathBuf};

    pub fn is_dir(_path: &Path) -> Result<bool, &'static str> {
        Err("porting: FSUtil::isDir not implemented")
    }

    pub fn is_file(_path: &Path) -> Result<bool, &'static str> {
        Err("porting: FSUtil::isFile not implemented")
    }

    pub fn read_json(_path: &Path) -> Result<serde_json::Value, &'static str> {
        Err("porting: FSUtil::readJson not implemented")
    }

    pub fn write_json(_path: &Path, _value: &serde_json::Value) -> Result<(), &'static str> {
        Err("porting: FSUtil::writeJson not implemented")
    }

    pub fn ensure_dir(_path: &Path) -> Result<(), &'static str> {
        Err("porting: FSUtil::ensureDir not implemented")
    }

    pub fn find_up(
        _targets: &[&str],
        _start: &Path,
        _stop: &Path,
    ) -> Result<Vec<PathBuf>, &'static str> {
        Err("porting: FSUtil::findUp not implemented")
    }

    pub fn exists(_path: &Path) -> Result<bool, &'static str> {
        Err("porting: FSUtil::exists not implemented")
    }

    pub fn remove(_path: &Path) -> Result<(), &'static str> {
        Err("porting: FSUtil::remove not implemented")
    }

    pub fn mime_type(_path: &str) -> Result<String, &'static str> {
        Err("porting: FSUtil::mimeType not implemented")
    }

    pub fn contains(_parent: &str, _child: &str) -> Result<bool, &'static str> {
        Err("porting: FSUtil::contains not implemented")
    }

    pub fn overlaps(_left: &str, _right: &str) -> Result<bool, &'static str> {
        Err("porting: FSUtil::overlaps not implemented")
    }
}

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
#[ignore = "porting: filesystem fs-util not implemented"]
fn is_dir_returns_true_for_directories() {
    let tmp = tmpdir();
    assert!(fs_util::is_dir(&tmp).unwrap());
}

#[test]
#[ignore = "porting: filesystem fs-util not implemented"]
fn is_dir_returns_false_for_files() {
    let tmp = tmpdir();
    let file = tmp.join("test.txt");
    fs::write(&file, "hello").unwrap();
    assert!(!fs_util::is_dir(&file).unwrap());
}

#[test]
#[ignore = "porting: filesystem fs-util not implemented"]
fn is_dir_returns_false_for_non_existent_paths() {
    let tmp = tmpdir();
    let missing = tmp.join("does-not-exist");
    assert!(!fs_util::is_dir(&missing).unwrap());
}

#[test]
#[ignore = "porting: filesystem fs-util not implemented"]
fn is_file_returns_true_for_files() {
    let tmp = tmpdir();
    let file = tmp.join("test.txt");
    fs::write(&file, "hello").unwrap();
    assert!(fs_util::is_file(&file).unwrap());
}

#[test]
#[ignore = "porting: filesystem fs-util not implemented"]
fn is_file_returns_false_for_directories() {
    let tmp = tmpdir();
    assert!(!fs_util::is_file(&tmp).unwrap());
}

#[test]
#[ignore = "porting: filesystem fs-util not implemented"]
fn read_json_and_write_json_round_trip() {
    let tmp = tmpdir();
    let file = tmp.join("data.json");
    let data = serde_json::json!({ "name": "test", "count": 42, "nested": { "ok": true } });

    fs_util::write_json(&file, &data).unwrap();
    let result = fs_util::read_json(&file).unwrap();

    assert_eq!(result, data);
}

#[test]
#[ignore = "porting: filesystem fs-util not implemented"]
fn ensure_dir_creates_nested_directories() {
    let tmp = tmpdir();
    let nested = tmp.join("a").join("b").join("c");

    fs_util::ensure_dir(&nested).unwrap();

    assert!(nested.is_dir());
}

#[test]
#[ignore = "porting: filesystem fs-util not implemented"]
fn ensure_dir_is_idempotent() {
    let tmp = tmpdir();
    let dir = tmp.join("existing");
    fs::create_dir(&dir).unwrap();

    fs_util::ensure_dir(&dir).unwrap();

    assert!(dir.is_dir());
}

#[test]
#[ignore = "porting: filesystem fs-util not implemented"]
fn find_up_finds_target_in_start_directory() {
    let tmp = tmpdir();
    fs::write(tmp.join("target.txt"), "found").unwrap();

    let result = fs_util::find_up(&["target.txt"], &tmp, &tmp).unwrap();
    assert_eq!(result, vec![tmp.join("target.txt")]);
}

#[test]
#[ignore = "porting: filesystem fs-util not implemented"]
fn find_up_finds_target_in_parent_directories() {
    let tmp = tmpdir();
    fs::write(tmp.join("marker"), "root").unwrap();
    let child = tmp.join("a").join("b");
    fs::create_dir_all(&child).unwrap();

    let result = fs_util::find_up(&["marker"], &child, &tmp).unwrap();
    assert_eq!(result, vec![tmp.join("marker")]);
}

#[test]
#[ignore = "porting: filesystem fs-util not implemented"]
fn find_up_returns_empty_array_when_not_found() {
    let tmp = tmpdir();
    let result = fs_util::find_up(&["nonexistent"], &tmp, &tmp).unwrap();
    assert!(result.is_empty());
}

#[test]
#[ignore = "porting: filesystem fs-util not implemented"]
fn built_in_exists_works() {
    let tmp = tmpdir();
    let file = tmp.join("exists.txt");
    fs::write(&file, "yes").unwrap();

    assert!(fs_util::exists(&file).unwrap());
    let missing = PathBuf::from(format!("{}.nope", file.to_string_lossy()));
    assert!(!fs_util::exists(&missing).unwrap());
}

#[test]
#[ignore = "porting: filesystem fs-util not implemented"]
fn built_in_remove_works() {
    let tmp = tmpdir();
    let file = tmp.join("delete-me.txt");
    fs::write(&file, "bye").unwrap();

    fs_util::remove(&file).unwrap();

    assert!(!fs_util::exists(&file).unwrap());
}

#[test]
#[ignore = "porting: filesystem fs-util not implemented"]
fn pure_helpers_mime_type_returns_correct_types() {
    assert_eq!(fs_util::mime_type("file.json").unwrap(), "application/json");
    assert_eq!(fs_util::mime_type("image.png").unwrap(), "image/png");
    assert_eq!(
        fs_util::mime_type("unknown.qzx").unwrap(),
        "application/octet-stream"
    );
}

#[test]
#[ignore = "porting: filesystem fs-util not implemented"]
fn pure_helpers_contains_checks_path_containment() {
    assert!(fs_util::contains("/a/b", "/a/b/c").unwrap());
    assert!(!fs_util::contains("/a/b", "/a/c").unwrap());
}

#[test]
#[ignore = "porting: filesystem fs-util not implemented"]
fn pure_helpers_overlaps_detects_overlapping_paths() {
    assert!(fs_util::overlaps("/a/b", "/a/b/c").unwrap());
    assert!(fs_util::overlaps("/a/b/c", "/a/b").unwrap());
    assert!(!fs_util::overlaps("/a", "/b").unwrap());
}
