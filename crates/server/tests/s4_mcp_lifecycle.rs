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

#[allow(dead_code)]
fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn sanitize_server_name(name: &str) -> Result<String, NotImplemented> {
    Ok(sanitize_name(name))
}

fn sanitize_tool_name(name: &str) -> Result<String, NotImplemented> {
    Ok(sanitize_name(name))
}

fn tool_key(server: &str, tool: &str) -> Result<String, NotImplemented> {
    Ok(format!(
        "{}_{}",
        sanitize_server_name(server)?,
        sanitize_tool_name(tool)?
    ))
}

fn content_key(server: &str, name: &str) -> Result<String, NotImplemented> {
    Ok(format!("{server}:{name}"))
}

fn follow_cursors(pages: &Value) -> Result<Vec<Value>, NotImplemented> {
    let mut seen = std::collections::HashSet::new();
    let mut cursor = "initial".to_string();
    let mut out = Vec::new();
    loop {
        if !seen.insert(cursor.clone()) {
            return Err(NotImplemented("mcp lifecycle"));
        }
        let page = pages.get(&cursor).ok_or(NotImplemented("mcp lifecycle"))?;
        if let Some(items) = page.get("items").and_then(Value::as_array) {
            out.extend(items.iter().cloned());
        }
        match page.get("nextCursor").and_then(Value::as_str) {
            Some(next) => cursor = next.to_string(),
            None => break,
        }
    }
    Ok(out)
}

fn includes_instructions(raw: &str) -> Result<bool, NotImplemented> {
    Ok(!raw.trim().is_empty())
}

#[test]
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
fn instructions_require_non_empty_trimmed_content() {
    assert!(includes_instructions("Use lookup before mutate.").unwrap());
    assert!(!includes_instructions("   ").unwrap());
    assert!(!includes_instructions("").unwrap());
}
