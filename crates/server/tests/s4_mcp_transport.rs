//! Port of packages/opencode/test/mcp/transport.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the MCP streamable-HTTP transport wrapper; see
//! docs/TEST-PORT.md.
//!
//! Ported: the pure reconnect decision — a JSON-RPC error response terminates the
//! stream and must not trigger a reconnect (so exactly one request is made).
//! Dropped: the runtime harness (fake fetch + ReadableStream + reconnection
//! timers) that observes the request count over time.

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn should_reconnect(_message: &Value) -> Result<bool, NotImplemented> {
    nope("mcp transport")
}

#[test]
#[ignore = "porting: mcp transport not implemented"]
fn does_not_reconnect_after_a_jsonrpc_error_response() {
    let error = json!({
        "jsonrpc": "2.0",
        "error": { "code": -32601, "message": "Method not found" },
        "id": 1
    });
    assert!(!should_reconnect(&error).unwrap());
}

#[test]
#[ignore = "porting: mcp transport not implemented"]
fn keeps_the_stream_alive_for_non_error_messages() {
    assert!(should_reconnect(&json!({ "jsonrpc": "2.0", "id": "prime", "result": {} })).unwrap());
}
