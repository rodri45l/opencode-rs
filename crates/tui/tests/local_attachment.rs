//! Port of packages/tui/test/prompt/local-attachment.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/component/prompt/local-attachment.ts; see docs/TEST-PORT.md.

use opencode_tui::local_attachment::{
    read_local_attachment_with, AttachmentContent, LocalAttachment, LocalFiles,
};

#[derive(Clone)]
struct Files {
    mime: Result<String, ()>,
    text: Result<String, ()>,
    bytes: Result<Vec<u8>, ()>,
}

impl LocalFiles for Files {
    fn mime(&self, _path: &str) -> Result<String, ()> {
        self.mime.clone()
    }
    fn read_text(&self, _path: &str) -> Result<String, ()> {
        self.text.clone()
    }
    fn read_bytes(&self, _path: &str) -> Result<Vec<u8>, ()> {
        self.bytes.clone()
    }
}

#[test]
fn reads_svg_attachments_as_text() {
    let files = Files {
        mime: Ok("image/svg+xml".to_string()),
        text: Ok("<svg />".to_string()),
        bytes: Ok(Vec::new()),
    };
    assert_eq!(
        read_local_attachment_with(&files, "/tmp/image.svg"),
        Some(LocalAttachment {
            kind: "text".to_string(),
            mime: "image/svg+xml".to_string(),
            content: AttachmentContent::Text("<svg />".to_string()),
        })
    );
}

#[test]
fn reads_image_and_pdf_attachments_as_bytes() {
    let files = Files {
        mime: Ok("application/pdf".to_string()),
        text: Ok(String::new()),
        bytes: Ok(vec![1, 2, 3]),
    };
    assert_eq!(
        read_local_attachment_with(&files, "/tmp/file.pdf"),
        Some(LocalAttachment {
            kind: "binary".to_string(),
            mime: "application/pdf".to_string(),
            content: AttachmentContent::Binary(vec![1, 2, 3]),
        })
    );
}

#[test]
fn ignores_unsupported_and_unreadable_local_files() {
    let unsupported = Files {
        mime: Ok("text/plain".to_string()),
        text: Ok(String::new()),
        bytes: Ok(Vec::new()),
    };
    assert_eq!(
        read_local_attachment_with(&unsupported, "/tmp/file.txt"),
        None
    );

    let unreadable = Files {
        mime: Ok("image/png".to_string()),
        text: Ok(String::new()),
        bytes: Err(()),
    };
    assert_eq!(
        read_local_attachment_with(&unreadable, "/tmp/missing.png"),
        None
    );
}
