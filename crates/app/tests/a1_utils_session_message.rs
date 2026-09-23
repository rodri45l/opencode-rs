//! Port of packages/app/src/utils/session-message.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
struct Message {
    id: String,
    role: String,
    parent_id: Option<String>,
    agent: Option<String>,
    model: Option<Model>,
    cost: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
struct Model {
    provider_id: String,
    model_id: String,
    variant: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct Part {
    id: String,
    part_type: String,
    tool: Option<String>,
}

#[derive(Default)]
struct Normalized {
    messages: Vec<Message>,
    parts: BTreeMap<String, Vec<Part>>,
}

// Local stub (fast wave): real module lands later.
fn normalize_session_messages(_session_id: &str, _source: &[SourceMessage]) -> Normalized {
    Normalized::default()
}

#[derive(Clone, Debug, PartialEq)]
enum SourceMessage {
    AgentSwitched {
        id: String,
        agent: String,
    },
    ModelSwitched {
        id: String,
        model: Model,
    },
    User {
        id: String,
        text: String,
        file_count: usize,
        agent_count: usize,
    },
    Assistant {
        id: String,
        agent: String,
        model: Model,
        content: Vec<ContentPart>,
        cost: f64,
    },
    Compaction {
        id: String,
    },
    Shell {
        id: String,
        command: String,
        output: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
enum ContentPart {
    Reasoning,
    Text,
    Tool {
        id: String,
        name: String,
        output: String,
    },
}

fn source() -> Vec<SourceMessage> {
    vec![
        SourceMessage::AgentSwitched {
            id: "msg_1".into(),
            agent: "build".into(),
        },
        SourceMessage::ModelSwitched {
            id: "msg_2".into(),
            model: Model {
                provider_id: "anthropic".into(),
                model_id: "claude".into(),
                variant: Some("high".into()),
            },
        },
        SourceMessage::User {
            id: "msg_3".into(),
            text: "inspect @src/client.ts".into(),
            file_count: 2,
            agent_count: 1,
        },
        SourceMessage::Assistant {
            id: "msg_4".into(),
            agent: "build".into(),
            model: Model {
                provider_id: "anthropic".into(),
                model_id: "claude".into(),
                variant: Some("high".into()),
            },
            content: vec![
                ContentPart::Reasoning,
                ContentPart::Text,
                ContentPart::Tool {
                    id: "call_1".into(),
                    name: "read".into(),
                    output: "hello".into(),
                },
            ],
            cost: 0.1,
        },
        SourceMessage::Compaction { id: "msg_5".into() },
    ]
}

#[test]
#[ignore = "porting: utils/session-message not implemented"]
fn projects_current_turns_into_stable_legacy_rendering_records() {
    let result = normalize_session_messages("ses_1", &source());
    assert_eq!(result.messages.len(), 2);
    assert_eq!(result.messages[0].id, "msg_3");
    assert_eq!(result.messages[0].role, "user");
    assert_eq!(result.messages[0].agent.as_deref(), Some("build"));
    assert_eq!(
        result.messages[0].model,
        Some(Model {
            provider_id: "anthropic".into(),
            model_id: "claude".into(),
            variant: Some("high".into())
        })
    );
    assert_eq!(result.messages[1].id, "msg_4");
    assert_eq!(result.messages[1].role, "assistant");
    assert_eq!(result.messages[1].parent_id.as_deref(), Some("msg_3"));
    assert_eq!(result.messages[1].cost, Some(0.1));

    assert_eq!(
        result
            .parts
            .get("msg_3")
            .map(|p| p.iter().map(|x| x.id.clone()).collect::<Vec<_>>()),
        Some(vec![
            "msg_3:text:0".to_string(),
            "msg_3:file:0".to_string(),
            "msg_3:file:1".to_string(),
            "msg_3:agent:0".to_string(),
            "msg_5:compaction".to_string(),
        ])
    );
    assert_eq!(
        result
            .parts
            .get("msg_4")
            .map(|p| p.iter().map(|x| x.id.clone()).collect::<Vec<_>>()),
        Some(vec![
            "msg_4:reasoning:0".to_string(),
            "msg_4:text:0".to_string(),
            "call_1".to_string(),
        ])
    );
}

#[test]
#[ignore = "porting: utils/session-message not implemented"]
fn does_not_invent_a_parent_for_an_assistant_only_page() {
    let source = vec![SourceMessage::Assistant {
        id: "msg_2".into(),
        agent: "build".into(),
        model: Model {
            provider_id: "provider".into(),
            model_id: "model".into(),
            variant: None,
        },
        content: vec![ContentPart::Text],
        cost: 0.0,
    }];
    assert!(normalize_session_messages("ses_1", &source)
        .messages
        .is_empty());
}

#[test]
#[ignore = "porting: utils/session-message not implemented"]
fn projects_a_current_shell_message_into_a_renderable_standalone_turn() {
    let source = vec![SourceMessage::Shell {
        id: "msg_shell".into(),
        command: "printf hello".into(),
        output: "hello".into(),
    }];
    let result = normalize_session_messages("ses_1", &source);
    assert_eq!(result.messages.len(), 2);
    assert_eq!(result.messages[0].id, "msg_shell");
    assert_eq!(result.messages[0].role, "user");
    assert_eq!(result.messages[1].id, "msg_shell:assistant");
    assert_eq!(result.messages[1].role, "assistant");
    assert_eq!(result.messages[1].parent_id.as_deref(), Some("msg_shell"));
    assert_eq!(result.parts.get("msg_shell").map(|p| p.len()), Some(1));
    assert_eq!(
        result
            .parts
            .get("msg_shell:assistant")
            .and_then(|p| p.first())
            .and_then(|p| p.tool.clone()),
        Some("bash".to_string())
    );
}

#[test]
#[ignore = "porting: utils/session-message not implemented"]
fn adapts_current_edit_fields_for_the_legacy_edit_renderer() {
    let source = vec![
        SourceMessage::User {
            id: "msg_user".into(),
            text: "edit it".into(),
            file_count: 0,
            agent_count: 0,
        },
        SourceMessage::Assistant {
            id: "msg_assistant".into(),
            agent: "build".into(),
            model: Model {
                provider_id: "provider".into(),
                model_id: "model".into(),
                variant: None,
            },
            content: vec![ContentPart::Tool {
                id: "call_edit".into(),
                name: "edit".into(),
                output: "Edited file successfully".into(),
            }],
            cost: 0.0,
        },
    ];
    let result = normalize_session_messages("ses_1", &source);
    let part = result
        .parts
        .get("msg_assistant")
        .and_then(|p| p.first())
        .cloned();
    assert_eq!(
        part.as_ref().map(|p| p.tool.clone()),
        Some(Some("edit".to_string()))
    );
}
