//! Port of packages/core/test/session-runner-tool-events.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a local tool success serializes media base64 exactly once,
//! drops the compatibility `result`, and reconstructs from structured content; a
//! provider-executed success keeps `result`; a binary failure publishes a failed
//! event and no success event; legacy success data with `result` still decodes;
//! and a finished step records a settlement without publishing step ended.
//! Re-derived: the `EventV2` service capture is replaced by an explicit
//! publisher that records its own events.

use opencode_core::session_runner_tool_events::{LlmEventPublisher, ToolResultSpec};
use serde_json::json;

const NOTE: &str = "porting: session runner tool events not implemented";
const BASE64: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB";

fn media_content() -> serde_json::Value {
    json!([
        { "type": "text", "text": "Image read successfully" },
        { "type": "file", "uri": format!("data:image/png;base64,{BASE64}"), "mime": "image/png" }
    ])
}

fn publisher() -> LlmEventPublisher {
    LlmEventPublisher::new("ses_tool_event_test", "build", "model", "provider")
}

#[test]
#[ignore = "porting: session runner tool events not implemented"]
fn local_tool_success_serializes_media_once_and_reconstructs() {
    let mut publisher = publisher();
    publisher
        .publish_tool_call("call-image", "read", json!({ "path": "pixel.png" }))
        .expect(NOTE);
    publisher
        .publish_tool_result(
            "call-image",
            ToolResultSpec {
                structured: json!({ "type": "media", "mime": "image/png" }),
                content: media_content(),
                result: Some(json!({ "type": "content", "value": media_content() })),
                provider_executed: false,
            },
        )
        .expect(NOTE);

    let success = publisher
        .published()
        .into_iter()
        .find(|event| event.event_type == "session.next.tool.success.1")
        .expect(NOTE);
    let serialized = serde_json::to_string(&success.data).expect(NOTE);
    assert_eq!(serialized.matches(BASE64).count(), 1);
    assert!(success.data.get("result").is_none());
    assert_eq!(success.data["content"], media_content());
}

#[test]
#[ignore = "porting: session runner tool events not implemented"]
fn provider_executed_success_retains_its_compatibility_result() {
    let mut publisher = publisher();
    publisher
        .publish_tool_call("call-image", "read", json!({ "path": "pixel.png" }))
        .expect(NOTE);
    publisher
        .publish_tool_result(
            "call-image",
            ToolResultSpec {
                structured: json!({ "type": "media", "mime": "image/png" }),
                content: media_content(),
                result: Some(json!({ "type": "content", "value": media_content() })),
                provider_executed: true,
            },
        )
        .expect(NOTE);

    let success = publisher
        .published()
        .into_iter()
        .find(|event| event.event_type == "session.next.tool.success.1")
        .expect(NOTE);
    assert!(success.data.get("result").is_some());
}

#[test]
#[ignore = "porting: session runner tool events not implemented"]
fn binary_failure_emits_no_success_event() {
    let mut publisher = publisher();
    publisher
        .publish_tool_call("call-image", "read", json!({ "path": "pixel.png" }))
        .expect(NOTE);
    publisher
        .publish_tool_failure(
            "call-image",
            json!({ "type": "error", "value": "Cannot read binary file" }),
        )
        .expect(NOTE);

    let events = publisher.published();
    assert!(!events
        .iter()
        .any(|event| event.event_type == "session.next.tool.success.1"));
    assert!(events
        .iter()
        .any(|event| event.event_type == "session.next.tool.failed.1"));
}

#[test]
#[ignore = "porting: session runner tool events not implemented"]
fn legacy_success_data_with_result_still_decodes() {
    let decoded = LlmEventPublisher::decode_success_data(&json!({
        "callID": "call-old",
        "structured": { "type": "media", "mime": "image/png" },
        "content": [{ "type": "file", "uri": format!("data:image/png;base64,{BASE64}"), "mime": "image/png" }],
        "result": { "type": "content", "value": [{ "type": "file", "uri": format!("data:image/png;base64,{BASE64}"), "mime": "image/png" }] },
        "provider": { "executed": false }
    }))
    .expect(NOTE);
    assert_eq!(decoded["result"]["type"], json!("content"));
}

#[test]
#[ignore = "porting: session runner tool events not implemented"]
fn step_finish_records_settlement_without_publishing_step_ended() {
    let mut publisher = publisher();
    publisher.publish_step_start(0).expect(NOTE);
    publisher.publish_step_finish(0, "stop").expect(NOTE);

    assert!(!publisher
        .published()
        .iter()
        .any(|event| event.event_type == "session.next.step.ended.2"));
    assert_eq!(
        publisher.step_settlement().expect(NOTE)["finish"],
        json!("stop")
    );
}
