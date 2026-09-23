//! Automatic cache-breakpoint placement.

use serde_json::{json, Map, Value};

use crate::error::{LlmError, LlmResult};

fn is_record(value: &Value) -> bool {
    value.is_object()
}

/// Apply the request's cache policy, returning the lowered request.
pub fn apply_cache_policy(mut request: Value) -> LlmResult<Value> {
    let policy = request.get("cache").cloned();
    let policy = match policy {
        None | Some(Value::Null) => json!("auto"),
        Some(other) => other,
    };

    if policy == json!("none") {
        return Ok(request);
    }

    let (tools, system, messages, ttl) = if policy == json!("auto") {
        (true, true, json!("latest-user-message"), None)
    } else if let Some(map) = policy.as_object() {
        (
            map.get("tools").and_then(Value::as_bool).unwrap_or(false),
            map.get("system").and_then(Value::as_bool).unwrap_or(false),
            map.get("messages").cloned().unwrap_or(Value::Null),
            map.get("ttlSeconds").cloned(),
        )
    } else {
        return Ok(request);
    };

    let hint = |ttl: &Option<Value>| -> Value {
        let mut map = Map::new();
        map.insert("type".into(), Value::String("ephemeral".into()));
        if let Some(ttl) = ttl {
            map.insert("ttlSeconds".into(), ttl.clone());
        }
        Value::Object(map)
    };

    if tools {
        if let Some(tool) = request
            .get_mut("tools")
            .and_then(Value::as_array_mut)
            .and_then(|tools| tools.last_mut())
        {
            if tool.get("cache").is_none() {
                tool["cache"] = hint(&ttl);
            }
        }
    }

    if system {
        if let Some(part) = request
            .get_mut("system")
            .and_then(Value::as_array_mut)
            .and_then(|parts| parts.last_mut())
        {
            if part.get("cache").is_none() {
                part["cache"] = hint(&ttl);
            }
        }
    }

    let mark_message = |message: &mut Value, ttl: &Option<Value>| {
        if let Some(part) = message
            .get_mut("content")
            .and_then(Value::as_array_mut)
            .and_then(|parts| parts.last_mut())
        {
            if part.get("cache").is_none() {
                part["cache"] = hint(ttl);
            }
        }
    };

    if let Some(messages_arr) = request.get_mut("messages").and_then(Value::as_array_mut) {
        match &messages {
            Value::String(mode) if mode == "latest-user-message" => {
                if let Some(message) = messages_arr
                    .iter_mut()
                    .rev()
                    .find(|m| m.get("role").and_then(Value::as_str) == Some("user"))
                {
                    mark_message(message, &ttl);
                }
            }
            Value::String(mode) if mode == "latest-assistant" => {
                if let Some(message) = messages_arr
                    .iter_mut()
                    .rev()
                    .find(|m| m.get("role").and_then(Value::as_str) == Some("assistant"))
                {
                    mark_message(message, &ttl);
                }
            }
            Value::Object(map) if map.contains_key("tail") => {
                let tail = map.get("tail").and_then(Value::as_u64).unwrap_or(0) as usize;
                let len = messages_arr.len();
                let start = len.saturating_sub(tail);
                for message in messages_arr.iter_mut().skip(start) {
                    mark_message(message, &ttl);
                }
            }
            _ => {}
        }
    }

    Ok(request)
}

/// Whether a value is a non-null record.
pub fn is_object(value: &Value) -> bool {
    is_record(value)
}

/// The crate error alias used by this module.
pub type CacheError = LlmError;
