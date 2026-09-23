//! Port of packages/core/test/location-filesystem.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `FileSystem` reads text and binary files with a MIME type,
//! lists direct children, and rejects lexical escapes. Re-derived against a
//! plain directory root; the `LayerNode`/`Location` wiring is dropped.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use opencode_core::location_filesystem::FileSystem;
use opencode_core::path::AbsolutePath;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-rs-locfs-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn reads_text_and_binary_files() {
    let dir = scratch();
    std::fs::write(dir.join("text.txt"), "hello").unwrap();
    std::fs::write(dir.join("data.bin"), [0u8, 1, 2]).unwrap();
    let root = AbsolutePath::new(dir.clone());
    let fs = FileSystem;

    let text = fs.read(&root, "text.txt").unwrap();
    assert_eq!(text.content, b"hello");
    assert_eq!(text.mime, "text/plain");

    let binary = fs.read(&root, "data.bin").unwrap();
    assert_eq!(binary.content, vec![0u8, 1, 2]);
}

#[test]
fn lists_direct_children() {
    let dir = scratch();
    std::fs::create_dir(dir.join("src")).unwrap();
    std::fs::write(dir.join("README.md"), "# Test").unwrap();
    let entries = FileSystem.list(&AbsolutePath::new(dir.clone())).unwrap();

    let mapped: Vec<(String, String)> = entries
        .into_iter()
        .map(|entry| (entry.path, entry.entry_type))
        .collect();
    assert_eq!(
        mapped,
        vec![
            (
                format!("src{}", std::path::MAIN_SEPARATOR),
                "directory".to_string()
            ),
            ("README.md".to_string(), "file".to_string()),
        ]
    );
}

#[test]
fn rejects_lexical_escapes() {
    let dir = scratch();
    assert!(FileSystem
        .read(&AbsolutePath::new(dir.clone()), "../outside.txt")
        .is_err());
}
