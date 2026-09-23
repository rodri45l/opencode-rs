//! Port of packages/opencode/test/session/schema-decoding.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: each migrated session-domain schema accepts valid input
//! (round-tripping to the same value) and rejects clearly-invalid input.

use opencode_server::session::{decode, DecodeError};
use serde_json::{json, Value};

const SESSION_ID: &str = "ses_01J5Y5H0AH4Q4NXJ6P4C3P5V2K";
const SESSION_ID_CHILD: &str = "ses_01J5Y5H0AH4Q4NXJ6P4C3P5V2L";
const MESSAGE_ID: &str = "msg_01J5Y5H0AH4Q4NXJ6P4C3P5V2M";
const PART_ID: &str = "prt_01J5Y5H0AH4Q4NXJ6P4C3P5V2N";
const PROJECT_ID: &str = "proj-alpha";
const WORKSPACE_ID: &str = "wrk-primary";

fn assert_round_trip(schema: &str, input: Value) -> Result<(), DecodeError> {
    assert_eq!(decode(schema, &input)?, input);
    Ok(())
}

#[test]
fn session_info_accepts_minimal_session() -> Result<(), DecodeError> {
    assert_round_trip(
        "Session.Info",
        json!({
            "id": SESSION_ID,
            "slug": "hello",
            "projectID": PROJECT_ID,
            "directory": "/tmp/proj",
            "title": "First session",
            "version": "0.1.0",
            "time": { "created": 1, "updated": 2 }
        }),
    )
}

#[test]
fn session_info_round_trips_every_optional_field() -> Result<(), DecodeError> {
    assert_round_trip(
        "Session.Info",
        json!({
            "id": SESSION_ID,
            "slug": "fullshape",
            "projectID": PROJECT_ID,
            "workspaceID": WORKSPACE_ID,
            "directory": "/tmp/proj",
            "path": "packages/opencode",
            "parentID": SESSION_ID_CHILD,
            "summary": {
                "additions": 10,
                "deletions": 5,
                "files": 2,
                "diffs": [{ "additions": 1, "deletions": 0, "file": "a.ts", "patch": "--- a/a.ts" }]
            },
            "share": { "url": "https://share.example.com/s/1" },
            "title": "Full session",
            "version": "1.0.0",
            "metadata": { "source": "test" },
            "time": { "created": 100, "updated": 200, "compacting": 150, "archived": 300 },
            "permission": [{ "action": "allow", "pattern": "*", "permission": "read" }],
            "revert": {
                "messageID": MESSAGE_ID,
                "partID": PART_ID,
                "snapshot": "snap-1",
                "diff": "diff-1"
            }
        }),
    )
}

#[test]
fn session_info_accepts_migrated_summary_diffs_without_file_details() -> Result<(), DecodeError> {
    assert_round_trip(
        "Session.Info",
        json!({
            "id": SESSION_ID,
            "slug": "legacy-diff",
            "projectID": PROJECT_ID,
            "directory": "/tmp/proj",
            "title": "Legacy diff",
            "version": "0.1.0",
            "summary": {
                "additions": 1,
                "deletions": 0,
                "files": 1,
                "diffs": [{ "additions": 1, "deletions": 0 }]
            },
            "time": { "created": 1, "updated": 2 }
        }),
    )
}

#[test]
fn session_info_rejects_unbranded_session_id() {
    assert!(decode("Session.Info", &json!({ "id": "not-a-session-id" })).is_err());
}

#[test]
fn session_info_rejects_missing_required_fields() {
    assert!(decode("Session.Info", &json!({ "id": SESSION_ID })).is_err());
}

#[test]
fn project_info_accepts_with_and_without_optional_name() -> Result<(), DecodeError> {
    assert_round_trip(
        "Session.ProjectInfo",
        json!({ "id": PROJECT_ID, "worktree": "/tmp/wt" }),
    )?;
    assert_round_trip(
        "Session.ProjectInfo",
        json!({ "id": PROJECT_ID, "worktree": "/tmp/wt", "name": "alpha" }),
    )
}

#[test]
fn global_info_accepts_null_project() -> Result<(), DecodeError> {
    assert_round_trip(
        "Session.GlobalInfo",
        json!({
            "id": SESSION_ID,
            "slug": "global",
            "projectID": PROJECT_ID,
            "directory": "/tmp/proj",
            "title": "global",
            "version": "0",
            "time": { "created": 0, "updated": 0 },
            "project": null
        }),
    )
}

#[test]
fn global_info_accepts_populated_project() -> Result<(), DecodeError> {
    assert_round_trip(
        "Session.GlobalInfo",
        json!({
            "id": SESSION_ID,
            "slug": "global",
            "projectID": PROJECT_ID,
            "directory": "/tmp/proj",
            "title": "global",
            "version": "0",
            "time": { "created": 0, "updated": 0 },
            "project": { "id": PROJECT_ID, "worktree": "/tmp/wt", "name": "alpha" }
        }),
    )
}

#[test]
fn create_input_accepts_undefined_and_populated_forms() -> Result<(), DecodeError> {
    assert_eq!(decode("Session.CreateInput", &Value::Null)?, Value::Null);
    assert_round_trip(
        "Session.CreateInput",
        json!({
            "parentID": SESSION_ID,
            "title": "child",
            "metadata": { "source": "test" },
            "permission": [{ "action": "ask", "pattern": "*", "permission": "bash" }],
            "workspaceID": WORKSPACE_ID
        }),
    )
}

#[test]
fn fork_input_round_trips() -> Result<(), DecodeError> {
    assert_round_trip(
        "Session.ForkInput",
        json!({ "sessionID": SESSION_ID, "messageID": MESSAGE_ID }),
    )?;
    assert_round_trip("Session.ForkInput", json!({ "sessionID": SESSION_ID }))
}

#[test]
fn set_title_input_rejects_missing_title() {
    assert!(decode("Session.SetTitleInput", &json!({ "sessionID": SESSION_ID })).is_err());
}

#[test]
fn set_archived_input_accepts_both_with_and_without_time() -> Result<(), DecodeError> {
    assert_round_trip(
        "Session.SetArchivedInput",
        json!({ "sessionID": SESSION_ID }),
    )?;
    assert_round_trip(
        "Session.SetArchivedInput",
        json!({ "sessionID": SESSION_ID, "time": 123 }),
    )
}

#[test]
fn set_permission_input_requires_a_ruleset() -> Result<(), DecodeError> {
    assert_round_trip(
        "Session.SetPermissionInput",
        json!({
            "sessionID": SESSION_ID,
            "permission": [{ "action": "deny", "pattern": "*", "permission": "write" }]
        }),
    )?;
    assert!(decode(
        "Session.SetPermissionInput",
        &json!({ "sessionID": SESSION_ID })
    )
    .is_err());
    Ok(())
}

#[test]
fn messages_input_accepts_optional_limit() -> Result<(), DecodeError> {
    assert_round_trip("Session.MessagesInput", json!({ "sessionID": SESSION_ID }))?;
    assert_round_trip(
        "Session.MessagesInput",
        json!({ "sessionID": SESSION_ID, "limit": 50 }),
    )
}

#[test]
fn revert_input_requires_message_id_and_optional_part_id() -> Result<(), DecodeError> {
    assert_round_trip(
        "SessionRevert.RevertInput",
        json!({ "sessionID": SESSION_ID, "messageID": MESSAGE_ID, "partID": PART_ID }),
    )?;
    assert_round_trip(
        "SessionRevert.RevertInput",
        json!({ "sessionID": SESSION_ID, "messageID": MESSAGE_ID }),
    )?;
    assert!(decode(
        "SessionRevert.RevertInput",
        &json!({ "sessionID": SESSION_ID })
    )
    .is_err());
    Ok(())
}

#[test]
fn diff_input_accepts_optional_message_id() -> Result<(), DecodeError> {
    assert_round_trip(
        "SessionSummary.DiffInput",
        json!({ "sessionID": SESSION_ID }),
    )?;
    assert_round_trip(
        "SessionSummary.DiffInput",
        json!({ "sessionID": SESSION_ID, "messageID": MESSAGE_ID }),
    )
}

#[test]
fn status_info_accepts_idle_and_busy() -> Result<(), DecodeError> {
    assert_round_trip("SessionStatus.Info", json!({ "type": "idle" }))?;
    assert_round_trip("SessionStatus.Info", json!({ "type": "busy" }))
}

#[test]
fn status_info_retry_carries_attempt_message_action_and_next() -> Result<(), DecodeError> {
    assert_round_trip(
        "SessionStatus.Info",
        json!({
            "type": "retry",
            "attempt": 1,
            "message": "transient",
            "action": {
                "reason": "free_tier_limit",
                "provider": "opencode",
                "title": "Free limit reached",
                "message": "Subscribe to OpenCode Go.",
                "label": "subscribe",
                "link": "https://opencode.ai/go"
            },
            "next": 500
        }),
    )
}

#[test]
fn status_info_rejects_unknown_type() {
    assert!(decode("SessionStatus.Info", &json!({ "type": "bogus" })).is_err());
}

#[test]
fn todo_info_round_trips_three_fields() -> Result<(), DecodeError> {
    assert_round_trip(
        "Todo.Info",
        json!({ "content": "do a thing", "status": "pending", "priority": "high" }),
    )
}

#[test]
fn loop_input_is_just_session_id() -> Result<(), DecodeError> {
    assert_round_trip(
        "SessionPrompt.LoopInput",
        json!({ "sessionID": SESSION_ID }),
    )
}

#[test]
fn shell_input_requires_agent_and_command() -> Result<(), DecodeError> {
    assert_round_trip(
        "SessionPrompt.ShellInput",
        json!({ "sessionID": SESSION_ID, "agent": "build", "command": "echo hi" }),
    )?;
    assert!(decode(
        "SessionPrompt.ShellInput",
        &json!({ "sessionID": SESSION_ID })
    )
    .is_err());
    Ok(())
}

#[test]
fn prompt_input_accepts_text_and_file_parts() -> Result<(), DecodeError> {
    let input = json!({
        "sessionID": SESSION_ID,
        "parts": [
            { "type": "text", "text": "hello" },
            { "type": "file", "mime": "image/png", "url": "data:image/png;base64,AAAA" }
        ]
    });
    let decoded = decode("SessionPrompt.PromptInput", &input)?;
    let parts = decoded["parts"].as_array().expect("parts");
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0]["type"], "text");
    assert_eq!(parts[1]["mime"], "image/png");
    Ok(())
}

#[test]
fn prompt_input_rejects_unknown_part_type() {
    assert!(decode(
        "SessionPrompt.PromptInput",
        &json!({
            "sessionID": SESSION_ID,
            "parts": [{ "type": "nonsense", "payload": 42 }]
        })
    )
    .is_err());
}

#[test]
fn command_input_round_trips_core_fields() -> Result<(), DecodeError> {
    assert_round_trip(
        "SessionPrompt.CommandInput",
        json!({ "sessionID": SESSION_ID, "arguments": "--flag", "command": "deploy" }),
    )
}
