//! Port of packages/client/test/promise.test.ts (upstream 18ef3cc).
//!
//! The reference exercises the generated Promise client against an injected
//! `fetch`. This crate has no transport seam yet, so the request/response tests
//! are red-first (`#[ignore]`) and fail with `ClientError::NotImplemented` when
//! the ignore is removed. SSE wire parsing needs no transport and is pinned
//! directly against [`opencode_client::decode_event_stream`].

mod common;

use std::collections::HashMap;

use common::FakeTransport;
use futures::executor::block_on;
use opencode_client::{
    is_session_not_found_error, is_unauthorized_error, Admission, Client, ClientError, CreateInput,
    HistoryInput, ListInput, ModelRef, Prompt, PromptInput, Session, SessionEventsInput,
};
use opencode_schema::LocationRef;

const CONNECTED_EVENT: &str = r#"{"id":"evt_connected","type":"server.connected","data":{}}"#;
const MODEL_SWITCHED_EVENT: &str = r#"{"id":"evt_model","type":"session.next.model.switched","durable":{"aggregateID":"ses_test","seq":1,"version":1},"data":{"timestamp":1717171717000,"sessionID":"ses_test","messageID":"msg_model","model":{"id":"claude","providerID":"anthropic"}}}"#;

#[test]
fn exposes_every_standard_http_api_group() {
    let client = Client::new("http://localhost:3000");
    let groups = client.groups().expect("groups");
    let names: Vec<&str> = groups.iter().map(|group| group.name).collect();

    assert_eq!(
        names,
        vec![
            "health",
            "location",
            "agents",
            "sessions",
            "messages",
            "models",
            "providers",
            "integrations",
            "credentials",
            "permissions",
            "files",
            "commands",
            "skills",
            "events",
            "ptys",
            "questions",
            "references",
            "projectCopies",
        ]
    );

    let operations = |name: &str| {
        groups
            .iter()
            .find(|group| group.name == name)
            .map(|group| group.operations.to_vec())
            .unwrap_or_default()
    };
    assert_eq!(operations("messages"), vec!["list"]);
    assert_eq!(
        operations("integrations"),
        vec![
            "list",
            "get",
            "connectKey",
            "connectOauth",
            "attemptStatus",
            "attemptComplete",
            "attemptCancel",
        ]
    );
    assert_eq!(operations("files"), vec!["list", "find"]);
    assert_eq!(
        operations("ptys"),
        vec!["list", "create", "get", "update", "remove"]
    );
}

#[test]
fn sessions_get_returns_the_wire_projection() {
    let transport = FakeTransport::new(vec![(
        "GET",
        "http://localhost:3000/api/session/ses_test",
        200,
        common::SESSION_JSON,
    )]);
    let client = Client::with_transport("http://localhost:3000", transport);
    let result: Session = block_on(client.sessions().get("ses_test")).expect("sessions.get");
    assert_eq!(result.time.created, 1_717_171_717_000);
}

#[test]
fn events_subscribe_exposes_the_promise_event_stream_wire_projection() {
    let body =
        format!(": heartbeat\n\ndata: {CONNECTED_EVENT}\n\ndata: {MODEL_SWITCHED_EVENT}\n\n");
    let events = opencode_client::decode_event_stream(&body).expect("decode");

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].name(), "server.connected");
    assert_eq!(events[1].name(), "session.next.model.switched");
    assert_eq!(
        events[1].data["timestamp"].as_i64(),
        Some(1_717_171_717_000)
    );
}

#[test]
fn events_subscribe_terminates_on_malformed_promise_sse_data() {
    let error = opencode_client::decode_event_stream("data: {not-json}\n\n").unwrap_err();
    assert!(matches!(error, ClientError::Protocol(_)));
}

#[test]
fn session_methods_use_the_public_http_contract() {
    let transport = FakeTransport::new(common::standard_routes());
    let client = Client::with_transport("http://localhost:3000", transport.clone());
    let sessions = client.sessions();

    let expected: Vec<(&str, &str)> = vec![
        (
            "GET",
            "http://localhost:3000/api/session?limit=10&order=desc",
        ),
        ("GET", "http://localhost:3000/api/session/active"),
        ("POST", "http://localhost:3000/api/session"),
        ("POST", "http://localhost:3000/api/session/ses_test/agent"),
        ("POST", "http://localhost:3000/api/session/ses_test/model"),
        ("POST", "http://localhost:3000/api/session/ses_test/prompt"),
        ("POST", "http://localhost:3000/api/session/ses_test/compact"),
        ("POST", "http://localhost:3000/api/session/ses_test/wait"),
        ("GET", "http://localhost:3000/api/session/ses_test/context"),
        (
            "GET",
            "http://localhost:3000/api/session/ses_test/history?limit=1&after=0",
        ),
        (
            "GET",
            "http://localhost:3000/api/session/ses_test/history?limit=2&after=1",
        ),
        (
            "GET",
            "http://localhost:3000/api/session/ses_test/event?after=0",
        ),
        (
            "POST",
            "http://localhost:3000/api/session/ses_test/interrupt",
        ),
        (
            "GET",
            "http://localhost:3000/api/session/ses_test/message/msg_model",
        ),
    ];

    let page = block_on(sessions.list(ListInput {
        limit: Some(10),
        order: Some("desc".into()),
    }))
    .expect("list");
    assert_eq!(page.cursor.next, "next");

    let active = block_on(sessions.active()).expect("active");
    assert_eq!(active, HashMap::from([("ses_test".to_string(), running())]));

    let created = block_on(sessions.create(CreateInput {
        location: LocationRef {
            directory: "/tmp/project".into(),
            workspace_id: None,
        },
    }))
    .expect("create");
    assert_eq!(created.id, "ses_test");

    block_on(sessions.switch_agent("ses_test", "build")).expect("switchAgent");
    block_on(sessions.switch_model(
        "ses_test",
        ModelRef {
            id: "claude".into(),
            provider_id: "anthropic".into(),
        },
    ))
    .expect("switchModel");

    let admitted: Admission = block_on(sessions.prompt(PromptInput {
        session_id: "ses_test".into(),
        prompt: Prompt {
            text: "Hello".into(),
        },
        resume: false,
    }))
    .expect("prompt");
    assert_eq!(admitted.id, "msg_test");

    block_on(sessions.compact("ses_test")).expect("compact");
    block_on(sessions.wait("ses_test")).expect("wait");

    let context = block_on(sessions.context("ses_test")).expect("context");
    assert!(context.is_empty());

    let history = block_on(sessions.history(HistoryInput {
        session_id: "ses_test".into(),
        after: Some(0),
        limit: Some(1),
    }))
    .expect("history");
    assert_eq!(history.data.len(), 1);

    let history_next = block_on(sessions.history(HistoryInput {
        session_id: "ses_test".into(),
        after: Some(1),
        limit: Some(2),
    }))
    .expect("history");
    assert!(!history_next.has_more);

    let events = block_on(futures::StreamExt::collect::<Vec<_>>(sessions.events(
        SessionEventsInput {
            session_id: "ses_test".into(),
            after: Some(0),
        },
    )));
    assert_eq!(events.len(), 1);

    block_on(sessions.interrupt("ses_test")).expect("interrupt");
    let message = block_on(sessions.message("ses_test", "msg_model")).expect("message");
    assert_eq!(message.id, "msg_model");

    let recorded: Vec<(String, String)> = transport.recorded();
    let expected: Vec<(String, String)> = expected
        .into_iter()
        .map(|(method, url)| (method.to_string(), url.to_string()))
        .collect();
    assert_eq!(recorded, expected);
}

#[test]
fn middleware_errors_remain_declared_client_errors() {
    let transport = FakeTransport::new(vec![(
        "POST",
        "http://localhost:3000/api/session",
        401,
        r#"{"_tag":"UnauthorizedError","message":"missing credentials"}"#,
    )]);
    let client = Client::with_transport("http://localhost:3000", transport);
    let error = block_on(client.sessions().create(CreateInput {
        location: LocationRef {
            directory: "/tmp/project".into(),
            workspace_id: None,
        },
    }))
    .unwrap_err();
    assert!(is_unauthorized_error(&error));
}

#[test]
fn sessions_history_decodes_session_not_found_error() {
    let transport = FakeTransport::new(vec![(
        "GET",
        "http://localhost:3000/api/session/ses_missing/history",
        404,
        r#"{"_tag":"SessionNotFoundError","sessionID":"ses_missing","message":"not found"}"#,
    )]);
    let client = Client::with_transport("http://localhost:3000", transport);
    let error = block_on(client.sessions().history(HistoryInput {
        session_id: "ses_missing".into(),
        after: None,
        limit: None,
    }))
    .unwrap_err();
    assert!(is_session_not_found_error(&error));
}

fn running() -> opencode_client::ActiveStatus {
    opencode_client::ActiveStatus {
        kind: "running".into(),
    }
}
