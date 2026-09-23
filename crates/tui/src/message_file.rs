//! Message file helpers.
//!
//! Derived from `packages/session-ui/src/components/message-file.ts`
//! (upstream 18ef3cc): data URLs are attachments, data-backed file mentions stay
//! inline, image/file kinds split by mime, and attachment labels come from the
//! basename extension.

use std::fmt;

/// Error raised by the message-file helpers.
#[derive(Debug, PartialEq, Eq)]
pub struct NotImplemented(pub &'static str);

impl fmt::Display for NotImplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for NotImplemented {}

/// Result alias for the message-file helpers.
pub type PortResult<T> = Result<T, NotImplemented>;

/// Message used by the ported tests when expecting success.
pub const NOTE: &str = "session-ui message-file helpers";

/// Whether an attachment is an image or a generic file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentKind {
    Image,
    File,
}

/// Source text span for a file mention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceText {
    pub value: String,
    pub start: i64,
    pub end: i64,
}

/// A file mention source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSource {
    pub path: String,
    pub text: SourceText,
}

/// A file part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePart {
    pub id: String,
    pub mime: String,
    pub url: String,
    pub filename: String,
    pub source: Option<FileSource>,
}

/// Whether the part is a data-URL attachment.
pub fn attached(part: &FilePart) -> PortResult<bool> {
    Ok(part.url.starts_with("data:") && !inline(part)?)
}

/// Whether the part is an inline mention.
pub fn inline(part: &FilePart) -> PortResult<bool> {
    Ok(part.source.is_some())
}

/// The attachment kind derived from the mime type.
pub fn kind(part: &FilePart) -> PortResult<AttachmentKind> {
    Ok(if part.mime.starts_with("image/") {
        AttachmentKind::Image
    } else {
        AttachmentKind::File
    })
}

fn basename(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn language_name(suffix: &str) -> Option<&'static str> {
    Some(match suffix {
        "md" | "markdown" => "Markdown",
        "mdx" => "MDX",
        "ts" | "mts" | "cts" => "TypeScript",
        "tsx" => "TSX",
        "js" | "mjs" | "cjs" => "JavaScript",
        "jsx" => "JSX",
        "json" | "jsonc" => "JSON",
        "css" => "CSS",
        "scss" => "SCSS",
        "sass" => "Sass",
        "less" => "Less",
        "html" | "htm" => "HTML",
        "py" => "Python",
        "rb" => "Ruby",
        "rs" => "Rust",
        "go" => "Go",
        "java" => "Java",
        "kt" | "kts" => "Kotlin",
        "sh" | "bash" | "zsh" | "fish" => "Shell",
        "yml" | "yaml" => "YAML",
        "toml" => "TOML",
        "sql" => "SQL",
        "c" => "C",
        "cpp" | "cc" | "cxx" => "C++",
        "cs" => "C#",
        "php" => "PHP",
        "swift" => "Swift",
        "vue" => "Vue",
        "svelte" => "Svelte",
        "xml" => "XML",
        "svg" => "SVG",
        "graphql" | "gql" => "GraphQL",
        "txt" => "Text",
        "lock" => "Lock",
        _ => return None,
    })
}

/// A human label for an attachment derived from its basename extension.
pub fn type_label(path: &str, mime: &str, fallback: &str) -> PortResult<String> {
    if mime == "application/pdf" {
        return Ok("PDF".to_string());
    }
    let base = basename(path);
    let index = base.rfind('.').map(|value| value as i64).unwrap_or(-1);
    let suffix = if index <= 0 {
        String::new()
    } else {
        base[index as usize + 1..].to_lowercase()
    };
    if suffix.is_empty() {
        return Ok(fallback.to_string());
    }
    Ok(language_name(&suffix)
        .map(str::to_string)
        .unwrap_or_else(|| suffix.to_uppercase()))
}
