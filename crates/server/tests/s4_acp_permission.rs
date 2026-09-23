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

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn permission_options() -> Result<Value, NotImplemented> {
    nope("acp permission")
}

fn permission_kind(_permission: &str) -> Result<String, NotImplemented> {
    nope("acp permission")
}

fn permission_title(_permission: &str, _metadata: &Value) -> Result<String, NotImplemented> {
    nope("acp permission")
}

fn apply_patch_title(_files: usize) -> Result<String, NotImplemented> {
    nope("acp permission")
}

fn reply_for_outcome(_outcome: &Value) -> Result<String, NotImplemented> {
    nope("acp permission")
}

#[test]
#[ignore = "porting: acp permission not implemented"]
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
#[ignore = "porting: acp permission not implemented"]
fn maps_permission_to_tool_kind() {
    assert_eq!(permission_kind("bash").unwrap(), "execute");
    assert_eq!(permission_kind("webfetch").unwrap(), "fetch");
    assert_eq!(permission_kind("edit").unwrap(), "edit");
}

#[test]
#[ignore = "porting: acp permission not implemented"]
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
#[ignore = "porting: acp permission not implemented"]
fn includes_per_file_titles_for_apply_patch_metadata() {
    assert_eq!(apply_patch_title(2).unwrap(), "2 files");
}

#[test]
#[ignore = "porting: acp permission not implemented"]
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
