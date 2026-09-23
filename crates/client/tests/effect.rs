//! Port of packages/client/test/effect.test.ts (upstream 18ef3cc).
//!
//! The reference exercises the Effect-native generated client. Rust has no
//! Effect runtime, so the typed tests are re-derived against the async client
//! surface and remain red-first (`#[ignore]`) until that surface lands. The SSE
//! decoding tests need no transport and are pinned directly.

use futures::executor::block_on;
use opencode_client::{
    Admission, Client, ClientError, CreateInput, HistoryInput, ListInput, ModelRef, Prompt,
    PromptInput, Session, SessionEventsInput, SessionMessage,
};
use opencode_schema::LocationRef;

const CONNECTED_EVENT: &str = r#"{"id":"evt_connected","type":"server.connected","data":{}}"#;
const MODEL_SWITCHED_EVENT: &str = r#"{"id":"evt_model","type":"session.next.model.switched","durable":{"aggregateID":"ses_test","seq":1,"version":1},"data":{"timestamp":1717171717000,"sessionID":"ses_test","messageID":"msg_model","model":{"id":"claude","providerID":"anthropic"}}}"#;

#[test]
#[ignore = "porting: typed sessions client not implemented"]
fn sessions_get_returns_the_decoded_projection() {
    let client = Client::new("http://localhost:3000");
    let result: Session = block_on(client.sessions().get("ses_test")).expect("get");
    assert_eq!(result.time.created, 1_717_171_717_000);
}

#[test]
fn events_subscribe_exposes_and_decodes_the_native_event_stream() {
    let body = format!("data: {CONNECTED_EVENT}\n\ndata: {MODEL_SWITCHED_EVENT}\n\n");
    let events = opencode_client::decode_event_stream(&body).expect("decode");

    let types: Vec<&str> = events.iter().map(|event| event.name()).collect();
    assert_eq!(
        types,
        vec!["server.connected", "session.next.model.switched"]
    );

    let durable = events[1].durable.as_ref().expect("durable");
    assert_eq!(durable.aggregate_id, "ses_test");
    assert_eq!(durable.seq, 1);
    assert_eq!(durable.version, 1);
    assert_eq!(
        events[1].data["timestamp"].as_i64(),
        Some(1_717_171_717_000)
    );
}

#[test]
fn events_subscribe_terminates_on_protocol_decode_failures() {
    let error = opencode_client::decode_event_stream("data: {\"type\":\"server.connected\"}\n\n")
        .unwrap_err();
    assert!(matches!(error, ClientError::Protocol(_)));
}

#[test]
#[ignore = "porting: typed sessions client not implemented"]
fn session_methods_retain_decoded_inputs_and_outputs() {
    let client = Client::new("http://localhost:3000");
    let sessions = client.sessions();

    let page = block_on(sessions.list(ListInput {
        limit: Some(10),
        order: None,
    }))
    .expect("list");
    assert_eq!(page.data[0].time.created, 1_717_171_717_000);

    let active = block_on(sessions.active()).expect("active");
    assert_eq!(active["ses_test"].kind, "running");

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
    assert_eq!(admitted.time_created, 1_717_171_717_000);

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
    assert_eq!(
        history.data[0].data["timestamp"].as_i64(),
        Some(1_717_171_717_000)
    );
    assert!(history.has_more);

    let events = block_on(futures::StreamExt::collect::<Vec<_>>(sessions.events(
        SessionEventsInput {
            session_id: "ses_test".into(),
            after: Some(0),
        },
    )));
    assert_eq!(events.len(), 1);

    block_on(sessions.interrupt("ses_test")).expect("interrupt");
    let message: SessionMessage =
        block_on(sessions.message("ses_test", "msg_model")).expect("message");
    assert_eq!(message.id, "msg_model");
    assert_eq!(message.kind, "model-switched");
}

#[test]
#[ignore = "porting: typed sessions client not implemented"]
fn sessions_history_retains_the_typed_session_not_found_error() {
    let client = Client::new("http://localhost:3000");
    let error = block_on(client.sessions().history(HistoryInput {
        session_id: "ses_missing".into(),
        after: None,
        limit: None,
    }))
    .unwrap_err();
    assert!(opencode_client::is_session_not_found_error(&error));
}
