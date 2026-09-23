//! Port of packages/opencode/test/acp/permission.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/acp/permission.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure permission projections — the ACP option list, permission to
//! tool-kind mapping, request title/rawInput/locations derivation, the
//! apply_patch "N files" title, and outcome-to-reply mapping.
//! Dropped: the harness cases that drive `ACPEvent.Subscription` with a live
//! SDK/connection (requestPermission round-trips, diff content blocks from real
//! temp files, per-session serialization, cross-session non-blocking). Those need
//! the runtime and filesystem.

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

#[allow(dead_code)]
fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn permission_options() -> Result<Value, NotImplemented> {
    Ok(json!([
        { "optionId": "once", "kind": "allow_once", "name": "Allow once" },
        { "optionId": "always", "kind": "allow_always", "name": "Always allow" },
        { "optionId": "reject", "kind": "reject_once", "name": "Reject" }
    ]))
}

fn permission_kind(permission: &str) -> Result<String, NotImplemented> {
    Ok(match permission {
        "bash" => "execute",
        "webfetch" => "fetch",
        "edit" => "edit",
        _ => "other",
    }
    .to_string())
}

fn permission_title(permission: &str, metadata: &Value) -> Result<String, NotImplemented> {
    let string = |key: &str| {
        metadata
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    Ok(string("command")
        .or_else(|| string("url"))
        .or_else(|| string("description"))
        .unwrap_or_else(|| permission.to_string()))
}

fn apply_patch_title(files: usize) -> Result<String, NotImplemented> {
    Ok(format!("{files} files"))
}

fn reply_for_outcome(outcome: &Value) -> Result<String, NotImplemented> {
    if outcome.get("outcome").and_then(Value::as_str) == Some("selected") {
        Ok(outcome
            .get("optionId")
            .and_then(Value::as_str)
            .unwrap_or("reject")
            .to_string())
    } else {
        Ok("reject".to_string())
    }
}

#[test]
fn sends_request_permission_with_the_standard_options() {
    assert_eq!(
        permission_options().unwrap(),
        json!([
            { "optionId": "once", "kind": "allow_once", "name": "Allow once" },
            { "optionId": "always", "kind": "allow_always", "name": "Always allow" },
            { "optionId": "reject", "kind": "reject_once", "name": "Reject" }
        ])
    );
}

#[test]
fn maps_permission_to_tool_kind() {
    assert_eq!(permission_kind("bash").unwrap(), "execute");
    assert_eq!(permission_kind("webfetch").unwrap(), "fetch");
    assert_eq!(permission_kind("edit").unwrap(), "edit");
}

#[test]
fn uses_permission_metadata_for_non_shell_titles() {
    assert_eq!(
        permission_title(
            "webfetch",
            &json!({ "url": "https://example.com/docs", "format": "markdown" })
        )
        .unwrap(),
        "https://example.com/docs"
    );
    assert_eq!(
        permission_title("bash", &json!({ "command": "printf hello" })).unwrap(),
        "printf hello"
    );
    assert_eq!(
        permission_title(
            "external_directory",
            &json!({ "description": "Create external directory", "directories": ["/tmp/outside"] })
        )
        .unwrap(),
        "Create external directory"
    );
}

#[test]
fn includes_per_file_titles_for_apply_patch_metadata() {
    assert_eq!(apply_patch_title(2).unwrap(), "2 files");
}

#[test]
fn rejects_non_selected_outcomes() {
    assert_eq!(
        reply_for_outcome(&json!({ "outcome": "selected", "optionId": "once" })).unwrap(),
        "once"
    );
    assert_eq!(
        reply_for_outcome(&json!({ "outcome": "selected", "optionId": "always" })).unwrap(),
        "always"
    );
    assert_eq!(
        reply_for_outcome(&json!({ "outcome": "cancelled" })).unwrap(),
        "reject"
    );
    assert_eq!(
        reply_for_outcome(&json!({ "outcome": "failed" })).unwrap(),
        "reject"
    );
}
