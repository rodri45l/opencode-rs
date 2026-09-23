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
//! real PNG decoding and image resizing are dropped; the decision logic is
//! exercised as a pure model in `opencode_core::read_tool`.

use opencode_core::read_tool::{FileContent, ImageConfig, ReadDecision, ReadTarget, ReadTool};
use serde_json::json;

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
        ReadTool::execute(&target, &ImageConfig::default(), false, true),
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
fn asks_for_external_directory_approval_before_reading_an_external_path() {
    assert_eq!(
        ReadTool::external_actions("/outside/notes.txt", true),
        vec!["external_directory".to_string(), "read".to_string()]
    );
    assert_eq!(
        ReadTool::external_actions("README.md", false),
        vec!["read".to_string()]
    );
}

#[test]
fn returns_a_small_png_as_native_media() {
    let decision = ReadTool::execute(
        &png("pixel.png", PIXEL),
        &ImageConfig::default(),
        false,
        true,
    );
    match decision {
        ReadDecision::Content { text, media } => {
            assert_eq!(text, "Image read successfully");
            let media = media.expect("media");
            assert_eq!(media.mime, "image/png");
            assert_eq!(media.uri, format!("data:image/png;base64,{PIXEL}"));
            assert_eq!(media.name, "pixel.png");
        }
        other => panic!("unexpected decision: {other:?}"),
    }
}

#[test]
fn returns_supported_image_contents_despite_a_misleading_binary_extension() {
    let decision = ReadTool::execute(
        &png("pixel.bin", PIXEL),
        &ImageConfig::default(),
        false,
        true,
    );
    match decision {
        ReadDecision::Content { media, .. } => {
            let media = media.expect("media");
            assert_eq!(media.mime, "image/png");
            assert_eq!(media.name, "pixel.bin");
        }
        other => panic!("unexpected decision: {other:?}"),
    }
}

#[test]
fn rejects_invalid_image_data_returned_by_the_filesystem() {
    assert_eq!(
        ReadTool::execute(
            &png("truncated.png", "iVBORw0KGgo="),
            &ImageConfig::default(),
            false,
            true
        ),
        ReadDecision::Error("Image could not be decoded: truncated.png".into())
    );
}

#[test]
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
    let decision = ReadTool::execute(&wide, &config, false, true);
    match decision {
        ReadDecision::Error(value) => assert!(value.contains("exceeding configured limits 4x2000")),
        other => panic!("unexpected decision: {other:?}"),
    }
}

#[test]
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
    match ReadTool::execute(&wide, &config, false, true) {
        ReadDecision::Content { media, .. } => {
            let media = media.expect("media");
            assert!(media.width <= 4);
            assert!(media.height <= 2_000);
        }
        other => panic!("unexpected decision: {other:?}"),
    }
}

#[test]
fn enforces_max_base64_bytes_after_resize_attempts() {
    let config = ImageConfig {
        auto_resize: true,
        max_width: 2_000,
        max_height: 2_000,
        max_base64_bytes: Some(1),
    };
    match ReadTool::execute(&png("pixel.png", PIXEL), &config, false, true) {
        ReadDecision::Error(value) => assert!(value.contains("/1 bytes")),
        other => panic!("unexpected decision: {other:?}"),
    }
}

#[test]
fn returns_expected_filesystem_failures_to_the_model() {
    assert_eq!(
        ReadTool::execute(&ReadTarget::Binary, &ImageConfig::default(), false, true),
        ReadDecision::Error("Cannot read binary file: archive.dat".into())
    );
}

#[test]
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
        ReadTool::execute(&target, &ImageConfig::default(), true, true),
        ReadDecision::Error("Unable to read README.md".into())
    );
}

#[test]
fn returns_missing_paths_as_model_visible_failures() {
    assert_eq!(
        ReadTool::execute(&ReadTarget::Missing, &ImageConfig::default(), false, true),
        ReadDecision::Error("Unable to read __missing_read_target__.txt".into())
    );
}

#[test]
fn lists_a_bounded_directory_page_through_read() {
    assert_eq!(
        ReadTool::execute(&ReadTarget::Directory, &ImageConfig::default(), false, true),
        ReadDecision::Json(json!({ "entries": [], "truncated": false }))
    );
}

#[test]
fn forwards_pagination_and_returns_bounded_text_pages_with_continuation() {
    assert_eq!(
        ReadTool::text_page("hello", 2, 1),
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
