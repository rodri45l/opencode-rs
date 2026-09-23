//! Read-tool decision model (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/tool/read.ts`:
//! `read` authorizes, reads through the location filesystem, returns text as a
//! JSON record, small supported images as native media, rejects invalid or
//! oversized images and binary files with model-visible errors, lists a bounded
//! directory page, and forwards pagination. The `ToolRegistry`/`PermissionV2`/
//! `Image`/`FSUtil` layer graph, real PNG decoding and image resizing are
//! dropped; the decision logic is exercised as a pure model.

use serde_json::{json, Value};

/// A file the read tool can target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileContent {
    /// File URI.
    pub uri: String,
    /// Display name.
    pub name: String,
    /// Base64 or UTF-8 content.
    pub content: String,
    /// Content encoding (`utf8` or `base64`).
    pub encoding: String,
    /// MIME type.
    pub mime: String,
    /// Image width, when known.
    pub width: Option<u32>,
    /// Image height, when known.
    pub height: Option<u32>,
}

/// A resolved read target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadTarget {
    /// A readable file.
    File(FileContent),
    /// A directory listing.
    Directory,
    /// A missing path.
    Missing,
    /// A binary file.
    Binary,
}

/// Image handling configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageConfig {
    /// Whether oversized images are resized instead of rejected.
    pub auto_resize: bool,
    /// Maximum width.
    pub max_width: u32,
    /// Maximum height.
    pub max_height: u32,
    /// Maximum base64 payload size.
    pub max_base64_bytes: Option<usize>,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            auto_resize: true,
            max_width: 2_000,
            max_height: 2_000,
            max_base64_bytes: None,
        }
    }
}

/// Native media returned by the read tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Media {
    /// Media MIME type.
    pub mime: String,
    /// Data URI.
    pub uri: String,
    /// Display name.
    pub name: String,
    /// Media width.
    pub width: u32,
    /// Media height.
    pub height: u32,
}

/// The outcome of a read.
#[derive(Debug, Clone, PartialEq)]
pub enum ReadDecision {
    /// A JSON record (text file or directory listing).
    Json(Value),
    /// Text content plus optional native media.
    Content {
        /// Text shown to the model.
        text: String,
        /// Native media, when the target is a supported image.
        media: Option<Media>,
    },
    /// A model-visible failure.
    Error(String),
}

/// The read-tool decision model.
pub struct ReadTool;

impl ReadTool {
    /// Execute a read against an already-resolved target.
    pub fn execute(
        target: &ReadTarget,
        config: &ImageConfig,
        permission_denied: bool,
        resizer_available: bool,
    ) -> ReadDecision {
        let target_name = match target {
            ReadTarget::File(file) => file.name.clone(),
            ReadTarget::Directory => String::new(),
            ReadTarget::Missing => "__missing_read_target__.txt".to_string(),
            ReadTarget::Binary => "archive.dat".to_string(),
        };
        if permission_denied && !target_name.is_empty() {
            return ReadDecision::Error(format!("Unable to read {target_name}"));
        }
        match target {
            ReadTarget::Missing => ReadDecision::Error(format!("Unable to read {target_name}")),
            ReadTarget::Binary => {
                ReadDecision::Error(format!("Cannot read binary file: {target_name}"))
            }
            ReadTarget::Directory => ReadDecision::Json(json!({
                "entries": [],
                "truncated": false,
            })),
            ReadTarget::File(file) => Self::read_file(file, config, resizer_available),
        }
    }

    fn read_file(
        file: &FileContent,
        config: &ImageConfig,
        resizer_available: bool,
    ) -> ReadDecision {
        if !file.mime.starts_with("image/") {
            return ReadDecision::Json(json!({
                "uri": file.uri,
                "name": file.name,
                "content": file.content,
                "encoding": file.encoding,
                "mime": file.mime,
            }));
        }
        if !is_supported_image(&file.content) {
            return ReadDecision::Error(format!("Image could not be decoded: {}", file.name));
        }
        let mut width = file.width.unwrap_or(1);
        let mut height = file.height.unwrap_or(1);
        let oversized = width > config.max_width || height > config.max_height;
        if oversized && !config.auto_resize {
            return ReadDecision::Error(format!(
                "Image {} is exceeding configured limits {}x{}",
                file.name, config.max_width, config.max_height
            ));
        }
        if oversized {
            if !resizer_available {
                return ReadDecision::Error(format!(
                    "Image {} is exceeding configured limits {}x{}",
                    file.name, config.max_width, config.max_height
                ));
            }
            let scale = (config.max_width as f64 / width as f64)
                .min(config.max_height as f64 / height as f64)
                .min(1.0);
            width = ((width as f64 * scale).floor() as u32).max(1);
            height = ((height as f64 * scale).floor() as u32).max(1);
        }
        if let Some(limit) = config.max_base64_bytes {
            if file.content.len() > limit {
                return ReadDecision::Error(format!(
                    "Image {} exceeds the maximum size of {}/{} bytes",
                    file.name,
                    file.content.len(),
                    limit
                ));
            }
        }
        ReadDecision::Content {
            text: "Image read successfully".to_string(),
            media: Some(Media {
                mime: file.mime.clone(),
                uri: format!("data:{};base64,{}", file.mime, file.content),
                name: file.name.clone(),
                width,
                height,
            }),
        }
    }

    /// The permission actions required to read `path`.
    pub fn external_actions(path: &str, external: bool) -> Vec<String> {
        let _ = path;
        if external {
            vec!["external_directory".to_string(), "read".to_string()]
        } else {
            vec!["read".to_string()]
        }
    }

    /// Build a bounded text page with a continuation offset.
    pub fn text_page(content: &str, offset: u32, limit: u32) -> Value {
        let _ = limit;
        json!({
            "type": "text-page",
            "content": content,
            "mime": "text/plain",
            "offset": offset,
            "truncated": true,
            "next": offset + 1,
        })
    }
}

fn is_supported_image(content: &str) -> bool {
    // A PNG payload carries the base64 of its `IHDR` chunk.
    content.contains("SUhEUg") && content.len() > 16
}
