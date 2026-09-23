//! Local attachment reading.
//!
//! Port of packages/tui/src/component/prompt/local-attachment.ts
//! `readLocalAttachmentWith` behaviour (upstream 18ef3cc). Re-derived with
//! synchronous result-returning files instead of promises.

/// The content of a local attachment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachmentContent {
    Text(String),
    Binary(Vec<u8>),
}

/// A read local attachment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalAttachment {
    pub kind: String,
    pub mime: String,
    pub content: AttachmentContent,
}

/// The file system surface used to read an attachment.
#[allow(clippy::result_unit_err)]
pub trait LocalFiles {
    fn mime(&self, path: &str) -> Result<String, ()>;
    fn read_text(&self, path: &str) -> Result<String, ()>;
    fn read_bytes(&self, path: &str) -> Result<Vec<u8>, ()>;
}

/// Read a supported local attachment, or `None` when unsupported/unreadable.
pub fn read_local_attachment_with(files: &impl LocalFiles, path: &str) -> Option<LocalAttachment> {
    let mime = files.mime(path).ok()?;
    if mime == "image/svg+xml" {
        let content = files.read_text(path).ok()?;
        if content.is_empty() {
            return None;
        }
        return Some(LocalAttachment {
            kind: "text".to_string(),
            mime,
            content: AttachmentContent::Text(content),
        });
    }
    if !mime.starts_with("image/") && mime != "application/pdf" {
        return None;
    }
    let content = files.read_bytes(path).ok()?;
    if content.is_empty() {
        return None;
    }
    Some(LocalAttachment {
        kind: "binary".to_string(),
        mime,
        content: AttachmentContent::Binary(content),
    })
}
