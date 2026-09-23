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

#[allow(dead_code)]
fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

const PAGE_SIZE: usize = 100;

fn session_number(id: &str) -> i64 {
    id.strip_prefix("ses_")
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

fn list_sessions(
    sessions: &Value,
    directory: Option<&str>,
    cursor: Option<&str>,
) -> Result<Value, NotImplemented> {
    let cursor_number = cursor.and_then(|value| value.parse::<i64>().ok());
    let mut filtered: Vec<Value> = sessions
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|session| {
            directory
                .map(|directory| session["directory"] == json!(directory))
                .unwrap_or(true)
        })
        .filter(|session| {
            cursor_number
                .map(|cursor| session_number(session["id"].as_str().unwrap_or("")) < cursor)
                .unwrap_or(true)
        })
        .collect();
    filtered.sort_by(|a, b| {
        let left = a["time"]["updated"].as_i64().unwrap_or(0);
        let right = b["time"]["updated"].as_i64().unwrap_or(0);
        right.cmp(&left).then_with(|| {
            session_number(b["id"].as_str().unwrap_or(""))
                .cmp(&session_number(a["id"].as_str().unwrap_or("")))
        })
    });

    let has_more = filtered.len() > PAGE_SIZE;
    let page: Vec<Value> = filtered
        .into_iter()
        .take(PAGE_SIZE)
        .map(|session| {
            json!({
                "sessionId": session["id"],
                "directory": session["directory"],
                "title": session["title"],
                "time": session["time"]
            })
        })
        .collect();
    let next_cursor = if has_more {
        page.last()
            .and_then(|session| session["sessionId"].as_str())
            .map(|id| json!(session_number(id).to_string()))
            .unwrap_or(Value::Null)
    } else {
        Value::Null
    };
    Ok(json!({ "sessions": page, "nextCursor": next_cursor }))
}

fn flatten_select_options(option: &Value) -> Result<Vec<Value>, NotImplemented> {
    let mut flat = Vec::new();
    if let Some(options) = option.get("options").and_then(Value::as_array) {
        for entry in options {
            if entry.get("options").is_some() {
                flat.extend(flatten_select_options(entry)?);
            } else {
                flat.push(json!({ "value": entry["value"], "name": entry["name"] }));
            }
        }
    }
    Ok(flat)
}

fn default_model(
    config_model: Option<&str>,
    providers_default: &Value,
) -> Result<Value, NotImplemented> {
    if let Some(config_model) = config_model {
        let mut parts = config_model.splitn(2, '/');
        let provider = parts.next().unwrap_or("");
        let model = parts.next().unwrap_or("");
        return Ok(json!({ "providerID": provider, "modelID": model }));
    }
    if let Some((provider, model)) = providers_default
        .as_object()
        .and_then(|map| map.iter().next())
    {
        return Ok(json!({ "providerID": provider, "modelID": model }));
    }
    Ok(Value::Null)
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
fn falls_back_to_the_provider_default_model() {
    assert_eq!(
        default_model(None, &json!({ "test": "test-model" })).unwrap(),
        json!({ "providerID": "test", "modelID": "test-model" })
    );
}
