//! Port of packages/core/test/filesystem/filesystem.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `isDir`/`isFile` predicates, safe reads, JSON round-trips,
//! directory creation, `writeWithDirs`, upward search, globbing, and the pure
//! `mimeType`/`contains`/`overlaps` helpers. Dropped (re-derived): the
//! `FileSystem` passthrough `exists`/`remove` cases, which test the Effect
//! platform service rather than `FSUtil`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use opencode_core::fs_util::{FSUtil, UpOptions};
use opencode_core::path::AbsolutePath;
use opencode_core::CoreError;
use serde_json::json;

const NOTE: &str = "porting: fs-util not implemented";

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-rs-fsutil-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn is_dir_returns_true_for_directories() {
    let dir = scratch();
    assert!(FSUtil::is_dir(&dir).expect(NOTE));
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn is_dir_returns_false_for_files() {
    let dir = scratch();
    let file = dir.join("test.txt");
    std::fs::write(&file, "hello").unwrap();
    assert!(!FSUtil::is_dir(&file).expect(NOTE));
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn is_dir_returns_false_for_non_existent_paths() {
    let dir = scratch();
    assert!(!FSUtil::is_dir(&dir.join("nonexistent")).expect(NOTE));
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn is_file_returns_true_for_files() {
    let dir = scratch();
    let file = dir.join("test.txt");
    std::fs::write(&file, "hello").unwrap();
    assert!(FSUtil::is_file(&file).expect(NOTE));
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn is_file_returns_false_for_directories() {
    let dir = scratch();
    assert!(!FSUtil::is_file(&dir).expect(NOTE));
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn read_file_string_safe_returns_contents_when_file_exists() {
    let dir = scratch();
    let file = dir.join("exists.txt");
    std::fs::write(&file, "hello").unwrap();
    assert_eq!(
        FSUtil::read_file_string_safe(&file).expect(NOTE),
        Some("hello".into())
    );
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn read_file_string_safe_returns_none_for_missing_files() {
    let dir = scratch();
    assert_eq!(
        FSUtil::read_file_string_safe(&dir.join("does-not-exist.txt")).expect(NOTE),
        None
    );
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn read_json_and_write_json_round_trip() {
    let dir = scratch();
    let file = dir.join("data.json");
    let data = json!({ "name": "test", "count": 42, "nested": { "ok": true } });

    FSUtil::write_json(&file, &data).expect(NOTE);
    assert_eq!(FSUtil::read_json(&file).expect(NOTE), data);
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn read_json_fails_invalid_json_through_the_error_channel() {
    let dir = scratch();
    let file = dir.join("broken.json");
    std::fs::write(&file, "{").unwrap();

    let error = FSUtil::read_json(&file).unwrap_err();
    assert!(matches!(error, CoreError::FileSystem(_)));
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn ensure_dir_creates_nested_directories() {
    let dir = scratch();
    let nested = dir.join("a").join("b").join("c");
    FSUtil::ensure_dir(&nested).expect(NOTE);
    assert!(nested.is_dir());
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn ensure_dir_is_idempotent() {
    let dir = scratch();
    let existing = dir.join("existing");
    std::fs::create_dir(&existing).unwrap();
    FSUtil::ensure_dir(&existing).expect(NOTE);
    assert!(existing.is_dir());
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn write_with_dirs_creates_parent_directories_if_missing() {
    let dir = scratch();
    let file = dir.join("deep").join("nested").join("file.txt");
    FSUtil::write_with_dirs(&file, b"hello").expect(NOTE);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "hello");
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn write_with_dirs_writes_directly_when_parent_exists() {
    let dir = scratch();
    let file = dir.join("direct.txt");
    FSUtil::write_with_dirs(&file, b"world").expect(NOTE);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "world");
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn write_with_dirs_writes_byte_content() {
    let dir = scratch();
    let file = dir.join("binary.bin");
    FSUtil::write_with_dirs(&file, &[0x00, 0x01, 0x02, 0x03]).expect(NOTE);
    assert_eq!(std::fs::read(&file).unwrap(), vec![0x00, 0x01, 0x02, 0x03]);
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn find_up_finds_the_target_in_the_start_directory() {
    let dir = scratch();
    std::fs::write(dir.join("target.txt"), "found").unwrap();
    assert_eq!(
        FSUtil::find_up("target.txt", &dir, None).expect(NOTE),
        vec![AbsolutePath::new(dir.join("target.txt"))]
    );
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn find_up_finds_the_target_in_parent_directories() {
    let dir = scratch();
    std::fs::write(dir.join("marker"), "root").unwrap();
    let child = dir.join("a").join("b");
    std::fs::create_dir_all(&child).unwrap();
    assert_eq!(
        FSUtil::find_up("marker", &child, Some(&dir)).expect(NOTE),
        vec![AbsolutePath::new(dir.join("marker"))]
    );
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn find_up_returns_empty_when_not_found() {
    let dir = scratch();
    assert_eq!(
        FSUtil::find_up("nonexistent", &dir, Some(&dir)).expect(NOTE),
        Vec::new()
    );
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn up_finds_multiple_targets_walking_up() {
    let dir = scratch();
    std::fs::write(dir.join("a.txt"), "a").unwrap();
    std::fs::write(dir.join("b.txt"), "b").unwrap();
    let child = dir.join("sub");
    std::fs::create_dir(&child).unwrap();
    std::fs::write(child.join("a.txt"), "a-child").unwrap();

    let found = FSUtil::up(UpOptions {
        targets: vec!["a.txt".into(), "b.txt".into()],
        start: child.clone(),
        stop: Some(dir.clone()),
    })
    .expect(NOTE);

    assert!(found.contains(&AbsolutePath::new(child.join("a.txt"))));
    assert!(found.contains(&AbsolutePath::new(dir.join("a.txt"))));
    assert!(found.contains(&AbsolutePath::new(dir.join("b.txt"))));
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn glob_finds_files_matching_a_pattern() {
    let dir = scratch();
    std::fs::write(dir.join("a.ts"), "a").unwrap();
    std::fs::write(dir.join("b.ts"), "b").unwrap();
    std::fs::write(dir.join("c.json"), "c").unwrap();

    let mut found = FSUtil::glob("*.ts", &dir, false).expect(NOTE);
    found.sort();
    assert_eq!(found, vec!["a.ts".to_string(), "b.ts".to_string()]);
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn glob_supports_absolute_paths() {
    let dir = scratch();
    std::fs::write(dir.join("file.txt"), "hello").unwrap();
    assert_eq!(
        FSUtil::glob("*.txt", &dir, true).expect(NOTE),
        vec![dir.join("file.txt").to_string_lossy().to_string()]
    );
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn glob_match_matches_patterns() {
    assert!(FSUtil::glob_match("*.ts", "foo.ts").expect(NOTE));
    assert!(!FSUtil::glob_match("*.ts", "foo.json").expect(NOTE));
    assert!(FSUtil::glob_match("src/**", "src/a/b.ts").expect(NOTE));
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn glob_up_finds_files_walking_up_directories() {
    let dir = scratch();
    std::fs::write(dir.join("root.md"), "root").unwrap();
    let child = dir.join("a").join("b");
    std::fs::create_dir_all(&child).unwrap();
    std::fs::write(child.join("leaf.md"), "leaf").unwrap();

    let found = FSUtil::glob_up("*.md", &child, &dir).expect(NOTE);
    assert!(found.contains(&AbsolutePath::new(child.join("leaf.md"))));
    assert!(found.contains(&AbsolutePath::new(dir.join("root.md"))));
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn mime_type_returns_correct_types() {
    assert_eq!(
        FSUtil::mime_type("file.json").expect(NOTE),
        "application/json"
    );
    assert_eq!(FSUtil::mime_type("image.png").expect(NOTE), "image/png");
    assert_eq!(
        FSUtil::mime_type("unknown.qzx").expect(NOTE),
        "application/octet-stream"
    );
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn contains_checks_path_containment() {
    assert!(FSUtil::contains("/a/b", "/a/b/c").expect(NOTE));
    assert!(FSUtil::contains("/a/b", "/a/b").expect(NOTE));
    assert!(!FSUtil::contains("/a/b", "/a/c").expect(NOTE));
    assert!(!FSUtil::contains("/a/b", "/a/bad").expect(NOTE));
}

#[test]
#[ignore = "porting: fs-util not implemented"]
fn overlaps_detects_overlapping_paths() {
    assert!(FSUtil::overlaps("/a/b", "/a/b/c").expect(NOTE));
    assert!(FSUtil::overlaps("/a/b/c", "/a/b").expect(NOTE));
    assert!(!FSUtil::overlaps("/a", "/b").expect(NOTE));
    assert!(!FSUtil::overlaps("/a/b", "/a/bad").expect(NOTE));
}
