//! Port of packages/core/test/tool-read-filesystem.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: reads fail with a typed filesystem error when a resolved
//! file disappears or a directory listing fails, with a path-kind error when the
//! target it is the wrong kind, with binary/malformed-UTF8 errors for
//! undecodable content, with an offset error for out-of-range pagination, and
//! with a media-ingest-limit error for oversized media; reading stops after the
//! requested page and reports the next offset. Re-derived: the Effect
//! `FSUtil`/`FileSystem` service wiring is replaced by `std::fs`; fixtures are
//! created in a scratch directory.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use opencode_core::tool_read_filesystem::{
    ReadToolError, ReadToolFileSystem, MAX_MEDIA_INGEST_BYTES,
};

const NOTE: &str = "porting: read-tool filesystem not implemented";

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-rs-readfs-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn fails_with_a_typed_filesystem_error_when_a_resolved_file_disappears() {
    let directory = scratch();
    let path = directory.join("missing.txt");
    let error = ReadToolFileSystem::read(path.to_str().unwrap(), "missing.txt", None, None)
        .expect_err(NOTE);
    assert!(matches!(error, ReadToolError::FileSystem { .. }));
    if let ReadToolError::FileSystem { method, .. } = error {
        assert_eq!(method, "readFile");
    }
}

#[test]
fn fails_when_a_file_becomes_the_wrong_path_kind() {
    let directory = scratch();
    let error = ReadToolFileSystem::read(directory.to_str().unwrap(), "folder", None, None)
        .expect_err(NOTE);
    assert!(matches!(error, ReadToolError::PathKind(_)));
}

#[test]
fn fails_with_a_typed_filesystem_error_when_directory_listing_fails() {
    let directory = scratch();
    let file = directory.join("some-file.txt");
    std::fs::write(&file, "hello").unwrap();
    let error = ReadToolFileSystem::list(file.to_str().unwrap()).expect_err(NOTE);
    assert!(matches!(error, ReadToolError::FileSystem { .. }));
    if let ReadToolError::FileSystem { method, .. } = error {
        assert_eq!(method, "readDirectoryEntries");
    }
}

#[test]
fn reports_binary_and_malformed_utf8_content_as_typed_errors() {
    let directory = scratch();
    let archive = directory.join("archive.dat");
    std::fs::write(&archive, [0x00, 0x01, 0x02, 0x03]).unwrap();
    let malformed_path = directory.join("malformed.txt");
    std::fs::write(&malformed_path, [0xff, 0xfe, 0xfd]).unwrap();

    let binary = ReadToolFileSystem::read(archive.to_str().unwrap(), "archive.dat", None, None)
        .expect_err(NOTE);
    assert!(matches!(binary, ReadToolError::Binary(_)));
    assert_eq!(binary.to_string(), "Cannot read binary file: archive.dat");

    let malformed = ReadToolFileSystem::read(
        malformed_path.to_str().unwrap(),
        "malformed.txt",
        None,
        None,
    )
    .expect_err(NOTE);
    assert!(matches!(malformed, ReadToolError::MalformedUtf8(_)));
}

#[test]
fn reports_out_of_range_pagination_as_a_typed_error() {
    let directory = scratch();
    let short = directory.join("short.txt");
    std::fs::write(&short, "only").unwrap();
    let error = ReadToolFileSystem::read(short.to_str().unwrap(), "short.txt", Some(2), None)
        .expect_err(NOTE);
    assert!(matches!(error, ReadToolError::OffsetOutOfRange(_)));
    assert_eq!(error.to_string(), "Offset 2 is out of range");
}

#[test]
fn stops_reading_after_the_requested_page() {
    let directory = scratch();
    let lines = directory.join("lines.txt");
    std::fs::write(&lines, "one\ntwo\nthree").unwrap();
    let page =
        ReadToolFileSystem::read(lines.to_str().unwrap(), "lines.txt", None, Some(1)).expect(NOTE);
    assert_eq!(page.content, "one");
    assert!(page.truncated);
    assert_eq!(page.next, Some(2));
}

#[test]
fn preserves_the_media_ingestion_limit_message() {
    let directory = scratch();
    let oversized = directory.join("oversized.png");
    let file = std::fs::File::create(&oversized).unwrap();
    file.set_len(MAX_MEDIA_INGEST_BYTES as u64 + 1).unwrap();
    drop(file);

    let error = ReadToolFileSystem::read(oversized.to_str().unwrap(), "oversized.png", None, None)
        .expect_err(NOTE);
    assert!(matches!(error, ReadToolError::MediaIngestLimit(_)));
    assert_eq!(
        error.to_string(),
        format!("Media exceeds {MAX_MEDIA_INGEST_BYTES} byte ingestion limit: oversized.png")
    );
}
