//! Port of packages/app/src/context/server-session-v2-reducer.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct SessionMessageInfo {
    id: String,
    message_type: String,
    text: Option<String>,
    content: Vec<Content>,
    retry: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct Content {
    content_type: String,
    text: Option<String>,
    id: Option<String>,
    tool_state_status: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
enum V2Event {
    InputAdmitted {
        input_id: String,
        text: String,
    },
    InputPromoted {
        input_id: String,
    },
    StepStarted {
        assistant_message_id: String,
    },
    TextStarted {
        assistant_message_id: String,
        ordinal: i64,
    },
    TextDelta {
        assistant_message_id: String,
        ordinal: i64,
        delta: String,
    },
    TextEnded {
        assistant_message_id: String,
        ordinal: i64,
        text: String,
    },
    ToolInputStarted {
        assistant_message_id: String,
        call_id: String,
        name: String,
    },
    ToolInputDelta {
        assistant_message_id: String,
        call_id: String,
        delta: String,
    },
    ToolCalled {
        assistant_message_id: String,
        call_id: String,
    },
    ToolSuccess {
        assistant_message_id: String,
        call_id: String,
        text: String,
    },
    RetryScheduled {
        assistant_message_id: String,
        attempt: i64,
    },
    ExecutionSucceeded,
}

#[derive(Clone, Debug, PartialEq)]
struct ReduceResult {
    messages: Vec<SessionMessageInfo>,
    session_id: String,
    missing: Option<String>,
    touched: Vec<String>,
}

struct V2SessionReducer;

impl V2SessionReducer {
    // Local stub (fast wave): real module lands later.
    fn reduce(&self, _messages: &[SessionMessageInfo], _event: V2Event) -> Option<ReduceResult> {
        None
    }
}

fn create_v2_session_reducer() -> V2SessionReducer {
    V2SessionReducer
}

#[test]
#[ignore = "porting: context/server-session-v2-reducer not implemented"]
fn projects_promoted_input_and_streaming_assistant_content() {
    let reducer = create_v2_session_reducer();
    let mut messages: Vec<SessionMessageInfo> = Vec::new();
    let mut apply = |event: V2Event| {
        if let Some(result) = reducer.reduce(&messages, event) {
            messages = result.messages;
        }
    };

    apply(V2Event::InputAdmitted {
        input_id: "msg_user".into(),
        text: "hello".into(),
    });
    apply(V2Event::InputPromoted {
        input_id: "msg_user".into(),
    });
    apply(V2Event::StepStarted {
        assistant_message_id: "msg_assistant".into(),
    });
    apply(V2Event::TextStarted {
        assistant_message_id: "msg_assistant".into(),
        ordinal: 0,
    });
    apply(V2Event::TextDelta {
        assistant_message_id: "msg_assistant".into(),
        ordinal: 0,
        delta: "hel".into(),
    });
    apply(V2Event::TextEnded {
        assistant_message_id: "msg_assistant".into(),
        ordinal: 0,
        text: "hello".into(),
    });

    assert_eq!(messages[0].id, "msg_user");
    assert_eq!(messages[0].message_type, "user");
    assert_eq!(messages[0].text.as_deref(), Some("hello"));
    assert_eq!(messages[1].id, "msg_assistant");
    assert_eq!(messages[1].message_type, "assistant");
    assert_eq!(
        messages[1].content,
        vec![Content {
            content_type: "text".into(),
            text: Some("hello".into()),
            id: None,
            tool_state_status: None
        }]
    );
}

#[test]
#[ignore = "porting: context/server-session-v2-reducer not implemented"]
fn folds_tool_retry_and_completion_events() {
    let reducer = create_v2_session_reducer();
    let mut messages: Vec<SessionMessageInfo> = Vec::new();
    let mut apply = |event: V2Event| {
        if let Some(result) = reducer.reduce(&messages, event) {
            messages = result.messages;
        }
    };

    apply(V2Event::StepStarted {
        assistant_message_id: "msg_assistant".into(),
    });
    apply(V2Event::ToolInputStarted {
        assistant_message_id: "msg_assistant".into(),
        call_id: "call_1".into(),
        name: "bash".into(),
    });
    apply(V2Event::ToolInputDelta {
        assistant_message_id: "msg_assistant".into(),
        call_id: "call_1".into(),
        delta: "{}".into(),
    });
    apply(V2Event::ToolCalled {
        assistant_message_id: "msg_assistant".into(),
        call_id: "call_1".into(),
    });
    apply(V2Event::ToolSuccess {
        assistant_message_id: "msg_assistant".into(),
        call_id: "call_1".into(),
        text: "done".into(),
    });
    apply(V2Event::RetryScheduled {
        assistant_message_id: "msg_assistant".into(),
        attempt: 2,
    });
    apply(V2Event::ExecutionSucceeded);

    assert_eq!(messages[0].message_type, "assistant");
    assert_eq!(messages[0].retry, None);
    assert_eq!(
        messages[0].content,
        vec![Content {
            content_type: "tool".into(),
            text: Some("done".into()),
            id: Some("call_1".into()),
            tool_state_status: Some("completed".into())
        }]
    );
}

#[test]
#[ignore = "porting: context/server-session-v2-reducer not implemented"]
fn requests_hydration_when_promotion_admission_was_missed() {
    let result = create_v2_session_reducer().reduce(
        &[],
        V2Event::InputPromoted {
            input_id: "msg_user".into(),
        },
    );
    assert_eq!(
        result,
        Some(ReduceResult {
            messages: Vec::new(),
            session_id: "ses_1".into(),
            missing: Some("msg_user".into()),
            touched: Vec::new()
        })
    );
}
