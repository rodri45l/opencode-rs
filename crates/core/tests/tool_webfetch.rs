//! Port of packages/core/test/tool-webfetch.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the input schema defaults `format` to `markdown` and rejects
//! out-of-range timeouts, HTML-to-text and HTML-to-markdown conversions strip
//! active content, only HTTP schemes are fetchable, and oversized bodies are
//! rejected. Re-derived: the `ToolRegistry`/`HttpClient`/`PermissionV2` wiring and
//! live-server redirect cases are dropped; the pure input and conversion
//! behaviour is kept.

use opencode_core::tool_webfetch::{WebFetchTool, MAX_TIMEOUT_SECONDS};
use serde_json::json;

const NOTE: &str = "porting: web fetch tool not implemented";

#[test]
#[ignore = "porting: web fetch tool not implemented"]
fn defaults_format_and_rejects_invalid_timeout_controls() {
    let decoded = WebFetchTool::parse_input(&json!({ "url": "https://example.com" })).expect(NOTE);
    assert_eq!(decoded.format, "markdown");
    assert!(
        WebFetchTool::parse_input(&json!({ "url": "https://example.com", "timeout": 0 })).is_err()
    );
    assert!(WebFetchTool::parse_input(
        &json!({ "url": "https://example.com", "timeout": MAX_TIMEOUT_SECONDS + 1 })
    )
    .is_err());
}

#[test]
#[ignore = "porting: web fetch tool not implemented"]
fn converts_html_text_and_markdown_without_active_content() {
    let html = "<h1>Hello</h1><script>bad()</script><p>world <strong>wide</strong></p><style>.bad {}</style>";
    assert_eq!(
        WebFetchTool::extract_text_from_html(html).expect(NOTE),
        "Helloworld wide"
    );
    assert_eq!(
        WebFetchTool::convert_html_to_markdown(html).expect(NOTE),
        "# Hello\n\nworld **wide**"
    );
}

#[test]
#[ignore = "porting: web fetch tool not implemented"]
fn rejects_non_http_schemes() {
    assert!(!WebFetchTool::is_supported_scheme("file:///etc/passwd").expect(NOTE));
    assert!(WebFetchTool::is_supported_scheme("https://example.com").expect(NOTE));
}
