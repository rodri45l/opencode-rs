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

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

fn store() -> &'static Mutex<HashMap<String, Session>> {
    static STORE: OnceLock<Mutex<HashMap<String, Session>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn create(
    id: &str,
    cwd: &str,
    mcp_servers: &Value,
    model: Option<SelectedModel>,
    variant: Option<&str>,
    mode_id: Option<&str>,
    created_at: Option<&str>,
) -> Result<Session, SessionError> {
    let session = Session {
        id: id.to_string(),
        cwd: cwd.to_string(),
        mcp_servers: mcp_servers.clone(),
        model,
        variant: variant.map(str::to_string),
        mode_id: mode_id.map(str::to_string),
        created_at: created_at.map(str::to_string),
        known_parts: Vec::new(),
    };
    store()
        .lock()
        .unwrap()
        .insert(id.to_string(), session.clone());
    Ok(session)
}

fn get(id: &str) -> Result<Session, SessionError> {
    store()
        .lock()
        .unwrap()
        .get(id)
        .cloned()
        .ok_or_else(|| SessionError::SessionNotFound {
            session_id: id.to_string(),
        })
}

fn try_get(id: &str) -> Result<Option<Session>, SessionError> {
    Ok(store().lock().unwrap().get(id).cloned())
}

fn set_model(id: &str, model: SelectedModel) -> Result<Session, SessionError> {
    let mut guard = store().lock().unwrap();
    let session = guard
        .get_mut(id)
        .ok_or_else(|| SessionError::SessionNotFound {
            session_id: id.to_string(),
        })?;
    session.model = Some(model);
    Ok(session.clone())
}

fn load(
    id: &str,
    cwd: &str,
    model: Option<SelectedModel>,
    variant: Option<&str>,
    mode_id: Option<&str>,
) -> Result<Session, SessionError> {
    create(id, cwd, &json!([]), model, variant, mode_id, None)
}

fn set_variant(id: &str, variant: &str) -> Result<(), SessionError> {
    let mut guard = store().lock().unwrap();
    let session = guard
        .get_mut(id)
        .ok_or_else(|| SessionError::SessionNotFound {
            session_id: id.to_string(),
        })?;
    session.variant = Some(variant.to_string());
    Ok(())
}

fn get_variant(id: &str) -> Result<Option<String>, SessionError> {
    Ok(get(id)?.variant)
}

fn set_mode(id: &str, mode: &str) -> Result<(), SessionError> {
    let mut guard = store().lock().unwrap();
    let session = guard
        .get_mut(id)
        .ok_or_else(|| SessionError::SessionNotFound {
            session_id: id.to_string(),
        })?;
    session.mode_id = Some(mode.to_string());
    Ok(())
}

fn get_mode(id: &str) -> Result<Option<String>, SessionError> {
    Ok(get(id)?.mode_id)
}

fn record_part_metadata(
    session_id: &str,
    message_id: &str,
    part_id: &str,
    tool_call_id: Option<&str>,
    metadata: Option<Value>,
) -> Result<PartMetadata, SessionError> {
    let mut guard = store().lock().unwrap();
    let session = guard
        .get_mut(session_id)
        .ok_or_else(|| SessionError::SessionNotFound {
            session_id: session_id.to_string(),
        })?;
    let entry = PartMetadata {
        message_id: message_id.to_string(),
        part_id: part_id.to_string(),
        tool_call_id: tool_call_id.map(str::to_string),
        metadata,
    };
    match session
        .known_parts
        .iter_mut()
        .find(|part| part.message_id == message_id && part.part_id == part_id)
    {
        Some(existing) => *existing = entry.clone(),
        None => session.known_parts.push(entry.clone()),
    }
    Ok(entry)
}

fn get_part_metadata(
    session_id: &str,
    message_id: &str,
    part_id: &str,
) -> Result<Option<PartMetadata>, SessionError> {
    let guard = store().lock().unwrap();
    Ok(guard.get(session_id).and_then(|session| {
        session
            .known_parts
            .iter()
            .find(|part| part.message_id == message_id && part.part_id == part_id)
            .cloned()
    }))
}

fn remove(id: &str) -> Result<Option<Session>, SessionError> {
    Ok(store().lock().unwrap().remove(id))
}

fn mcp_server() -> Value {
    json!({ "name": "local-tools", "command": "node", "args": ["server.js"], "env": [] })
}

#[test]
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
fn fails_required_lookups_with_typed_session_not_found() {
    let error = get("ses_missing").unwrap_err();
    match error {
        SessionError::SessionNotFound { session_id } => assert_eq!(session_id, "ses_missing"),
        other => panic!("expected SessionNotFound, got {other:?}"),
    }
}

#[test]
fn try_get_lets_event_routing_ignore_unknown_sessions() {
    assert_eq!(try_get("ses_missing").unwrap(), None);
    assert_eq!(
        get_part_metadata("ses_missing", "msg_1", "part_1").unwrap(),
        None
    );
}

#[test]
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
