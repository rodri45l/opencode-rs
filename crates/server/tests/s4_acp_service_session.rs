//! Port of packages/opencode/test/acp/service-session.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/acp/service.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure session-list pagination (directory filter, updated-desc
//! ordering, 100-item page, cursor semantics), select-option flattening, and
//! new-session default-model resolution from config.
//! Dropped: the Effect/SDK-driven service cases — transcript replay, config
//! restoration from messages/durable state, MCP registration, prompt/command
//! routing, usage updates, error mapping. Those need the live SDK client,
//! ACP connection, and session runtime.

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn list_sessions(
    _sessions: &Value,
    _directory: Option<&str>,
    _cursor: Option<&str>,
) -> Result<Value, NotImplemented> {
    nope("acp service-session")
}

fn flatten_select_options(_option: &Value) -> Result<Vec<Value>, NotImplemented> {
    nope("acp service-session")
}

fn default_model(
    _config_model: Option<&str>,
    _providers_default: &Value,
) -> Result<Value, NotImplemented> {
    nope("acp service-session")
}

fn fixture_sessions() -> Value {
    Value::Array(
        (0..102)
            .map(|index| {
                json!({
                    "id": format!("ses_{}", index + 1),
                    "directory": if index % 2 == 0 { "/workspace" } else { "/other" },
                    "title": format!("Session {}", index + 1),
                    "time": { "created": index + 1, "updated": index + 1 }
                })
            })
            .collect(),
    )
}

#[test]
#[ignore = "porting: acp service-session not implemented"]
fn lists_sessions_sorted_by_updated_time_with_cursor_support() {
    let first = list_sessions(&fixture_sessions(), Some("/workspace"), None).unwrap();
    let second = list_sessions(
        &fixture_sessions(),
        Some("/workspace"),
        first["nextCursor"].as_str(),
    )
    .unwrap();
    let sessions = first["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 51);
    assert_eq!(sessions[0]["sessionId"], json!("ses_101"));
    assert_eq!(sessions[sessions.len() - 1]["sessionId"], json!("ses_1"));
    assert_eq!(first["nextCursor"], json!(null));
    assert_eq!(second["sessions"], first["sessions"]);
}

#[test]
#[ignore = "porting: acp service-session not implemented"]
fn lists_all_sessions_with_next_cursor_when_the_first_page_is_full() {
    let first = list_sessions(&fixture_sessions(), None, None).unwrap();
    let second = list_sessions(&fixture_sessions(), None, first["nextCursor"].as_str()).unwrap();
    let sessions = first["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 100);
    assert_eq!(sessions[0]["sessionId"], json!("ses_102"));
    assert_eq!(sessions[sessions.len() - 1]["sessionId"], json!("ses_3"));
    assert_eq!(first["nextCursor"], json!("3"));
    let ids: Vec<_> = second["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|session| session["sessionId"].clone())
        .collect();
    assert_eq!(ids, vec![json!("ses_2"), json!("ses_1")]);
}

#[test]
#[ignore = "porting: acp service-session not implemented"]
fn flattens_nested_select_options() {
    let option = json!({
        "options": [
            { "value": "low", "name": "Low" },
            { "group": "other", "options": [{ "value": "medium", "name": "Medium" }] }
        ]
    });
    assert_eq!(
        flatten_select_options(&option).unwrap(),
        vec![
            json!({ "value": "low", "name": "Low" }),
            json!({ "value": "medium", "name": "Medium" })
        ]
    );
}

#[test]
#[ignore = "porting: acp service-session not implemented"]
fn uses_the_configured_model_as_the_new_session_default() {
    assert_eq!(
        default_model(
            Some("test/configured-model"),
            &json!({ "test": "test-model" })
        )
        .unwrap(),
        json!({ "providerID": "test", "modelID": "configured-model" })
    );
}

#[test]
#[ignore = "porting: acp service-session not implemented"]
fn falls_back_to_the_provider_default_model() {
    assert_eq!(
        default_model(None, &json!({ "test": "test-model" })).unwrap(),
        json!({ "providerID": "test", "modelID": "test-model" })
    );
}
