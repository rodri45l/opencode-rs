//! Port of packages/tui/test/cli/tui/data.test.tsx (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/context/data.tsx; see docs/TEST-PORT.md.
//! SDK fetching and the Solid store wiring are replaced by an equivalent
//! in-memory event reducer.
#![allow(dead_code)]

use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
struct Provider {
    executed: bool,
    metadata: Option<Value>,
    result_metadata: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
enum ToolState {
    Pending {
        input: String,
    },
    Running {
        input: Value,
        structured: Value,
        content: Vec<Value>,
    },
    Error {
        error: Value,
        input: Value,
        structured: Value,
        content: Vec<Value>,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct Tool {
    id: String,
    state: ToolState,
    provider: Provider,
}

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Assistant {
        id: String,
        content: Vec<Tool>,
        completed: bool,
    },
    ModelSwitched {
        id: String,
    },
    AgentSwitched {
        id: String,
    },
    User {
        id: String,
        text: String,
    },
    System {
        id: String,
        text: String,
    },
}

impl Message {
    fn id(&self) -> &str {
        match self {
            Message::Assistant { id, .. }
            | Message::ModelSwitched { id }
            | Message::AgentSwitched { id }
            | Message::User { id, .. }
            | Message::System { id, .. } => id,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Message::Assistant { .. } => "assistant",
            Message::ModelSwitched { .. } => "model-switched",
            Message::AgentSwitched { .. } => "agent-switched",
            Message::User { .. } => "user",
            Message::System { .. } => "system",
        }
    }
}

type Store = BTreeMap<String, Vec<Message>>;

fn prepend(store: &mut Store, session_id: &str, message: Message) {
    let list = store.entry(session_id.to_string()).or_default();
    if list.iter().any(|existing| existing.id() == message.id()) {
        return;
    }
    list.insert(0, message);
}

fn assistant_mut<'a>(store: &'a mut Store, session_id: &str, id: &str) -> Option<&'a mut Message> {
    store
        .get_mut(session_id)?
        .iter_mut()
        .find(|message| matches!(message, Message::Assistant { id: current, .. } if current.as_str() == id))
}

fn latest_tool_mut<'a>(
    store: &'a mut Store,
    session_id: &str,
    assistant_id: &str,
    call_id: &str,
) -> Option<&'a mut Tool> {
    let assistant = store.get_mut(session_id)?.iter_mut().find(
        |message| matches!(message, Message::Assistant { id, .. } if id.as_str() == assistant_id),
    )?;
    match assistant {
        Message::Assistant { content, .. } => {
            content.iter_mut().rev().find(|tool| tool.id == call_id)
        }
        _ => None,
    }
}

fn property<'a>(event: &'a Value, key: &str) -> Option<&'a Value> {
    event.get("properties")?.get(key)
}

fn string_property(event: &Value, key: &str) -> Option<String> {
    property(event, key)
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn provider_from(value: Option<&Value>) -> Provider {
    let executed = value
        .and_then(|provider| provider.get("executed"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let metadata = value.and_then(|provider| provider.get("metadata")).cloned();
    Provider {
        executed,
        metadata,
        result_metadata: None,
    }
}

fn apply(store: &mut Store, event: &Value) {
    let Some(kind) = event.get("type").and_then(Value::as_str) else {
        return;
    };
    let Some(session_id) = string_property(event, "sessionID") else {
        return;
    };

    match kind {
        "session.next.agent.switched" => {
            if let Some(message_id) = string_property(event, "messageID") {
                prepend(
                    store,
                    &session_id,
                    Message::AgentSwitched { id: message_id },
                );
            }
        }
        "session.next.model.switched" => {
            if let Some(message_id) = string_property(event, "messageID") {
                prepend(
                    store,
                    &session_id,
                    Message::ModelSwitched { id: message_id },
                );
            }
        }
        "session.next.step.started" => {
            let Some(message_id) = string_property(event, "assistantMessageID") else {
                return;
            };
            let list = store.entry(session_id.clone()).or_default();
            if list
                .iter()
                .any(|message| message.id() == message_id.as_str())
            {
                return;
            }
            if let Some(Message::Assistant { completed, .. }) = list.iter_mut().find(
                |message| matches!(message, Message::Assistant { completed, .. } if !completed),
            ) {
                *completed = true;
            }
            prepend(
                store,
                &session_id,
                Message::Assistant {
                    id: message_id,
                    content: Vec::new(),
                    completed: false,
                },
            );
        }
        "session.next.tool.input.started" => {
            let (Some(assistant_id), Some(call_id)) = (
                string_property(event, "assistantMessageID"),
                string_property(event, "callID"),
            ) else {
                return;
            };
            if let Some(Message::Assistant { content, .. }) =
                assistant_mut(store, &session_id, &assistant_id)
            {
                content.push(Tool {
                    id: call_id,
                    state: ToolState::Pending {
                        input: String::new(),
                    },
                    provider: Provider {
                        executed: false,
                        metadata: None,
                        result_metadata: None,
                    },
                });
            }
        }
        "session.next.tool.called" => {
            let (Some(assistant_id), Some(call_id)) = (
                string_property(event, "assistantMessageID"),
                string_property(event, "callID"),
            ) else {
                return;
            };
            let input = property(event, "input")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let provider = provider_from(property(event, "provider"));
            if let Some(tool) = latest_tool_mut(store, &session_id, &assistant_id, &call_id) {
                tool.state = ToolState::Running {
                    input,
                    structured: json!({}),
                    content: Vec::new(),
                };
                tool.provider = provider;
            }
        }
        "session.next.tool.failed" => {
            let (Some(assistant_id), Some(call_id)) = (
                string_property(event, "assistantMessageID"),
                string_property(event, "callID"),
            ) else {
                return;
            };
            let error = property(event, "error").cloned().unwrap_or(Value::Null);
            let result_metadata = property(event, "provider")
                .and_then(|provider| provider.get("metadata"))
                .cloned();
            let provider_executed = property(event, "provider")
                .and_then(|provider| provider.get("executed"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if let Some(tool) = latest_tool_mut(store, &session_id, &assistant_id, &call_id) {
                let (input, structured, content) = match &tool.state {
                    ToolState::Pending { .. } => (json!({}), json!({}), Vec::new()),
                    ToolState::Running {
                        input,
                        structured,
                        content,
                    } => (input.clone(), structured.clone(), content.clone()),
                    ToolState::Error { .. } => return,
                };
                let previous_executed = tool.provider.executed;
                let previous_metadata = tool.provider.metadata.clone();
                tool.state = ToolState::Error {
                    error,
                    input,
                    structured,
                    content,
                };
                tool.provider = Provider {
                    executed: provider_executed || previous_executed,
                    metadata: previous_metadata,
                    result_metadata,
                };
            }
        }
        "session.next.prompted" => {
            let (Some(message_id), Some(text)) = (
                string_property(event, "messageID"),
                property(event, "prompt")
                    .and_then(|prompt| prompt.get("text"))
                    .and_then(Value::as_str),
            ) else {
                return;
            };
            prepend(
                store,
                &session_id,
                Message::User {
                    id: message_id,
                    text: text.to_string(),
                },
            );
        }
        "session.next.prompt.admitted" => {}
        "session.next.context.updated" => {
            let (Some(message_id), Some(text)) = (
                string_property(event, "messageID"),
                string_property(event, "text"),
            ) else {
                return;
            };
            prepend(
                store,
                &session_id,
                Message::System {
                    id: message_id,
                    text,
                },
            );
        }
        _ => {}
    }
}

fn evidence() -> Vec<Value> {
    vec![
        json!({ "type": "session.next.agent.switched", "properties": { "sessionID": "session-1", "messageID": "msg_agent_1", "timestamp": 0, "agent": "build" } }),
        json!({ "type": "session.next.model.switched", "properties": { "sessionID": "session-1", "messageID": "msg_model_1", "timestamp": 0, "model": { "id": "model-1", "providerID": "provider-1" } } }),
        json!({ "type": "session.next.step.started", "properties": { "sessionID": "session-1", "assistantMessageID": "msg_explicit_assistant_9", "timestamp": 1, "agent": "build", "model": { "id": "model-1", "providerID": "provider-1" } } }),
        json!({ "type": "session.next.tool.input.started", "properties": { "sessionID": "session-1", "assistantMessageID": "msg_explicit_assistant_9", "timestamp": 2, "callID": "call-1", "name": "bash" } }),
        json!({ "type": "session.next.tool.called", "properties": { "sessionID": "session-1", "timestamp": 2, "assistantMessageID": "msg_explicit_assistant_9", "callID": "call-1", "tool": "bash", "input": {}, "provider": { "executed": false, "metadata": { "fake": { "call": true } } } } }),
        json!({ "type": "session.next.tool.failed", "properties": { "sessionID": "session-1", "timestamp": 3, "assistantMessageID": "msg_explicit_assistant_9", "callID": "call-1", "error": { "type": "unknown", "message": "aborted" }, "provider": { "executed": false, "metadata": { "fake": { "result": true } } } } }),
    ]
}

#[test]
fn settles_pending_tools_when_a_live_failure_arrives() {
    let mut store = Store::new();
    for event in evidence() {
        apply(&mut store, &event);
    }

    let messages = &store["session-1"];
    assert_eq!(
        messages.iter().map(Message::kind).collect::<Vec<_>>(),
        vec!["assistant", "model-switched", "agent-switched"]
    );

    let assistant = messages.first().unwrap();
    let (id, content) = match assistant {
        Message::Assistant { id, content, .. } => (id, content),
        other => panic!("expected assistant, got {other:?}"),
    };
    assert_eq!(id, "msg_explicit_assistant_9");
    let tool = &content[0];
    match &tool.state {
        ToolState::Error {
            error,
            input,
            structured,
            content,
        } => {
            assert_eq!(error, &json!({ "type": "unknown", "message": "aborted" }));
            assert_eq!(input, &json!({}));
            assert_eq!(structured, &json!({}));
            assert_eq!(content, &Vec::<Value>::new());
        }
        other => panic!("expected error state, got {other:?}"),
    }
    assert_eq!(
        tool.provider,
        Provider {
            executed: false,
            metadata: Some(json!({ "fake": { "call": true } })),
            result_metadata: Some(json!({ "fake": { "result": true } })),
        }
    );
}

#[test]
fn renders_admitted_prompts_only_after_they_become_model_visible() {
    let mut store = Store::new();
    let admitted = json!({ "type": "session.next.prompt.admitted", "properties": { "sessionID": "session-1", "messageID": "msg_user_1", "timestamp": 0, "prompt": { "text": "hello" }, "delivery": "steer" } });
    apply(&mut store, &admitted);
    assert_eq!(
        store.get("session-1").cloned().unwrap_or_default(),
        Vec::new()
    );

    let prompted = json!({ "type": "session.next.prompted", "properties": { "sessionID": "session-1", "messageID": "msg_user_1", "timestamp": 0, "prompt": { "text": "hello" }, "delivery": "steer" } });
    apply(&mut store, &prompted);

    let messages = &store["session-1"];
    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::User { id, text } => {
            assert_eq!(id, "msg_user_1");
            assert_eq!(text, "hello");
        }
        other => panic!("expected user message, got {other:?}"),
    }
}

#[test]
fn projects_live_context_updates_with_their_message_id() {
    let mut store = Store::new();
    apply(
        &mut store,
        &json!({ "type": "session.next.context.updated", "properties": { "sessionID": "session-1", "messageID": "msg_context_1", "timestamp": 1, "text": "Updated context" } }),
    );

    let messages = &store["session-1"];
    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::System { id, text } => {
            assert_eq!(id, "msg_context_1");
            assert_eq!(text, "Updated context");
        }
        other => panic!("expected system message, got {other:?}"),
    }
}
