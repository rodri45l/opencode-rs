//! Port of packages/app/src/pages/session/timeline/rows-current.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
enum ContentPart {
    Text(String),
    Reasoning(String),
}

#[derive(Clone, Debug, PartialEq)]
enum SessionMessageInfo {
    User {
        id: String,
        text: String,
        created: i64,
    },
    Assistant {
        id: String,
        agent: String,
        content: Vec<ContentPart>,
        created: i64,
        completed: Option<i64>,
        failed: bool,
    },
    Shell {
        id: String,
        command: String,
        status: String,
        exit: i64,
        created: i64,
        completed: Option<i64>,
    },
}

#[derive(Clone, Debug, PartialEq)]
enum TimelineRow {
    UserMessage {
        id: String,
    },
    AssistantPart {
        user_id: String,
        message_id: String,
        kind: String,
        index: i64,
    },
    TurnGap {
        id: String,
    },
    Thinking {
        id: String,
    },
}

impl TimelineRow {
    fn key(&self) -> String {
        match self {
            TimelineRow::UserMessage { id } => format!("user-message:{id}"),
            TimelineRow::AssistantPart {
                user_id,
                message_id,
                kind,
                index,
            } => format!("assistant-part:{user_id}:{message_id}:{kind}:{index}"),
            TimelineRow::TurnGap { id } => format!("turn-gap:{id}"),
            TimelineRow::Thinking { id } => format!("thinking:{id}"),
        }
    }

    fn tag(&self) -> &'static str {
        match self {
            TimelineRow::UserMessage { .. } => "UserMessage",
            TimelineRow::AssistantPart { .. } => "AssistantPart",
            TimelineRow::TurnGap { .. } => "TurnGap",
            TimelineRow::Thinking { .. } => "Thinking",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct ConstructResult {
    active_message_id: Option<String>,
    rows: Vec<TimelineRow>,
}

// Local stub (fast wave): real module lands later.
fn construct_session_message_rows(
    _source: &[SessionMessageInfo],
    _status: &str,
    _optimistic: bool,
    _users: &[String],
) -> ConstructResult {
    ConstructResult::default()
}

fn user(id: &str, text: &str, created: i64) -> SessionMessageInfo {
    SessionMessageInfo::User {
        id: id.into(),
        text: text.into(),
        created,
    }
}

fn assistant(
    id: &str,
    content: Vec<ContentPart>,
    created: i64,
    completed: Option<i64>,
    failed: bool,
) -> SessionMessageInfo {
    SessionMessageInfo::Assistant {
        id: id.into(),
        agent: "build".into(),
        content,
        created,
        completed,
        failed,
    }
}

fn keys(result: &ConstructResult) -> Vec<String> {
    result.rows.iter().map(TimelineRow::key).collect()
}

#[test]
#[ignore = "porting: pages/session/timeline/rows not implemented"]
fn derives_turns_and_tagged_rows_from_chronological_current_messages() {
    let source = vec![
        user("msg_1", "first", 1),
        assistant(
            "msg_2",
            vec![ContentPart::Text("answer".into())],
            2,
            Some(3),
            false,
        ),
        user("msg_3", "second", 4),
        assistant(
            "msg_4",
            vec![ContentPart::Reasoning("working".into())],
            5,
            None,
            false,
        ),
    ];

    let result =
        construct_session_message_rows(&source, "busy", true, &["msg_1".into(), "msg_3".into()]);

    assert_eq!(result.active_message_id, Some("msg_3".to_string()));
    assert_eq!(
        keys(&result),
        vec![
            "user-message:msg_1",
            "assistant-part:msg_1:msg_2:text:0",
            "turn-gap:msg_3",
            "user-message:msg_3",
            "assistant-part:msg_3:msg_4:reasoning:0",
        ]
    );
}

#[test]
#[ignore = "porting: pages/session/timeline/rows not implemented"]
fn renders_a_current_shell_message_as_a_standalone_turn() {
    let source = vec![SessionMessageInfo::Shell {
        id: "msg_shell".into(),
        command: "pwd".into(),
        status: "exited".into(),
        exit: 0,
        created: 1,
        completed: Some(2),
    }];

    let result = construct_session_message_rows(&source, "idle", true, &[]);

    assert_eq!(result.active_message_id, Some("msg_shell".to_string()));
    assert_eq!(
        keys(&result),
        vec![
            "user-message:msg_shell",
            "assistant-part:msg_shell:msg_shell:tool"
        ]
    );
}

#[test]
#[ignore = "porting: pages/session/timeline/rows not implemented"]
fn keeps_a_projected_parent_missing_from_the_source_page_before_newer_turns() {
    let source = [
        user("msg_user_1", "first question", 1),
        assistant(
            "msg_assistant_1",
            vec![ContentPart::Text("first answer".into())],
            2,
            Some(3),
            false,
        ),
        user("msg_user_2", "second question", 4),
        assistant(
            "msg_assistant_2",
            vec![ContentPart::Text("second answer".into())],
            5,
            Some(6),
            false,
        ),
    ];

    let result = construct_session_message_rows(
        &source[1..],
        "idle",
        true,
        &["msg_user_1".into(), "msg_user_2".into()],
    );

    assert_eq!(
        keys(&result),
        vec![
            "user-message:msg_user_1",
            "assistant-part:msg_user_1:msg_assistant_1:text:0",
            "turn-gap:msg_user_2",
            "user-message:msg_user_2",
            "assistant-part:msg_user_2:msg_assistant_2:text:0",
        ]
    );
}

#[test]
#[ignore = "porting: pages/session/timeline/rows not implemented"]
fn renders_an_optimistic_user_turn_and_thinking_before_the_protocol_message_arrives() {
    let source = vec![user("msg_z", "existing", 1)];

    let result =
        construct_session_message_rows(&source, "busy", true, &["msg_z".into(), "msg_a".into()]);

    assert_eq!(result.active_message_id, Some("msg_a".to_string()));
    assert_eq!(
        keys(&result),
        vec![
            "user-message:msg_z",
            "turn-gap:msg_a",
            "user-message:msg_a",
            "thinking:msg_a",
        ]
    );
}

#[test]
#[ignore = "porting: pages/session/timeline/rows not implemented"]
fn removes_a_failed_assistant_error_when_the_turn_continues_streaming() {
    let source = vec![
        user("msg_user", "recover", 1),
        assistant("msg_failed", Vec::new(), 2, Some(3), true),
        assistant(
            "msg_recovery",
            vec![ContentPart::Text("streaming again".into())],
            4,
            None,
            false,
        ),
    ];

    let result = construct_session_message_rows(&source, "busy", true, &["msg_user".into()]);

    assert_eq!(
        result.rows.iter().map(TimelineRow::tag).collect::<Vec<_>>(),
        vec!["UserMessage", "AssistantPart"]
    );
}
