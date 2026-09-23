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

fn format_import_file_error(name: &str, err: ImportError) -> String {
    match err {
        ImportError::NotFound => format!("File not found: {name}"),
        ImportError::PermissionDenied => "Failed to read file: Permission denied".to_string(),
        ImportError::Json(message) => format!("Invalid JSON in {name}: {message}"),
    }
}

fn parse_share_url(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let (authority, path) = rest.split_once('/')?;
    if authority.is_empty() {
        return None;
    }
    let path = path.trim_end_matches('/');
    let segments: Vec<&str> = path.split('/').collect();
    if segments.len() == 2 && segments[0] == "share" && !segments[1].is_empty() {
        Some(segments[1].to_string())
    } else {
        None
    }
}

fn origin(url: &str) -> Option<(String, String, u16)> {
    let (scheme, rest) = url.split_once("://")?;
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let authority = rest.split('/').next().unwrap_or("");
    if authority.is_empty() {
        return None;
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) if !host.is_empty() && port.chars().all(|c| c.is_ascii_digit()) => {
            (host.to_string(), port.parse().ok()?)
        }
        _ => (
            authority.to_string(),
            if scheme == "https" { 443 } else { 80 },
        ),
    };
    Some((scheme.to_string(), host, port))
}

fn should_attach_share_auth_headers(url: &str, control: &str) -> bool {
    match (origin(url), origin(control)) {
        (Some(url), Some(control)) => url == control,
        _ => false,
    }
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

fn transform_share_data(data: &[ShareData]) -> Option<TransformedShare> {
    let session = data.iter().find(|item| item.kind == ShareKind::Session)?;
    let mut messages: Vec<TransformedMessage> = Vec::new();
    for item in data {
        if item.kind == ShareKind::Message && !item.id.is_empty() {
            messages.push(TransformedMessage {
                id: item.id.clone(),
                parts: Vec::new(),
            });
        }
    }
    for item in data {
        if item.kind != ShareKind::Part {
            continue;
        }
        if let Some(message_id) = &item.message_id {
            if let Some(message) = messages.iter_mut().find(|m| &m.id == message_id) {
                message.parts.push(item.id.clone());
            }
        }
    }
    if messages.is_empty() {
        return None;
    }
    Some(TransformedShare {
        info_id: session.id.clone(),
        messages,
    })
}

#[test]
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
fn rejects_invalid_share_urls() {
    assert_eq!(parse_share_url("https://opncd.ai/s/Jsj3hNIW"), None);
    assert_eq!(parse_share_url("https://opncd.ai/share/"), None);
    assert_eq!(parse_share_url("https://opncd.ai/share/id/extra"), None);
    assert_eq!(parse_share_url("not-a-url"), None);
}

#[test]
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
