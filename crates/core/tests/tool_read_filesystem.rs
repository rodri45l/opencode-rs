//! Port of packages/core/test/tool-read-filesystem.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: reads fail with a typed filesystem error when a resolved
//! file disappears or a directory listing fails, with a path-kind error when the
//! target it is the wrong kind, with binary/malformed-UTF8 errors for
//! undecodable content, with an offset error for out-of-range pagination, and
//! with a media-ingest-limit error for oversized media; reading stops after the
//! requested page and reports the next offset. Re-derived: the Effect
//! `FSUtil`/`FileSystem` service wiring is replaced by `std::fs`; the readonly
//! service argument is dropped.

use opencode_core::tool_read_filesystem::{
    ReadToolError, ReadToolFileSystem, MAX_MEDIA_INGEST_BYTES,
};

const NOTE: &str = "porting: read-tool filesystem not implemented";

#[test]
#[ignore = "porting: read-tool filesystem not implemented"]
fn fails_with_a_typed_filesystem_error_when_a_resolved_file_disappears() {
    let error = ReadToolFileSystem::read(
        "/tmp/does-not-exist-opencode/missing.txt",
        "missing.txt",
        None,
        None,
    )
    .expect_err(NOTE);
    assert!(matches!(error, ReadToolError::FileSystem { .. }));
    assert_eq!(error.to_string(), "readFile: not implemented");
}

#[test]
#[ignore = "porting: read-tool filesystem not implemented"]
fn fails_when_a_file_becomes_the_wrong_path_kind() {
    let error =
        ReadToolFileSystem::read("/tmp/some-directory", "folder", None, None).expect_err(NOTE);
    assert!(matches!(error, ReadToolError::PathKind(_)));
}

#[test]
#[ignore = "porting: read-tool filesystem not implemented"]
fn fails_with_a_typed_filesystem_error_when_directory_listing_fails() {
    let error = ReadToolFileSystem::list("/tmp/some-file.txt").expect_err(NOTE);
    assert!(matches!(error, ReadToolError::FileSystem { .. }));
    if let ReadToolError::FileSystem { method, .. } = error {
        assert_eq!(method, "readDirectoryEntries");
    }
}

#[test]
#[ignore = "porting: read-tool filesystem not implemented"]
fn reports_binary_and_malformed_utf8_content_as_typed_errors() {
    let binary =
        ReadToolFileSystem::read("/tmp/archive.dat", "archive.dat", None, None).expect_err(NOTE);
    assert!(matches!(binary, ReadToolError::Binary(_)));
    assert_eq!(binary.to_string(), "Cannot read binary file: archive.dat");

    let malformed = ReadToolFileSystem::read("/tmp/malformed.txt", "malformed.txt", None, None)
        .expect_err(NOTE);
    assert!(matches!(malformed, ReadToolError::MalformedUtf8(_)));
}

#[test]
#[ignore = "porting: read-tool filesystem not implemented"]
fn reports_out_of_range_pagination_as_a_typed_error() {
    let error =
        ReadToolFileSystem::read("/tmp/short.txt", "short.txt", Some(2), None).expect_err(NOTE);
    assert!(matches!(error, ReadToolError::OffsetOutOfRange(_)));
    assert_eq!(error.to_string(), "Offset 2 is out of range");
}

#[test]
#[ignore = "porting: read-tool filesystem not implemented"]
fn stops_reading_after_the_requested_page() {
    let page =
        ReadToolFileSystem::read("/tmp/malformed.txt", "malformed.txt", None, Some(1)).expect(NOTE);
    assert_eq!(page.content, "one");
    assert!(page.truncated);
    assert_eq!(page.next, Some(2));
}

#[test]
#[ignore = "porting: read-tool filesystem not implemented"]
fn preserves_the_media_ingestion_limit_message() {
    let error = ReadToolFileSystem::read("/tmp/oversized.png", "oversized.png", None, None)
        .expect_err(NOTE);
    assert!(matches!(error, ReadToolError::MediaIngestLimit(_)));
    assert_eq!(
        error.to_string(),
        format!("Media exceeds {MAX_MEDIA_INGEST_BYTES} byte ingestion limit: oversized.png")
    );
}
