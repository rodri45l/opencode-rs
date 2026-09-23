//! Port of packages/opencode/test/acp/session.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/acp/session.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure in-memory ACP session state machine (create/get/tryGet,
//! model/variant/mode updates, part-metadata routing, remove).
//! Dropped: none — the reference drives this service through an Effect Layer,
//! but the observable behaviour is pure map state.

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum SessionError {
    SessionNotFound { session_id: String },
    NotImplemented(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
struct SelectedModel {
    provider_id: String,
    model_id: String,
}

#[derive(Debug, Clone, PartialEq)]
struct PartMetadata {
    message_id: String,
    part_id: String,
    tool_call_id: Option<String>,
    metadata: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
struct Session {
    id: String,
    cwd: String,
    mcp_servers: Value,
    model: Option<SelectedModel>,
    variant: Option<String>,
    mode_id: Option<String>,
    created_at: Option<String>,
    known_parts: Vec<PartMetadata>,
}

fn model(provider_id: &str, model_id: &str) -> SelectedModel {
    SelectedModel {
        provider_id: provider_id.to_string(),
        model_id: model_id.to_string(),
    }
}

fn create(
    _id: &str,
    _cwd: &str,
    _mcp_servers: &Value,
    _model: Option<SelectedModel>,
    _variant: Option<&str>,
    _mode_id: Option<&str>,
    _created_at: Option<&str>,
) -> Result<Session, SessionError> {
    Err(SessionError::NotImplemented("acp session"))
}

fn get(_id: &str) -> Result<Session, SessionError> {
    Err(SessionError::NotImplemented("acp session"))
}

fn try_get(_id: &str) -> Result<Option<Session>, SessionError> {
    Err(SessionError::NotImplemented("acp session"))
}

fn set_model(_id: &str, _model: SelectedModel) -> Result<Session, SessionError> {
    Err(SessionError::NotImplemented("acp session"))
}

fn load(
    _id: &str,
    _cwd: &str,
    _model: Option<SelectedModel>,
    _variant: Option<&str>,
    _mode_id: Option<&str>,
) -> Result<Session, SessionError> {
    Err(SessionError::NotImplemented("acp session"))
}

fn set_variant(_id: &str, _variant: &str) -> Result<(), SessionError> {
    Err(SessionError::NotImplemented("acp session"))
}

fn get_variant(_id: &str) -> Result<Option<String>, SessionError> {
    Err(SessionError::NotImplemented("acp session"))
}

fn set_mode(_id: &str, _mode: &str) -> Result<(), SessionError> {
    Err(SessionError::NotImplemented("acp session"))
}

fn get_mode(_id: &str) -> Result<Option<String>, SessionError> {
    Err(SessionError::NotImplemented("acp session"))
}

fn record_part_metadata(
    _session_id: &str,
    _message_id: &str,
    _part_id: &str,
    _tool_call_id: Option<&str>,
    _metadata: Option<Value>,
) -> Result<PartMetadata, SessionError> {
    Err(SessionError::NotImplemented("acp session"))
}

fn get_part_metadata(
    _session_id: &str,
    _message_id: &str,
    _part_id: &str,
) -> Result<Option<PartMetadata>, SessionError> {
    Err(SessionError::NotImplemented("acp session"))
}

fn remove(_id: &str) -> Result<Option<Session>, SessionError> {
    Err(SessionError::NotImplemented("acp session"))
}

fn mcp_server() -> Value {
    json!({ "name": "local-tools", "command": "node", "args": ["server.js"], "env": [] })
}

#[test]
#[ignore = "porting: acp session not implemented"]
fn creates_and_retrieves_session_state() {
    let created = create(
        "ses_1",
        "/workspace",
        &json!([mcp_server()]),
        Some(model("anthropic", "claude-sonnet")),
        Some("high"),
        Some("build"),
        Some("2026-05-25T00:00:00.000Z"),
    )
    .unwrap();
    assert_eq!(created.id, "ses_1");
    assert_eq!(created.cwd, "/workspace");
    assert_eq!(created.mcp_servers, json!([mcp_server()]));
    assert_eq!(created.model, Some(model("anthropic", "claude-sonnet")));
    assert_eq!(created.variant.as_deref(), Some("high"));
    assert_eq!(created.mode_id.as_deref(), Some("build"));

    let loaded = get("ses_1").unwrap();
    assert_eq!(loaded.created_at, created.created_at);
    assert!(loaded.known_parts.is_empty());
}

#[test]
#[ignore = "porting: acp session not implemented"]
fn fails_required_lookups_with_typed_session_not_found() {
    let error = get("ses_missing").unwrap_err();
    match error {
        SessionError::SessionNotFound { session_id } => assert_eq!(session_id, "ses_missing"),
        other => panic!("expected SessionNotFound, got {other:?}"),
    }
}

#[test]
#[ignore = "porting: acp session not implemented"]
fn try_get_lets_event_routing_ignore_unknown_sessions() {
    assert_eq!(try_get("ses_missing").unwrap(), None);
    assert_eq!(
        get_part_metadata("ses_missing", "msg_1", "part_1").unwrap(),
        None
    );
}

#[test]
#[ignore = "porting: acp session not implemented"]
fn updates_selected_model_while_preserving_session_identity_and_inputs() {
    create(
        "ses_model",
        "/workspace",
        &json!([mcp_server()]),
        Some(model("anthropic", "claude-sonnet")),
        Some("high"),
        Some("build"),
        None,
    )
    .unwrap();
    let updated = set_model("ses_model", model("openai", "gpt-5")).unwrap();
    assert_eq!(updated.id, "ses_model");
    assert_eq!(updated.cwd, "/workspace");
    assert_eq!(updated.mcp_servers, json!([mcp_server()]));
    assert_eq!(updated.model, Some(model("openai", "gpt-5")));
    assert_eq!(updated.variant.as_deref(), Some("high"));
    assert_eq!(updated.mode_id.as_deref(), Some("build"));
}

#[test]
#[ignore = "porting: acp session not implemented"]
fn updates_selected_variant_and_mode_independently() {
    load(
        "ses_config",
        "/workspace",
        Some(model("anthropic", "claude-sonnet")),
        Some("low"),
        Some("plan"),
    )
    .unwrap();
    set_variant("ses_config", "high").unwrap();
    assert_eq!(get_variant("ses_config").unwrap().as_deref(), Some("high"));
    assert_eq!(get_mode("ses_config").unwrap().as_deref(), Some("plan"));

    set_mode("ses_config", "build").unwrap();
    assert_eq!(get_variant("ses_config").unwrap().as_deref(), Some("high"));
    assert_eq!(get_mode("ses_config").unwrap().as_deref(), Some("build"));
}

#[test]
#[ignore = "porting: acp session not implemented"]
fn records_known_message_part_metadata_for_delta_routing() {
    create(
        "ses_parts",
        "/workspace",
        &json!([]),
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let metadata = record_part_metadata(
        "ses_parts",
        "msg_1",
        "part_1",
        Some("tool_1"),
        Some(json!({ "output": "first chunk" })),
    )
    .unwrap();
    assert_eq!(
        metadata,
        PartMetadata {
            message_id: "msg_1".into(),
            part_id: "part_1".into(),
            tool_call_id: Some("tool_1".into()),
            metadata: Some(json!({ "output": "first chunk" }))
        }
    );
    assert_eq!(
        get_part_metadata("ses_parts", "msg_1", "part_1").unwrap(),
        Some(metadata)
    );
}

#[test]
#[ignore = "porting: acp session not implemented"]
fn keeps_repeated_part_ids_distinct_across_messages() {
    create(
        "ses_duplicate_parts",
        "/workspace",
        &json!([]),
        None,
        None,
        None,
        None,
    )
    .unwrap();
    record_part_metadata(
        "ses_duplicate_parts",
        "msg_1",
        "part_1",
        None,
        Some(json!({ "output": "from first message" })),
    )
    .unwrap();
    record_part_metadata(
        "ses_duplicate_parts",
        "msg_2",
        "part_1",
        None,
        Some(json!({ "output": "from second message" })),
    )
    .unwrap();
    let first = get_part_metadata("ses_duplicate_parts", "msg_1", "part_1")
        .unwrap()
        .unwrap();
    let second = get_part_metadata("ses_duplicate_parts", "msg_2", "part_1")
        .unwrap()
        .unwrap();
    assert_eq!(
        first.metadata,
        Some(json!({ "output": "from first message" }))
    );
    assert_eq!(
        second.metadata,
        Some(json!({ "output": "from second message" }))
    );
}

#[test]
#[ignore = "porting: acp session not implemented"]
fn removing_a_session_clears_its_known_part_metadata() {
    create(
        "ses_remove",
        "/workspace",
        &json!([]),
        None,
        None,
        None,
        None,
    )
    .unwrap();
    record_part_metadata("ses_remove", "msg_1", "part_1", None, None).unwrap();
    let removed = remove("ses_remove").unwrap().unwrap();
    assert_eq!(removed.known_parts.len(), 1);
    assert_eq!(try_get("ses_remove").unwrap(), None);
    assert_eq!(
        get_part_metadata("ses_remove", "msg_1", "part_1").unwrap(),
        None
    );
}
