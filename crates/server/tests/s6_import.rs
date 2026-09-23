//! Port of packages/opencode/test/cli/import.test.ts (upstream 18ef3cc).
//!
//! RED-first: `cli/cmd/import` is not implemented in this crate. The reference
//! pure helpers (error formatting, share-URL parsing, auth-header scoping and
//! share-data transform) are pinned against local typed stubs.

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
enum ImportError {
    NotFound,
    PermissionDenied,
    Json(String),
}

fn format_import_file_error(_name: &str, _err: ImportError) -> String {
    String::new()
}

fn parse_share_url(_url: &str) -> Option<String> {
    None
}

fn should_attach_share_auth_headers(_url: &str, _control: &str) -> bool {
    false
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ShareKind {
    Session,
    Message,
    Part,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShareData {
    kind: ShareKind,
    id: String,
    session_id: Option<String>,
    message_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TransformedMessage {
    id: String,
    parts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TransformedShare {
    info_id: String,
    messages: Vec<TransformedMessage>,
}

fn transform_share_data(_data: &[ShareData]) -> Option<TransformedShare> {
    None
}

#[test]
#[ignore = "porting: cli import error formatting not implemented"]
fn formats_import_file_errors() {
    assert_eq!(
        format_import_file_error("test.json", ImportError::NotFound),
        "File not found: test.json"
    );
    assert_eq!(
        format_import_file_error("test.json", ImportError::PermissionDenied),
        "Failed to read file: Permission denied"
    );
    assert_eq!(
        format_import_file_error(
            "test.json",
            ImportError::Json("Unexpected token".to_string())
        ),
        "Invalid JSON in test.json: Unexpected token"
    );
}

#[test]
#[ignore = "porting: cli import parseShareUrl not implemented"]
fn parses_valid_share_urls() {
    assert_eq!(
        parse_share_url("https://opncd.ai/share/Jsj3hNIW"),
        Some("Jsj3hNIW".to_string())
    );
    assert_eq!(
        parse_share_url("https://custom.example.com/share/abc123"),
        Some("abc123".to_string())
    );
    assert_eq!(
        parse_share_url("http://localhost:3000/share/test_id-123"),
        Some("test_id-123".to_string())
    );
}

#[test]
#[ignore = "porting: cli import parseShareUrl not implemented"]
fn rejects_invalid_share_urls() {
    assert_eq!(parse_share_url("https://opncd.ai/s/Jsj3hNIW"), None);
    assert_eq!(parse_share_url("https://opncd.ai/share/"), None);
    assert_eq!(parse_share_url("https://opncd.ai/share/id/extra"), None);
    assert_eq!(parse_share_url("not-a-url"), None);
}

#[test]
#[ignore = "porting: cli import shouldAttachShareAuthHeaders not implemented"]
fn only_attaches_share_auth_headers_for_same_origin_urls() {
    assert!(should_attach_share_auth_headers(
        "https://control.example.com/share/abc",
        "https://control.example.com"
    ));
    assert!(!should_attach_share_auth_headers(
        "https://other.example.com/share/abc",
        "https://control.example.com"
    ));
    assert!(should_attach_share_auth_headers(
        "https://control.example.com:443/share/abc",
        "https://control.example.com"
    ));
    assert!(!should_attach_share_auth_headers(
        "not-a-url",
        "https://control.example.com"
    ));
}

fn session(id: &str) -> ShareData {
    ShareData {
        kind: ShareKind::Session,
        id: id.to_string(),
        session_id: None,
        message_id: None,
    }
}

fn message(id: &str, session_id: &str) -> ShareData {
    ShareData {
        kind: ShareKind::Message,
        id: id.to_string(),
        session_id: Some(session_id.to_string()),
        message_id: None,
    }
}

fn part(id: &str, message_id: &str) -> ShareData {
    ShareData {
        kind: ShareKind::Part,
        id: id.to_string(),
        session_id: None,
        message_id: Some(message_id.to_string()),
    }
}

#[test]
#[ignore = "porting: cli import transformShareData not implemented"]
fn transforms_share_data_to_storage_format() {
    let data = vec![
        session("sess-1"),
        message("msg-1", "sess-1"),
        part("part-1", "msg-1"),
        part("part-2", "msg-1"),
    ];
    let result = transform_share_data(&data).expect("transform");
    assert_eq!(result.info_id, "sess-1");
    assert_eq!(result.messages.len(), 1);
    assert_eq!(result.messages[0].parts.len(), 2);
}

#[test]
#[ignore = "porting: cli import transformShareData not implemented"]
fn returns_null_for_invalid_share_data() {
    assert_eq!(transform_share_data(&[]), None);
    assert_eq!(
        transform_share_data(&[ShareData {
            kind: ShareKind::Message,
            id: String::new(),
            session_id: None,
            message_id: None,
        }]),
        None
    );
    assert_eq!(transform_share_data(&[session("s")]), None);
}
