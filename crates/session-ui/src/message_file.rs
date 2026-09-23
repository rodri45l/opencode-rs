//! Message file-part projection.
//!
//! Port of packages/session-ui/src/components/message-file.ts behaviour
//! (upstream 18ef3cc).

/// A file part as projected by the session UI.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilePart {
    pub url: String,
    pub filename: Option<String>,
    pub mime: String,
    pub source: Option<FileSource>,
}

/// The source metadata attached to a file part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSource {
    pub kind: String,
    pub path: Option<String>,
}

/// Whether a file part is a data-URL attachment (not an inline mention).
pub fn attached(part: &FilePart) -> bool {
    part.url.starts_with("data:") && part.source.is_none()
}

/// Whether a file part is an inline file mention.
pub fn inline(part: &FilePart) -> bool {
    part.source
        .as_ref()
        .map(|source| source.kind == "file")
        .unwrap_or(false)
}

/// The attachment kind derived from the MIME type.
pub fn kind(part: &FilePart) -> &'static str {
    if part.mime.starts_with("image/") {
        "image"
    } else {
        "file"
    }
}

/// The human label for an attachment, derived from its basename extension.
pub fn type_label(filename: &str, _mime: &str, fallback: &str) -> String {
    let extension = std::path::Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str());
    match extension {
        Some("md") => "Markdown".to_string(),
        Some("ts") => "TypeScript".to_string(),
        Some("pdf") => "PDF".to_string(),
        Some(other) if !other.is_empty() => other.to_uppercase(),
        _ => fallback.to_string(),
    }
}
