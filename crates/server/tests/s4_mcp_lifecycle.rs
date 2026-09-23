//! Port of packages/opencode/test/mcp/lifecycle.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/mcp/index.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure naming and pagination rules — server/tool name sanitization
//! and key prefixing, cursor following that accepts an empty cursor and rejects a
//! repeated one, and the "non-empty trimmed instructions only" filter.
//! Dropped: every case that opens a live stdio/HTTP MCP session (roots
//! advertisement, tool-list-change cache refresh, disconnect/reconnect, timeouts,
//! process termination, OAuth callback cancellation). Those need spawned servers
//! and the Effect runtime.

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn sanitize_server_name(_name: &str) -> Result<String, NotImplemented> {
    nope("mcp lifecycle")
}

fn sanitize_tool_name(_name: &str) -> Result<String, NotImplemented> {
    nope("mcp lifecycle")
}

fn tool_key(_server: &str, _tool: &str) -> Result<String, NotImplemented> {
    nope("mcp lifecycle")
}

fn content_key(_server: &str, _name: &str) -> Result<String, NotImplemented> {
    nope("mcp lifecycle")
}

fn follow_cursors(_pages: &Value) -> Result<Vec<Value>, NotImplemented> {
    nope("mcp lifecycle")
}

fn includes_instructions(_raw: &str) -> Result<bool, NotImplemented> {
    nope("mcp lifecycle")
}

#[test]
#[ignore = "porting: mcp lifecycle not implemented"]
fn tools_prefix_sanitized_server_and_tool_names() {
    assert_eq!(
        sanitize_server_name("my.special-server").unwrap(),
        "my_special-server"
    );
    assert_eq!(sanitize_tool_name("tool.b").unwrap(), "tool_b");
    assert_eq!(
        tool_key("my.special-server", "tool-a").unwrap(),
        "my_special-server_tool-a"
    );
    assert_eq!(
        tool_key("my.special-server", "tool.b").unwrap(),
        "my_special-server_tool_b"
    );
}

#[test]
#[ignore = "porting: mcp lifecycle not implemented"]
fn prompts_and_resources_are_keyed_by_server_and_uri() {
    assert_eq!(
        content_key("paged-server", "prompt-one").unwrap(),
        "paged-server:prompt-one"
    );
    assert_eq!(
        content_key("content-server", "file:///test.txt").unwrap(),
        "content-server:file:///test.txt"
    );
}

#[test]
#[ignore = "porting: mcp lifecycle not implemented"]
fn accepts_empty_cursors_and_rejects_repeated_cursors() {
    let empty = json!({
        "initial": { "items": [{ "name": "prompt-one" }], "nextCursor": "" },
        "": { "items": [{ "name": "prompt-two" }] }
    });
    assert_eq!(
        follow_cursors(&empty).unwrap(),
        vec![
            json!({ "name": "prompt-one" }),
            json!({ "name": "prompt-two" })
        ]
    );

    let looping = json!({
        "initial": { "items": [], "nextCursor": "repeat" },
        "repeat": { "items": [], "nextCursor": "repeat" }
    });
    assert!(follow_cursors(&looping).is_err());
}

#[test]
#[ignore = "porting: mcp lifecycle not implemented"]
fn follows_cursors_across_multiple_pages() {
    let pages = json!({
        "initial": { "items": [{ "name": "tool-one" }], "nextCursor": "tools-2" },
        "tools-2": { "items": [{ "name": "tool-two" }] }
    });
    assert_eq!(
        follow_cursors(&pages).unwrap(),
        vec![json!({ "name": "tool-one" }), json!({ "name": "tool-two" })]
    );
}

#[test]
#[ignore = "porting: mcp lifecycle not implemented"]
fn instructions_require_non_empty_trimmed_content() {
    assert!(includes_instructions("Use lookup before mutate.").unwrap());
    assert!(!includes_instructions("   ").unwrap());
    assert!(!includes_instructions("").unwrap());
}
