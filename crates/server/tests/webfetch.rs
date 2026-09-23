//! Port of packages/opencode/test/tool/webfetch.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: image responses become file attachments, SVG stays text,
//! text responses pass through, and HTML is reduced to visible text. The
//! reference spins a local Bun server; this port injects a fake `HttpClient`
//! with the same responses.

use opencode_server::tools::{
    HttpClient, HttpResponse, ToolContext, ToolError, WebFetchArgs, WebFetchTool,
};

struct FakeClient {
    content_type: String,
    body: Vec<u8>,
}

impl HttpClient for FakeClient {
    fn get(&self, _url: &str) -> Result<HttpResponse, ToolError> {
        Ok(HttpResponse {
            status: 200,
            content_type: self.content_type.clone(),
            body: self.body.clone(),
        })
    }
}

fn context() -> ToolContext {
    ToolContext {
        session_id: "ses_test".to_string(),
        message_id: "msg_message".to_string(),
        agent: "build".to_string(),
        ..ToolContext::default()
    }
}

fn fetch(
    content_type: &str,
    body: Vec<u8>,
    url: &str,
    format: &str,
) -> Result<opencode_server::tools::ToolResult, ToolError> {
    let client = FakeClient {
        content_type: content_type.to_string(),
        body,
    };
    WebFetchTool::new().execute_with(
        &client,
        WebFetchArgs {
            url: url.to_string(),
            format: format.to_string(),
        },
        &mut context(),
    )
}

#[test]
#[ignore = "porting: tool.webfetch not implemented"]
fn returns_image_responses_as_file_attachments() -> Result<(), ToolError> {
    let bytes = vec![137, 80, 78, 71, 13, 10, 26, 10];
    let result = fetch(
        "IMAGE/PNG; charset=binary",
        bytes,
        "http://localhost/image.png",
        "markdown",
    )?;

    assert_eq!(result.output, "Image fetched successfully");
    let attachments = result.attachments.expect("attachments");
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].kind, "file");
    assert_eq!(attachments[0].mime, "image/png");
    assert!(attachments[0].url.starts_with("data:image/png;base64,"));
    Ok(())
}

#[test]
#[ignore = "porting: tool.webfetch not implemented"]
fn keeps_svg_as_text_output() -> Result<(), ToolError> {
    let body = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><text>hello</text></svg>".to_vec();
    let result = fetch(
        "image/svg+xml; charset=UTF-8",
        body,
        "http://localhost/image.svg",
        "html",
    )?;

    assert!(result.output.contains("<svg"));
    assert!(result.attachments.is_none());
    Ok(())
}

#[test]
#[ignore = "porting: tool.webfetch not implemented"]
fn keeps_text_responses_as_text_output() -> Result<(), ToolError> {
    let result = fetch(
        "text/plain; charset=utf-8",
        b"hello from webfetch".to_vec(),
        "http://localhost/file.txt",
        "text",
    )?;

    assert_eq!(result.output, "hello from webfetch");
    assert!(result.attachments.is_none());
    Ok(())
}

#[test]
#[ignore = "porting: tool.webfetch not implemented"]
fn extracts_text_from_html_without_scripts_or_styles() -> Result<(), ToolError> {
    let body = b"<html><head><style>.hidden{}</style><script>alert('x')</script></head><body>Hello <b>world</b></body></html>".to_vec();
    let result = fetch(
        "text/html; charset=utf-8",
        body,
        "http://localhost/page.html",
        "text",
    )?;

    assert_eq!(result.output, "Hello world");
    assert!(result.attachments.is_none());
    Ok(())
}
