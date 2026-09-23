//! Port of packages/core/test/tool-read.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `read` registers, authorizes and reads through the
//! location filesystem; an external absolute path requires
//! `external_directory` approval before `read`; a small supported image is
//! returned as native media (`Image read successfully` plus a data URI) rather
//! than durable base64 text, including when a misleading binary extension is
//! present; invalid image data fails `Image could not be decoded: <name>`;
//! oversized images fail with `exceeding configured limits WxH` when resizing
//! is disabled and are resized otherwise; `max_base64_bytes` fails with
//! `/<n> bytes`; binary files fail `Cannot read binary file: <path>`; denied
//! permission and missing paths fail `Unable to read <path>`; directories list
//! a bounded page; and text pages forward pagination with a continuation.
//! Re-derived: the `ToolRegistry`/`PermissionV2`/`Image`/`FSUtil` layer graph,
//! real PNG decoding and `@silvia-odwyer/photon-node` resizing are dropped; the
//! decision logic is exercised as a pure model.

#![allow(dead_code)]

const NOTE: &str = "porting: read tool not implemented";

use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileContent {
    pub uri: String,
    pub name: String,
    pub content: String,
    pub encoding: String,
    pub mime: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadTarget {
    File(FileContent),
    Directory,
    Missing,
    Binary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageConfig {
    pub auto_resize: bool,
    pub max_width: u32,
    pub max_height: u32,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Media {
    pub mime: String,
    pub uri: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReadDecision {
    Json(Value),
    Content { text: String, media: Option<Media> },
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortError {
    NotImplemented(&'static str),
}

impl std::fmt::Display for PortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortError::NotImplemented(topic) => write!(f, "not implemented: {topic}"),
        }
    }
}

impl std::error::Error for PortError {}

pub struct ReadTool;

impl ReadTool {
    pub fn execute(
        _target: &ReadTarget,
        _config: &ImageConfig,
        _permission_denied: bool,
        _resizer_available: bool,
    ) -> Result<ReadDecision, PortError> {
        Err(PortError::NotImplemented(NOTE))
    }

    pub fn external_actions(_path: &str, _external: bool) -> Result<Vec<String>, PortError> {
        Err(PortError::NotImplemented(NOTE))
    }

    pub fn text_page(_content: &str, _offset: u32, _limit: u32) -> Result<Value, PortError> {
        Err(PortError::NotImplemented(NOTE))
    }
}

fn png(name: &str, content: &str) -> ReadTarget {
    ReadTarget::File(FileContent {
        uri: format!("file:///{name}"),
        name: name.into(),
        content: content.into(),
        encoding: "base64".into(),
        mime: "image/png".into(),
        width: Some(1),
        height: Some(1),
    })
}

const PIXEL: &str =
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=";

#[test]
#[ignore = "porting: read tool not implemented"]
fn registers_authorizes_and_reads_through_the_location_filesystem() {
    let target = ReadTarget::File(FileContent {
        uri: "file:///README.md".into(),
        name: "README.md".into(),
        content: "hello".into(),
        encoding: "utf8".into(),
        mime: "text/plain".into(),
        width: None,
        height: None,
    });
    assert_eq!(
        ReadTool::execute(&target, &ImageConfig::default(), false, true).expect(NOTE),
        ReadDecision::Json(json!({
            "uri": "file:///README.md",
            "name": "README.md",
            "content": "hello",
            "encoding": "utf8",
            "mime": "text/plain"
        }))
    );
}

#[test]
#[ignore = "porting: read tool not implemented"]
fn asks_for_external_directory_approval_before_reading_an_external_path() {
    assert_eq!(
        ReadTool::external_actions("/outside/notes.txt", true).expect(NOTE),
        vec!["external_directory".to_string(), "read".to_string()]
    );
    assert_eq!(
        ReadTool::external_actions("README.md", false).expect(NOTE),
        vec!["read".to_string()]
    );
}

#[test]
#[ignore = "porting: read tool not implemented"]
fn returns_a_small_png_as_native_media() {
    let decision = ReadTool::execute(
        &png("pixel.png", PIXEL),
        &ImageConfig::default(),
        false,
        true,
    )
    .expect(NOTE);
    match decision {
        ReadDecision::Content { text, media } => {
            assert_eq!(text, "Image read successfully");
            let media = media.expect(NOTE);
            assert_eq!(media.mime, "image/png");
            assert_eq!(media.uri, format!("data:image/png;base64,{PIXEL}"));
            assert_eq!(media.name, "pixel.png");
        }
        other => panic!("unexpected decision: {other:?}"),
    }
}

#[test]
#[ignore = "porting: read tool not implemented"]
fn returns_supported_image_contents_despite_a_misleading_binary_extension() {
    let decision = ReadTool::execute(
        &png("pixel.bin", PIXEL),
        &ImageConfig::default(),
        false,
        true,
    )
    .expect(NOTE);
    match decision {
        ReadDecision::Content { media, .. } => {
            let media = media.expect(NOTE);
            assert_eq!(media.mime, "image/png");
            assert_eq!(media.name, "pixel.bin");
        }
        other => panic!("unexpected decision: {other:?}"),
    }
}

#[test]
#[ignore = "porting: read tool not implemented"]
fn rejects_invalid_image_data_returned_by_the_filesystem() {
    assert_eq!(
        ReadTool::execute(
            &png("truncated.png", "iVBORw0KGgo="),
            &ImageConfig::default(),
            false,
            true
        )
        .expect(NOTE),
        ReadDecision::Error("Image could not be decoded: truncated.png".into())
    );
}

#[test]
#[ignore = "porting: read tool not implemented"]
fn rejects_oversized_images_when_resizing_is_disabled() {
    let config = ImageConfig {
        auto_resize: false,
        max_width: 4,
        max_height: 2_000,
        max_base64_bytes: None,
    };
    let wide = ReadTarget::File(FileContent {
        uri: "file:///wide.png".into(),
        name: "wide.png".into(),
        content: PIXEL.into(),
        encoding: "base64".into(),
        mime: "image/png".into(),
        width: Some(16),
        height: Some(1),
    });
    let decision = ReadTool::execute(&wide, &config, false, true).expect(NOTE);
    match decision {
        ReadDecision::Error(value) => assert!(value.contains("exceeding configured limits 4x2000")),
        other => panic!("unexpected decision: {other:?}"),
    }
}

#[test]
#[ignore = "porting: read tool not implemented"]
fn resizes_images_to_configured_dimensions_before_returning_media() {
    let config = ImageConfig {
        auto_resize: true,
        max_width: 4,
        max_height: 2_000,
        max_base64_bytes: None,
    };
    let wide = ReadTarget::File(FileContent {
        uri: "file:///wide.png".into(),
        name: "wide.png".into(),
        content: PIXEL.into(),
        encoding: "base64".into(),
        mime: "image/png".into(),
        width: Some(16),
        height: Some(1),
    });
    match ReadTool::execute(&wide, &config, false, true).expect(NOTE) {
        ReadDecision::Content { media, .. } => {
            let media = media.expect(NOTE);
            assert!(media.width <= 4);
            assert!(media.height <= 2_000);
        }
        other => panic!("unexpected decision: {other:?}"),
    }
}

#[test]
#[ignore = "porting: read tool not implemented"]
fn enforces_max_base64_bytes_after_resize_attempts() {
    let config = ImageConfig {
        auto_resize: true,
        max_width: 2_000,
        max_height: 2_000,
        max_base64_bytes: Some(1),
    };
    match ReadTool::execute(&png("pixel.png", PIXEL), &config, false, true).expect(NOTE) {
        ReadDecision::Error(value) => assert!(value.contains("/1 bytes")),
        other => panic!("unexpected decision: {other:?}"),
    }
}

#[test]
#[ignore = "porting: read tool not implemented"]
fn returns_expected_filesystem_failures_to_the_model() {
    assert_eq!(
        ReadTool::execute(&ReadTarget::Binary, &ImageConfig::default(), false, true).expect(NOTE),
        ReadDecision::Error("Cannot read binary file: archive.dat".into())
    );
}

#[test]
#[ignore = "porting: read tool not implemented"]
fn does_not_read_when_permission_is_denied() {
    let target = ReadTarget::File(FileContent {
        uri: "file:///README.md".into(),
        name: "README.md".into(),
        content: "hello".into(),
        encoding: "utf8".into(),
        mime: "text/plain".into(),
        width: None,
        height: None,
    });
    assert_eq!(
        ReadTool::execute(&target, &ImageConfig::default(), true, true).expect(NOTE),
        ReadDecision::Error("Unable to read README.md".into())
    );
}

#[test]
#[ignore = "porting: read tool not implemented"]
fn returns_missing_paths_as_model_visible_failures() {
    assert_eq!(
        ReadTool::execute(&ReadTarget::Missing, &ImageConfig::default(), false, true).expect(NOTE),
        ReadDecision::Error("Unable to read __missing_read_target__.txt".into())
    );
}

#[test]
#[ignore = "porting: read tool not implemented"]
fn lists_a_bounded_directory_page_through_read() {
    assert_eq!(
        ReadTool::execute(&ReadTarget::Directory, &ImageConfig::default(), false, true)
            .expect(NOTE),
        ReadDecision::Json(json!({ "entries": [], "truncated": false }))
    );
}

#[test]
#[ignore = "porting: read tool not implemented"]
fn forwards_pagination_and_returns_bounded_text_pages_with_continuation() {
    assert_eq!(
        ReadTool::text_page("hello", 2, 1).expect(NOTE),
        json!({
            "type": "text-page",
            "content": "hello",
            "mime": "text/plain",
            "offset": 2,
            "truncated": true,
            "next": 3
        })
    );
}
