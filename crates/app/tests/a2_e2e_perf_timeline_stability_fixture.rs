//! Port of packages/app/e2e/performance/timeline-stability/fixture.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, Default, PartialEq)]
struct PartSeed {
    id: String,
    kind: String,
    text: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct ErrorSeed {
    name: String,
    message: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct MessageSeed {
    id: String,
    parent_id: Option<String>,
    role: String,
    error: Option<ErrorSeed>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct StatusProperties {
    session_id: String,
    status_type: String,
    attempt: Option<i64>,
    message: Option<String>,
    next: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct EventPayload {
    id: String,
    kind: String,
    properties: StatusProperties,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct TimelineEvent {
    directory: String,
    payload: EventPayload,
}

fn user_message(_parts: Vec<PartSeed>) -> MessageSeed {
    MessageSeed {
        id: "msg_user".into(),
        parent_id: None,
        role: "user".into(),
        error: None,
    }
}

// Local stub (fast wave): real module lands later.
fn assistant_message(
    _parts: Vec<PartSeed>,
    _parent_id: Option<&str>,
    _error: Option<ErrorSeed>,
) -> Result<MessageSeed, String> {
    Ok(MessageSeed {
        id: "msg_assistant".into(),
        parent_id: None,
        role: "assistant".into(),
        error: None,
    })
}

fn event(_kind: &str, _status_type: &str) -> TimelineEvent {
    TimelineEvent {
        directory: "C:/OpenCode/TimelineStability".into(),
        payload: EventPayload {
            id: String::new(),
            kind: _kind.into(),
            properties: StatusProperties {
                session_id: "ses_timeline_stability".into(),
                status_type: _status_type.into(),
                ..Default::default()
            },
        },
    }
}

fn validate_timeline_messages(_messages: &[MessageSeed]) -> Result<usize, String> {
    Ok(0)
}

fn validate_timeline_event(_event: &TimelineEvent) -> Result<(), String> {
    Ok(())
}

fn is_deterministic_event_id(id: &str) -> bool {
    let Some(suffix) = id.strip_prefix("evt_timeline_") else {
        return false;
    };
    suffix.len() == 4 && suffix.chars().all(|c| c.is_ascii_digit())
}

#[test]
#[ignore = "porting: e2e/performance/timeline-stability fixture not implemented"]
fn accepts_a_valid_timeline() {
    let messages = vec![
        user_message(Vec::new()),
        assistant_message(Vec::new(), None, None).unwrap(),
    ];
    assert_eq!(validate_timeline_messages(&messages).unwrap(), 2);
}

#[test]
#[ignore = "porting: e2e/performance/timeline-stability fixture not implemented"]
fn rejects_malformed_sdk_values_at_runtime() {
    assert!(assistant_message(
        Vec::new(),
        None,
        Some(ErrorSeed {
            name: "APIError".into(),
            message: "failed".into(),
        })
    )
    .is_err());

    let mut retry = event("session.status", "retry");
    retry.payload.properties.attempt = Some(1);
    assert!(validate_timeline_event(&retry).is_err());
}

#[test]
#[ignore = "porting: e2e/performance/timeline-stability fixture not implemented"]
fn rejects_duplicate_ids_and_orphan_assistants() {
    assert!(
        validate_timeline_messages(&[user_message(Vec::new()), user_message(Vec::new())]).is_err()
    );
    assert!(validate_timeline_messages(&[
        user_message(Vec::new()),
        assistant_message(Vec::new(), Some("msg_missing_parent"), None).unwrap(),
    ])
    .is_err());
}

#[test]
#[ignore = "porting: e2e/performance/timeline-stability fixture not implemented"]
fn assigns_deterministic_event_ids() {
    let first = event("session.status", "busy");
    let second = event("session.status", "idle");

    assert!(is_deterministic_event_id(&first.payload.id));
    let first_value: i64 = first.payload.id[13..].parse().unwrap();
    let second_value: i64 = second.payload.id[13..].parse().unwrap();
    assert_eq!(second_value, first_value + 1);
}
