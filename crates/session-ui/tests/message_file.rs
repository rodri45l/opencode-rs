//! Port of packages/session-ui/src/components/message-file.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/message-file.ts; see docs/TEST-PORT.md.

use opencode_session_ui::message_file::{attached, inline, kind, type_label, FilePart, FileSource};

fn file(url: &str, mime: &str, source: Option<FileSource>) -> FilePart {
    FilePart {
        url: url.to_string(),
        filename: Some("README.txt".to_string()),
        mime: mime.to_string(),
        source,
    }
}

fn source_file(path: &str) -> FileSource {
    FileSource {
        kind: "file".to_string(),
        path: Some(path.to_string()),
    }
}

#[test]
fn treats_data_urls_as_attachments() {
    assert!(attached(&file(
        "data:text/plain;base64,SGVsbG8=",
        "text/plain",
        None
    )));
    assert!(!attached(&file(
        "file:///repo/README.txt",
        "text/plain",
        None
    )));
}

#[test]
fn keeps_data_backed_file_mentions_inline() {
    assert!(inline(&file(
        "file:///repo/README.txt",
        "text/plain",
        Some(source_file("/repo/README.txt"))
    )));

    let mentioned = file(
        "data:text/plain;base64,SGVsbG8=",
        "text/plain",
        Some(source_file("/repo/README.txt")),
    );
    assert!(inline(&mentioned));
    assert!(!attached(&mentioned));
}

#[test]
fn separates_image_and_file_attachment_kinds() {
    assert_eq!(kind(&file("file:///x", "image/png", None)), "image");
    assert_eq!(kind(&file("file:///x", "application/pdf", None)), "file");
}

#[test]
fn labels_attachment_types_from_the_basename_extension() {
    assert_eq!(type_label("list.md", "text/plain", "File"), "Markdown");
    assert_eq!(
        type_label("/repo/src/main.ts", "text/plain", "File"),
        "TypeScript"
    );
    assert_eq!(
        type_label("/tmp/report.pdf", "application/pdf", "File"),
        "PDF"
    );
    assert_eq!(type_label("notes.xyz", "text/plain", "File"), "XYZ");
    assert_eq!(
        type_label("/home/user/my.project/Makefile", "text/plain", "File"),
        "File"
    );
    assert_eq!(type_label(".gitignore", "text/plain", "File"), "File");
    assert_eq!(type_label("/repo/.env", "text/plain", "File"), "File");
}
