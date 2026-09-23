//! Port of packages/tui/test/util/transcript.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/util/transcript.ts; see docs/TEST-PORT.md.

use opencode_tui::transcript::{
    format_assistant_header, format_message, format_part, format_transcript, AssistantInfo,
    MessageBody, MessageInfo, Part, ProviderInfo, ToolPart, TranscriptOptions, UserInfo,
};
use serde_json::json;

fn providers() -> Vec<ProviderInfo> {
    vec![ProviderInfo {
        id: "anthropic".to_string(),
        name: "Anthropic".to_string(),
        models: vec![(
            "claude-sonnet-4-20250514".to_string(),
            "Claude Sonnet 4".to_string(),
        )],
    }]
}

fn assistant() -> AssistantInfo {
    AssistantInfo {
        agent: "build".to_string(),
        model_id: "claude-sonnet-4-20250514".to_string(),
        provider_id: "anthropic".to_string(),
        created: 1_000_000,
        completed: Some(1_005_400),
    }
}

fn options() -> TranscriptOptions {
    TranscriptOptions {
        thinking: true,
        tool_details: true,
        assistant_metadata: true,
        providers: providers(),
    }
}

#[test]
fn includes_metadata_when_enabled() {
    assert_eq!(
        format_assistant_header(&assistant(), true, &[]),
        "## Assistant (Build · claude-sonnet-4-20250514 · 5.4s)\n\n"
    );
}

#[test]
fn uses_model_display_name_when_available() {
    assert_eq!(
        format_assistant_header(&assistant(), true, &providers()),
        "## Assistant (Build · Claude Sonnet 4 · 5.4s)\n\n"
    );
}

#[test]
fn excludes_metadata_when_disabled() {
    assert_eq!(
        format_assistant_header(&assistant(), false, &[]),
        "## Assistant\n\n"
    );
}

#[test]
fn handles_missing_completed_time() {
    let message = AssistantInfo {
        completed: None,
        ..assistant()
    };
    assert_eq!(
        format_assistant_header(&message, true, &[]),
        "## Assistant (Build · claude-sonnet-4-20250514)\n\n"
    );
}

#[test]
fn titlecases_agent_name() {
    let message = AssistantInfo {
        agent: "plan".to_string(),
        ..assistant()
    };
    assert!(format_assistant_header(&message, true, &[]).contains("Plan"));
}

#[test]
fn formats_text_part() {
    let part = Part::Text {
        text: "Hello world".to_string(),
        synthetic: false,
    };
    assert_eq!(format_part(&part, &options()), "Hello world\n\n");
}

#[test]
fn skips_synthetic_text_parts() {
    let part = Part::Text {
        text: "Synthetic content".to_string(),
        synthetic: true,
    };
    assert_eq!(format_part(&part, &options()), "");
}

#[test]
fn formats_reasoning_when_thinking_enabled() {
    let part = Part::Reasoning {
        text: "Let me think...".to_string(),
    };
    assert_eq!(
        format_part(&part, &options()),
        "_Thinking:_\n\nLet me think...\n\n"
    );
}

#[test]
fn skips_reasoning_when_thinking_disabled() {
    let part = Part::Reasoning {
        text: "Let me think...".to_string(),
    };
    let options = TranscriptOptions {
        thinking: false,
        ..options()
    };
    assert_eq!(format_part(&part, &options), "");
}

#[test]
fn formats_tool_part_with_details() {
    let part = Part::Tool(ToolPart {
        tool: "bash".to_string(),
        status: "completed".to_string(),
        input: json!({ "command": "ls" }),
        output: Some("file1.txt\nfile2.txt".to_string()),
        error: None,
    });
    let result = format_part(&part, &options());
    assert!(result.contains("**Tool: bash**"));
    assert!(result.contains("**Input:**"));
    assert!(result.contains("\"command\": \"ls\""));
    assert!(result.contains("**Output:**"));
    assert!(result.contains("file1.txt"));
}

#[test]
fn formats_tool_output_containing_triple_backticks_without_breaking_markdown() {
    let part = Part::Tool(ToolPart {
        tool: "bash".to_string(),
        status: "completed".to_string(),
        input: json!({ "command": "echo '```hello```'" }),
        output: Some("```hello```".to_string()),
        error: None,
    });
    let result = format_part(&part, &options());
    assert!(result.starts_with("**Tool: bash**\n"));
    assert!(result.contains("**Input:**\n```json"));
    assert!(result.contains("**Output:**\n```\n```hello```\n```"));
}

#[test]
fn formats_tool_part_without_details_when_disabled() {
    let part = Part::Tool(ToolPart {
        tool: "bash".to_string(),
        status: "completed".to_string(),
        input: json!({ "command": "ls" }),
        output: Some("file1.txt".to_string()),
        error: None,
    });
    let options = TranscriptOptions {
        tool_details: false,
        ..options()
    };
    let result = format_part(&part, &options);
    assert!(result.contains("**Tool: bash**"));
    assert!(!result.contains("**Input:**"));
    assert!(!result.contains("**Output:**"));
}

#[test]
fn formats_tool_error() {
    let part = Part::Tool(ToolPart {
        tool: "bash".to_string(),
        status: "error".to_string(),
        input: json!({ "command": "invalid" }),
        output: None,
        error: Some("Command failed".to_string()),
    });
    let result = format_part(&part, &options());
    assert!(result.contains("**Error:**"));
    assert!(result.contains("Command failed"));
}

#[test]
fn formats_user_message() {
    let message = MessageInfo {
        id: "msg_123".to_string(),
        created: 1_000_000,
        body: MessageBody::User(UserInfo {
            agent: "build".to_string(),
            provider_id: "anthropic".to_string(),
            model_id: "claude-sonnet-4-20250514".to_string(),
        }),
    };
    let parts = vec![Part::Text {
        text: "Hello".to_string(),
        synthetic: false,
    }];
    let result = format_message(&message, &parts, &options());
    assert!(result.contains("## User"));
    assert!(result.contains("Hello"));
}

#[test]
fn formats_assistant_message_with_metadata() {
    let message = MessageInfo {
        id: "msg_123".to_string(),
        created: 1_000_000,
        body: MessageBody::Assistant(assistant()),
    };
    let parts = vec![Part::Text {
        text: "Hi there".to_string(),
        synthetic: false,
    }];
    let result = format_message(&message, &parts, &options());
    assert!(result.contains("## Assistant (Build · Claude Sonnet 4 · 5.4s)"));
    assert!(result.contains("Hi there"));
}

#[test]
fn formats_complete_transcript() {
    let messages = vec![
        (
            MessageInfo {
                id: "msg_1".to_string(),
                created: 1_000_000_000_000,
                body: MessageBody::User(UserInfo {
                    agent: "build".to_string(),
                    provider_id: "anthropic".to_string(),
                    model_id: "claude-sonnet-4-20250514".to_string(),
                }),
            },
            vec![Part::Text {
                text: "Hello".to_string(),
                synthetic: false,
            }],
        ),
        (
            MessageInfo {
                id: "msg_2".to_string(),
                created: 1_000_000_000_100,
                body: MessageBody::Assistant(AssistantInfo {
                    agent: "build".to_string(),
                    model_id: "claude-sonnet-4-20250514".to_string(),
                    provider_id: "anthropic".to_string(),
                    created: 1_000_000_000_100,
                    completed: Some(1_000_000_000_600),
                }),
            },
            vec![Part::Text {
                text: "Hi!".to_string(),
                synthetic: false,
            }],
        ),
    ];
    let options = TranscriptOptions {
        thinking: false,
        tool_details: false,
        assistant_metadata: true,
        providers: providers(),
    };
    let result = format_transcript("Test Session", "ses_abc123", &messages, &options);

    assert!(result.contains("# Test Session"));
    assert!(result.contains("**Session ID:** ses_abc123"));
    assert!(result.contains("## User"));
    assert!(result.contains("Hello"));
    assert!(result.contains("## Assistant (Build · Claude Sonnet 4 · 0.5s)"));
    assert!(result.contains("Hi!"));
    assert!(result.contains("---"));
}

#[test]
fn orders_messages_by_creation_time_and_preserves_part_order() {
    let message = |id: &str, created: i64, parts: &[&str]| {
        (
            MessageInfo {
                id: id.to_string(),
                created,
                body: MessageBody::User(UserInfo {
                    agent: "build".to_string(),
                    provider_id: "anthropic".to_string(),
                    model_id: "claude".to_string(),
                }),
            },
            parts
                .iter()
                .map(|text| Part::Text {
                    text: text.to_string(),
                    synthetic: false,
                })
                .collect::<Vec<_>>(),
        )
    };
    let messages = vec![
        message("msg_a", 30, &["third"]),
        message("msg_z", 10, &["first", "second"]),
    ];
    let options = TranscriptOptions {
        thinking: false,
        tool_details: false,
        assistant_metadata: false,
        providers: Vec::new(),
    };
    let result = format_transcript("Order", "ses_abc123", &messages, &options);

    assert!(result.find("first").unwrap() < result.find("second").unwrap());
    assert!(result.find("second").unwrap() < result.find("third").unwrap());
}

#[test]
fn falls_back_to_raw_model_id_when_provider_data_is_missing() {
    let messages = vec![(
        MessageInfo {
            id: "msg_1".to_string(),
            created: 1_000_000_000_100,
            body: MessageBody::Assistant(AssistantInfo {
                agent: "build".to_string(),
                model_id: "claude-sonnet-4-20250514".to_string(),
                provider_id: "anthropic".to_string(),
                created: 1_000_000_000_100,
                completed: Some(1_000_000_000_600),
            }),
        },
        vec![Part::Text {
            text: "Response".to_string(),
            synthetic: false,
        }],
    )];
    let options = TranscriptOptions {
        thinking: false,
        tool_details: false,
        assistant_metadata: true,
        providers: Vec::new(),
    };
    let result = format_transcript("Test Session", "ses_abc123", &messages, &options);

    assert!(result.contains("## Assistant (Build · claude-sonnet-4-20250514 · 0.5s)"));
}

#[test]
fn formats_transcript_without_assistant_metadata() {
    let messages = vec![(
        MessageInfo {
            id: "msg_1".to_string(),
            created: 1_000_000_000_100,
            body: MessageBody::Assistant(AssistantInfo {
                agent: "build".to_string(),
                model_id: "claude-sonnet-4-20250514".to_string(),
                provider_id: "anthropic".to_string(),
                created: 1_000_000_000_100,
                completed: Some(1_000_000_000_600),
            }),
        },
        vec![Part::Text {
            text: "Response".to_string(),
            synthetic: false,
        }],
    )];
    let options = TranscriptOptions {
        thinking: false,
        tool_details: false,
        assistant_metadata: false,
        providers: Vec::new(),
    };
    let result = format_transcript("Test Session", "ses_abc123", &messages, &options);

    assert!(result.contains("## Assistant\n\n"));
    assert!(!result.contains("Build"));
    assert!(!result.contains("claude-sonnet-4-20250514"));
}
