//! Port of packages/session-ui/src/components/message-file.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/message-file.ts: data URLs
//! are attachments, data-backed file mentions stay inline, image/file kinds split by
//! mime, and attachment labels come from the basename extension.
//! Red-first: the message-file helpers are not implemented.

use opencode_tui::message_file::{
    attached, inline, kind, type_label, AttachmentKind, FilePart, FileSource, SourceText, NOTE,
};

fn file(url: &str, mime: &str, filename: &str) -> FilePart {
    FilePart {
        id: "part_1".into(),
        mime: mime.into(),
        url: url.into(),
        filename: filename.into(),
        source: None,
    }
}

fn mention(part: &mut FilePart) {
    part.source = Some(FileSource {
        path: "/repo/README.txt".into(),
        text: SourceText {
            value: "@README.txt".into(),
            start: 0,
            end: 11,
        },
    });
}

#[test]
fn treats_data_urls_as_attachments() {
    assert!(attached(&file(
        "data:text/plain;base64,SGVsbG8=",
        "text/plain",
        "README.txt"
    ))
    .expect(NOTE));
    assert!(!attached(&file("file:///repo/README.txt", "text/plain", "README.txt")).expect(NOTE));
}

#[test]
fn keeps_data_backed_file_mentions_inline() {
    let mut mentioned = file("file:///repo/README.txt", "text/plain", "README.txt");
    mention(&mut mentioned);
    assert!(inline(&mentioned).expect(NOTE));

    let mut data_mentioned = file(
        "data:text/plain;base64,SGVsbG8=",
        "text/plain",
        "README.txt",
    );
    mention(&mut data_mentioned);
    assert!(inline(&data_mentioned).expect(NOTE));
    assert!(!attached(&data_mentioned).expect(NOTE));
}

#[test]
fn separates_image_and_file_attachment_kinds() {
    assert_eq!(
        kind(&file("file:///a.png", "image/png", "a.png")).expect(NOTE),
        AttachmentKind::Image
    );
    assert_eq!(
        kind(&file("file:///a.pdf", "application/pdf", "a.pdf")).expect(NOTE),
        AttachmentKind::File
    );
}

#[test]
fn labels_attachment_types_from_the_basename_extension() {
    assert_eq!(
        type_label("list.md", "text/plain", "File").expect(NOTE),
        "Markdown"
    );
    assert_eq!(
        type_label("/repo/src/main.ts", "text/plain", "File").expect(NOTE),
        "TypeScript"
    );
    assert_eq!(
        type_label("/tmp/report.pdf", "application/pdf", "File").expect(NOTE),
        "PDF"
    );
    assert_eq!(
        type_label("notes.xyz", "text/plain", "File").expect(NOTE),
        "XYZ"
    );
    assert_eq!(
        type_label("/home/user/my.project/Makefile", "text/plain", "File").expect(NOTE),
        "File"
    );
    assert_eq!(
        type_label(".gitignore", "text/plain", "File").expect(NOTE),
        "File"
    );
    assert_eq!(
        type_label("/repo/.env", "text/plain", "File").expect(NOTE),
        "File"
    );
}
