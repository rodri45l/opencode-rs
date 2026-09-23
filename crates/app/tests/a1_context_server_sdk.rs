//! Port of packages/app/src/context/server-sdk.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
enum Payload {
    PartDelta {
        id: Option<String>,
        message_id: String,
        part_id: String,
        field: String,
        delta: String,
    },
    PartUpdated {
        id: Option<String>,
        text: String,
    },
    SessionStatus {
        kind: String,
    },
    MessageRemoved,
    MessageUpdated,
    SessionDeleted,
    PermissionAsked {
        id: String,
        session_id: String,
        permission: String,
        patterns: Vec<String>,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct Event {
    directory: String,
    payload: Payload,
}

#[derive(Clone, Debug, PartialEq)]
struct Adapted {
    event_type: String,
    id: String,
    permission: String,
    patterns: Vec<String>,
    current_id: String,
}

// Local stubs (fast wave): real module lands later.
fn resume_stream_after_page_show(_persisted: bool, _starts: &mut usize) {}

fn adapt_server_event(_event: &Event) -> Adapted {
    Adapted {
        event_type: String::new(),
        id: String::new(),
        permission: String::new(),
        patterns: Vec::new(),
        current_id: String::new(),
    }
}

fn coalesce_server_events(_events: Vec<Event>) -> Vec<Event> {
    Vec::new()
}

#[allow(clippy::ptr_arg)]
fn enqueue_server_event(_events: &mut Vec<Event>, _event: Event) {}

fn delta(value: &str, field: &str, part_id: &str) -> Event {
    Event {
        directory: "/repo".into(),
        payload: Payload::PartDelta {
            id: None,
            message_id: "msg".into(),
            part_id: part_id.into(),
            field: field.into(),
            delta: value.into(),
        },
    }
}

#[test]
#[ignore = "porting: context/server-sdk not implemented"]
fn restarts_a_stream_only_after_a_back_forward_cache_restore() {
    let mut starts = 0;
    resume_stream_after_page_show(false, &mut starts);
    resume_stream_after_page_show(true, &mut starts);
    assert_eq!(starts, 1);
}

#[test]
#[ignore = "porting: context/server-sdk not implemented"]
fn preserves_v2_events_while_adapting_permission_requests_for_existing_consumers() {
    let current = Event {
        directory: "/repo".into(),
        payload: Payload::PermissionAsked {
            id: "perm_1".into(),
            session_id: "ses_1".into(),
            permission: "read".into(),
            patterns: vec!["src/**".into()],
        },
    };
    let adapted = adapt_server_event(&current);
    assert_eq!(adapted.event_type, "permission.asked");
    assert_eq!(adapted.id, "perm_1");
    assert_eq!(adapted.permission, "read");
    assert_eq!(adapted.patterns, vec!["src/**".to_string()]);
    assert_eq!(adapted.current_id, "evt_1");
}

#[test]
#[ignore = "porting: context/server-sdk not implemented"]
fn merges_adjacent_deltas_for_the_same_field() {
    let mut first = delta("hello ", "text", "part");
    if let Payload::PartDelta { id, .. } = &mut first.payload {
        *id = Some("first".into());
    }
    let mut second = delta("world", "text", "part");
    if let Payload::PartDelta { id, .. } = &mut second.payload {
        *id = Some("second".into());
    }

    let result = coalesce_server_events(vec![first, second]);
    assert_eq!(result.len(), 1);
    assert!(matches!(
        &result[0].payload,
        Payload::PartDelta { id: Some(id), delta, .. } if id == "second" && delta == "hello world"
    ));
}

#[test]
#[ignore = "porting: context/server-sdk not implemented"]
fn preserves_event_boundaries_and_distinct_fields() {
    let status = Event {
        directory: "/repo".into(),
        payload: Payload::SessionStatus {
            kind: "idle".into(),
        },
    };
    let result = coalesce_server_events(vec![
        delta("a", "text", "part"),
        delta("b", "metadata", "part"),
        status,
        delta("c", "text", "part"),
    ]);
    let types: Vec<&str> = result
        .iter()
        .map(|event| match event.payload {
            Payload::PartDelta { .. } => "message.part.delta",
            Payload::SessionStatus { .. } => "session.status",
            _ => "other",
        })
        .collect();
    assert_eq!(
        types,
        vec![
            "message.part.delta",
            "message.part.delta",
            "session.status",
            "message.part.delta"
        ]
    );
}

#[test]
#[ignore = "porting: context/server-sdk not implemented"]
fn preserves_event_id_order_across_interleaved_deltas() {
    let mut first = delta("a", "text", "part");
    let mut other = delta("b", "text", "other");
    let mut last = delta("c", "text", "part");
    for (event, id) in [(&mut first, "1"), (&mut other, "2"), (&mut last, "3")] {
        if let Payload::PartDelta { id: slot, .. } = &mut event.payload {
            *slot = Some(id.into());
        }
    }
    let result = coalesce_server_events(vec![first, other, last]);
    let ids: Vec<Option<String>> = result
        .iter()
        .map(|event| match &event.payload {
            Payload::PartDelta { id, .. } => id.clone(),
            _ => None,
        })
        .collect();
    assert_eq!(
        ids,
        vec![
            Some("1".to_string()),
            Some("2".to_string()),
            Some("3".to_string())
        ]
    );
}

#[test]
#[ignore = "porting: context/server-sdk not implemented"]
fn preserves_part_updates_across_message_remove_and_re_add_barriers() {
    let mut events = Vec::new();
    enqueue_server_event(
        &mut events,
        Event {
            directory: "/repo".into(),
            payload: Payload::PartUpdated {
                id: None,
                text: "old".into(),
            },
        },
    );
    enqueue_server_event(
        &mut events,
        Event {
            directory: "/repo".into(),
            payload: Payload::MessageRemoved,
        },
    );
    enqueue_server_event(
        &mut events,
        Event {
            directory: "/repo".into(),
            payload: Payload::MessageUpdated,
        },
    );
    enqueue_server_event(
        &mut events,
        Event {
            directory: "/repo".into(),
            payload: Payload::PartUpdated {
                id: None,
                text: "new".into(),
            },
        },
    );
    let types: Vec<&str> = events
        .iter()
        .map(|event| match event.payload {
            Payload::PartUpdated { .. } => "message.part.updated",
            Payload::MessageRemoved => "message.removed",
            Payload::MessageUpdated => "message.updated",
            _ => "other",
        })
        .collect();
    assert_eq!(
        types,
        vec![
            "message.part.updated",
            "message.removed",
            "message.updated",
            "message.part.updated"
        ]
    );
}

#[test]
#[ignore = "porting: context/server-sdk not implemented"]
fn preserves_deltas_after_a_replacement_snapshot() {
    let mut events = Vec::new();
    enqueue_server_event(
        &mut events,
        Event {
            directory: "/repo".into(),
            payload: Payload::PartUpdated {
                id: None,
                text: "a".into(),
            },
        },
    );
    enqueue_server_event(
        &mut events,
        Event {
            directory: "/repo".into(),
            payload: Payload::PartUpdated {
                id: None,
                text: "ab".into(),
            },
        },
    );
    enqueue_server_event(&mut events, delta("c", "text", "part"));

    let result = coalesce_server_events(events);
    let types: Vec<&str> = result
        .iter()
        .map(|event| match event.payload {
            Payload::PartUpdated { .. } => "message.part.updated",
            Payload::PartDelta { .. } => "message.part.delta",
            _ => "other",
        })
        .collect();
    assert_eq!(types, vec!["message.part.updated", "message.part.delta"]);
}

#[test]
#[ignore = "porting: context/server-sdk not implemented"]
fn preserves_updates_after_session_deletion() {
    let mut events = Vec::new();
    enqueue_server_event(
        &mut events,
        Event {
            directory: "/repo".into(),
            payload: Payload::PartUpdated {
                id: None,
                text: "old".into(),
            },
        },
    );
    enqueue_server_event(
        &mut events,
        Event {
            directory: "/repo".into(),
            payload: Payload::SessionDeleted,
        },
    );
    enqueue_server_event(
        &mut events,
        Event {
            directory: "/repo".into(),
            payload: Payload::PartUpdated {
                id: None,
                text: "new".into(),
            },
        },
    );
    let types: Vec<&str> = events
        .iter()
        .map(|event| match event.payload {
            Payload::PartUpdated { .. } => "message.part.updated",
            Payload::SessionDeleted => "session.deleted",
            _ => "other",
        })
        .collect();
    assert_eq!(
        types,
        vec![
            "message.part.updated",
            "session.deleted",
            "message.part.updated"
        ]
    );
}

#[test]
#[ignore = "porting: context/server-sdk not implemented"]
fn does_not_coalesce_edge_triggered_session_statuses() {
    let mut events = Vec::new();
    for kind in ["retry", "busy"] {
        enqueue_server_event(
            &mut events,
            Event {
                directory: "/repo".into(),
                payload: Payload::SessionStatus { kind: kind.into() },
            },
        );
    }
    assert_eq!(events.len(), 2);
}
