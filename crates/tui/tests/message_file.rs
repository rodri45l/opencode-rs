//! Port of packages/session-ui/src/components/message-file.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/message-file.ts: data URLs
//! are attachments, data-backed file mentions stay inline, image/file kinds split by
//! mime, and attachment labels come from the basename extension.
//! Red-first: the message-file helpers are not implemented.

#[allow(dead_code)]
mod message_file {
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: session-ui message-file helpers not implemented";

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AttachmentKind {
        Image,
        File,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SourceText {
        pub value: String,
        pub start: i64,
        pub end: i64,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FileSource {
        pub path: String,
        pub text: SourceText,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct FilePart {
        pub id: String,
        pub mime: String,
        pub url: String,
        pub filename: String,
        pub source: Option<FileSource>,
    }

    pub fn attached(_part: &FilePart) -> PortResult<bool> {
        Err(NotImplemented(NOTE))
    }

    pub fn inline(_part: &FilePart) -> PortResult<bool> {
        Err(NotImplemented(NOTE))
    }

    pub fn kind(_part: &FilePart) -> PortResult<AttachmentKind> {
        Err(NotImplemented(NOTE))
    }

    pub fn type_label(_path: &str, _mime: &str, _fallback: &str) -> PortResult<String> {
        Err(NotImplemented(NOTE))
    }
}

use message_file::{
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
#[ignore = "porting: session-ui message-file helpers not implemented"]
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
#[ignore = "porting: session-ui message-file helpers not implemented"]
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
#[ignore = "porting: session-ui message-file helpers not implemented"]
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
#[ignore = "porting: session-ui message-file helpers not implemented"]
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
