//! Port of packages/opencode/test/acp/content.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/acp/content.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure content-block projections `contentBlockToParts`,
//! `promptContentToParts`, and `partsToContentChunks`.
//! Dropped: none — every upstream case is pure. `pathToFileURL` is a Node
//! helper; the Rust port asserts the platform-independent URL shape.

use opencode_server::acp_content::{
    content_block_to_parts, parts_to_content_chunks, prompt_content_to_parts,
};
use serde_json::json;

#[test]
fn plain_text_block_becomes_a_text_part() {
    assert_eq!(
        content_block_to_parts(&json!({ "type": "text", "text": "hello" })).unwrap(),
        json!([{ "type": "text", "text": "hello" }])
    );
}

#[test]
fn assistant_only_text_audience_becomes_synthetic() {
    assert_eq!(
        content_block_to_parts(&json!({
            "type": "text",
            "text": "internal",
            "annotations": { "audience": ["assistant"] }
        }))
        .unwrap(),
        json!([{ "type": "text", "text": "internal", "synthetic": true }])
    );
}

#[test]
fn user_only_text_audience_becomes_ignored() {
    assert_eq!(
        content_block_to_parts(&json!({
            "type": "text",
            "text": "visible to user",
            "annotations": { "audience": ["user"] }
        }))
        .unwrap(),
        json!([{ "type": "text", "text": "visible to user", "ignored": true }])
    );
}

#[test]
fn image_block_with_base64_data_becomes_a_data_url_file_part() {
    assert_eq!(
        content_block_to_parts(&json!({
            "type": "image",
            "data": "AAAA",
            "mimeType": "image/png",
            "uri": "file:///tmp/screenshot.png"
        }))
        .unwrap(),
        json!([{
            "type": "file",
            "url": "data:image/png;base64,AAAA",
            "filename": "screenshot.png",
            "mime": "image/png"
        }])
    );
}

#[test]
fn image_block_with_http_uri_becomes_a_file_part() {
    assert_eq!(
        content_block_to_parts(&json!({
            "type": "image",
            "data": "",
            "mimeType": "image/jpeg",
            "uri": "http://example.com/assets/photo.jpg"
        }))
        .unwrap(),
        json!([{
            "type": "file",
            "url": "http://example.com/assets/photo.jpg",
            "filename": "photo.jpg",
            "mime": "image/jpeg"
        }])
    );
}

#[test]
fn resource_link_file_url_becomes_a_file_part_with_name_and_fallback_mime() {
    assert_eq!(
        content_block_to_parts(&json!({
            "type": "resource_link",
            "uri": "file:///tmp/notes.txt",
            "name": "client-notes.txt"
        }))
        .unwrap(),
        json!([{
            "type": "file",
            "url": "file:///tmp/notes.txt",
            "filename": "client-notes.txt",
            "mime": "text/plain"
        }])
    );
}

#[test]
fn resource_link_zed_path_becomes_a_file_url_part() {
    let result = content_block_to_parts(&json!({
        "type": "resource_link",
        "uri": "zed://workspace?path=/tmp/project/src/app.ts",
        "name": "app.ts",
        "mimeType": "text/typescript"
    }))
    .unwrap();
    let part = &result[0];
    assert_eq!(part["type"], json!("file"));
    assert_eq!(part["filename"], json!("app.ts"));
    assert_eq!(part["mime"], json!("text/typescript"));
    let url = part["url"].as_str().unwrap();
    assert!(url.starts_with("file://"));
    assert!(url.ends_with("/tmp/project/src/app.ts"));
}

#[test]
fn resource_with_text_becomes_a_sourced_text_part() {
    let result = content_block_to_parts(&json!({
        "type": "resource",
        "resource": {
            "uri": "file:///tmp/context.txt#L12-L14",
            "mimeType": "text/plain",
            "text": "context"
        }
    }))
    .unwrap();
    let parts = result.as_array().unwrap();
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0]["type"], json!("text"));
    let text = parts[0]["text"].as_str().unwrap();
    assert!(text.ends_with("\ncontext"));
    assert!(text.contains("context.txt"));
    assert!(text.contains("12"));
}

#[test]
fn resource_with_text_uses_uri_fallback_for_non_file_resources() {
    assert_eq!(
        content_block_to_parts(&json!({
            "type": "resource",
            "resource": { "uri": "mcp://server/context", "text": "context" }
        }))
        .unwrap(),
        json!([{ "type": "text", "text": "[mcp://server/context]\ncontext" }])
    );
}

#[test]
fn resource_with_text_includes_file_path() {
    let result = content_block_to_parts(&json!({
        "type": "resource",
        "resource": {
            "uri": "file:///tmp/context.txt",
            "mimeType": "text/plain",
            "text": "context"
        }
    }))
    .unwrap();
    let parts = result.as_array().unwrap();
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0]["type"], json!("text"));
    let text = parts[0]["text"].as_str().unwrap();
    assert!(text.ends_with("\ncontext"));
    assert!(text.contains("context.txt"));
}

#[test]
fn resource_with_blob_and_mime_type_becomes_a_data_url_file_part() {
    assert_eq!(
        content_block_to_parts(&json!({
            "type": "resource",
            "resource": {
                "uri": "file:///tmp/report.pdf",
                "mimeType": "application/pdf",
                "blob": "JVBERg=="
            }
        }))
        .unwrap(),
        json!([{
            "type": "file",
            "url": "data:application/pdf;base64,JVBERg==",
            "filename": "report.pdf",
            "mime": "application/pdf"
        }])
    );
}

#[test]
fn data_url_resource_is_preserved_as_a_file_part() {
    assert_eq!(
        content_block_to_parts(&json!({
            "type": "resource",
            "resource": {
                "uri": "data:text/plain;base64,aGVsbG8=",
                "mimeType": "text/plain",
                "blob": "ignored"
            }
        }))
        .unwrap(),
        json!([{
            "type": "file",
            "url": "data:text/plain;base64,aGVsbG8=",
            "filename": "file",
            "mime": "text/plain"
        }])
    );
}

#[test]
fn unsupported_blocks_are_ignored() {
    assert_eq!(
        prompt_content_to_parts(
            &json!([{ "type": "audio", "data": "AAAA", "mimeType": "audio/wav" }])
        )
        .unwrap(),
        json!([])
    );
    assert_eq!(
        prompt_content_to_parts(&json!([{ "type": "unknown", "text": "skip" }])).unwrap(),
        json!([])
    );
}

#[test]
fn replays_text_audience_annotations() {
    assert_eq!(
        parts_to_content_chunks(&json!([{ "type": "text", "text": "cached", "synthetic": true }]))
            .unwrap(),
        json!([{
            "content": {
                "type": "text",
                "text": "cached",
                "annotations": { "audience": ["assistant"] }
            }
        }])
    );
}

#[test]
fn replays_file_and_data_url_parts_as_acp_content() {
    let result = parts_to_content_chunks(&json!([
        { "type": "file", "url": "file:///tmp/readme.md", "filename": "readme.md", "mime": "text/markdown" },
        { "type": "file", "url": "data:text/plain;base64,aGVsbG8=", "filename": "note.txt", "mime": "text/plain" }
    ]))
    .unwrap();
    let chunks = result.as_array().unwrap();
    assert_eq!(chunks.len(), 2);
    assert_eq!(
        chunks[0],
        json!({
            "content": {
                "type": "resource_link",
                "uri": "file:///tmp/readme.md",
                "name": "readme.md",
                "mimeType": "text/markdown"
            }
        })
    );
    assert_eq!(chunks[1]["content"]["type"], json!("resource"));
    assert_eq!(
        chunks[1]["content"]["resource"]["mimeType"],
        json!("text/plain")
    );
    assert_eq!(chunks[1]["content"]["resource"]["text"], json!("hello"));
    let uri = chunks[1]["content"]["resource"]["uri"].as_str().unwrap();
    assert!(uri.starts_with("file://"));
    assert!(uri.ends_with("note.txt"));
}
