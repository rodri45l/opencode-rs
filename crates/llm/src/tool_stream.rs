//! Streamed tool-call assembly shared by protocol adapters.

use serde_json::{json, Map, Value};

use crate::error::{LlmError, LlmResult};

/// Incremental tool-call state machine.
pub struct ToolStream;

fn key_of(item: &Value) -> String {
    match item {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn empty_state() -> Value {
    json!({ "tools": {}, "events": [] })
}

fn parse_input(raw: &str) -> LlmResult<Value> {
    let text = if raw.is_empty() { "{}" } else { raw };
    serde_json::from_str(text)
        .map_err(|_| LlmError::InvalidRequest(format!("Invalid JSON tool input: {text}")))
}

impl ToolStream {
    /// Empty tool state.
    pub fn empty() -> Value {
        empty_state()
    }

    /// Start tracking a tool call.
    pub fn start(state: Value, item: Value, tool: Value) -> Value {
        let mut state = state;
        let key = key_of(&item);
        let mut entry = Map::new();
        entry.insert("id".into(), tool.get("id").cloned().unwrap_or(Value::Null));
        entry.insert(
            "name".into(),
            tool.get("name").cloned().unwrap_or(Value::Null),
        );
        entry.insert(
            "input".into(),
            tool.get("input")
                .cloned()
                .unwrap_or_else(|| Value::String(String::new())),
        );
        if let Some(provider_executed) = tool.get("providerExecuted") {
            entry.insert("providerExecuted".into(), provider_executed.clone());
        }
        state["tools"][&key] = Value::Object(entry);
        state
    }

    /// Append a delta or start a new tool call.
    pub fn append_or_start(
        state: Value,
        index: usize,
        delta: Value,
        missing: &str,
    ) -> LlmResult<Value> {
        let mut state = state;
        let key = index.to_string();
        let existing = state["tools"].get(&key).cloned();
        let text = delta
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if let Some(mut entry) = existing {
            let id = entry.get("id").cloned().unwrap_or(Value::Null);
            let name = entry.get("name").cloned().unwrap_or(Value::Null);
            let prev = entry
                .get("input")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            entry["input"] = Value::String(format!("{prev}{text}"));
            state["tools"][&key] = entry;
            state["events"] = json!([{
                "type": "tool-input-delta",
                "id": id,
                "name": name,
                "text": text,
            }]);
            return Ok(state);
        }

        let id = delta
            .get("id")
            .filter(|value| !value.is_null())
            .ok_or_else(|| LlmError::InvalidRequest(missing.to_string()))?;
        let name = delta
            .get("name")
            .filter(|value| !value.is_null())
            .ok_or_else(|| LlmError::InvalidRequest(missing.to_string()))?;
        let mut entry = Map::new();
        entry.insert("id".into(), id.clone());
        entry.insert("name".into(), name.clone());
        entry.insert("input".into(), Value::String(text.clone()));
        if let Some(provider_executed) = delta.get("providerExecuted") {
            entry.insert("providerExecuted".into(), provider_executed.clone());
        }
        state["tools"][&key] = Value::Object(entry);
        state["events"] = json!([
            { "type": "tool-input-start", "id": id, "name": name },
            { "type": "tool-input-delta", "id": id, "name": name, "text": text },
        ]);
        Ok(state)
    }

    /// Append a delta to an existing tool call.
    pub fn append_existing(
        state: Value,
        index: usize,
        text: &str,
        missing: &str,
    ) -> LlmResult<Value> {
        let mut state = state;
        let key = index.to_string();
        let Some(mut entry) = state["tools"].get(&key).cloned() else {
            return Err(LlmError::InvalidRequest(missing.to_string()));
        };
        let id = entry.get("id").cloned().unwrap_or(Value::Null);
        let name = entry.get("name").cloned().unwrap_or(Value::Null);
        let prev = entry
            .get("input")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        entry["input"] = Value::String(format!("{prev}{text}"));
        state["tools"][&key] = entry;
        state["events"] = json!([{
            "type": "tool-input-delta",
            "id": id,
            "name": name,
            "text": text,
        }]);
        Ok(state)
    }

    fn finish_key(
        state: &mut Value,
        key: &str,
        input_override: Option<&str>,
    ) -> LlmResult<Vec<Value>> {
        let Some(entry) = state["tools"].get(key).cloned() else {
            return Err(LlmError::InvalidRequest("unknown tool stream entry".into()));
        };
        let id = entry.get("id").cloned().unwrap_or(Value::Null);
        let name = entry.get("name").cloned().unwrap_or(Value::Null);
        let raw = input_override.map(str::to_string).unwrap_or_else(|| {
            entry
                .get("input")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        });
        let input = parse_input(&raw)?;
        if let Some(tools) = state.get_mut("tools").and_then(Value::as_object_mut) {
            tools.remove(key);
        }
        let mut call = Map::new();
        call.insert("type".into(), Value::String("tool-call".into()));
        call.insert("id".into(), id.clone());
        call.insert("name".into(), name.clone());
        call.insert("input".into(), input);
        if let Some(provider_executed) = entry.get("providerExecuted") {
            call.insert("providerExecuted".into(), provider_executed.clone());
        }
        Ok(vec![
            json!({ "type": "tool-input-end", "id": id, "name": name }),
            Value::Object(call),
        ])
    }

    /// Finish a tool call by parsing accumulated input.
    pub fn finish(state: Value, index: usize) -> LlmResult<Value> {
        let mut state = state;
        let key = index.to_string();
        let events = Self::finish_key(&mut state, &key, None)?;
        state["events"] = Value::Array(events);
        Ok(state)
    }

    /// Finish a tool call with an explicit input override.
    pub fn finish_with_input(state: Value, item: Value, input: &str) -> LlmResult<Value> {
        let mut state = state;
        let key = key_of(&item);
        let events = Self::finish_key(&mut state, &key, Some(input))?;
        state["events"] = Value::Array(events);
        Ok(state)
    }

    /// Finish every tracked tool call.
    pub fn finish_all(state: Value) -> LlmResult<Value> {
        let mut state = state;
        let keys: Vec<String> = state["tools"]
            .as_object()
            .map(|tools| tools.keys().cloned().collect())
            .unwrap_or_default();
        let mut events = Vec::new();
        for key in keys {
            events.extend(Self::finish_key(&mut state, &key, None)?);
        }
        state["events"] = Value::Array(events);
        Ok(state)
    }

    /// Whether a result is a `NotImplemented` error sentinel.
    pub fn is_error(value: &Value) -> bool {
        value.get("error").is_some()
    }
}
