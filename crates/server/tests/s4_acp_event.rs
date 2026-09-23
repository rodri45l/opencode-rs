//! Port of packages/opencode/test/acp/event.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/acp/event.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure classification rules the subscription applies to live
//! events — delta field/part-type to ACP session-update kind, tool state to ACP
//! status, reasoning part-id boundaries, and the rule that live user-role deltas
//! are not replayed as `user_message_chunk`.
//! Dropped: every case that drives the `ACPEvent.Subscription` through an Effect
//! runtime and a fake event stream (routing isolation, metadata fetch counts,
//! shell-snapshot dedupe, replay ordering/failure handling, synthetic pending
//! tool calls, attachments). Those need the live subscription/connection; the
//! projections they rely on are covered by `s4_acp_tool.rs`.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn delta_update_kind(_field: &str, _part_type: &str) -> Result<String, NotImplemented> {
    nope("acp event")
}

fn tool_update_status(_state_status: &str) -> Result<String, NotImplemented> {
    nope("acp event")
}

fn thought_message_id(_part_id: &str) -> Result<String, NotImplemented> {
    nope("acp event")
}

fn should_emit_live_delta(_role: &str) -> Result<bool, NotImplemented> {
    nope("acp event")
}

#[test]
#[ignore = "porting: acp event not implemented"]
fn maps_text_and_reasoning_deltas_to_acp_update_kinds() {
    assert_eq!(
        delta_update_kind("text", "text").unwrap(),
        "agent_message_chunk"
    );
    assert_eq!(
        delta_update_kind("text", "reasoning").unwrap(),
        "agent_thought_chunk"
    );
}

#[test]
#[ignore = "porting: acp event not implemented"]
fn maps_tool_state_to_acp_status() {
    assert_eq!(tool_update_status("pending").unwrap(), "pending");
    assert_eq!(tool_update_status("running").unwrap(), "in_progress");
    assert_eq!(tool_update_status("completed").unwrap(), "completed");
    assert_eq!(tool_update_status("error").unwrap(), "failed");
}

#[test]
#[ignore = "porting: acp event not implemented"]
fn uses_reasoning_part_ids_as_thought_message_boundaries() {
    assert_eq!(thought_message_id("part_first").unwrap(), "part_first");
    assert_eq!(thought_message_id("part_second").unwrap(), "part_second");
}

#[test]
#[ignore = "porting: acp event not implemented"]
fn ignores_live_user_parts_to_avoid_user_message_chunk_duplication() {
    assert!(!should_emit_live_delta("user").unwrap());
    assert!(should_emit_live_delta("assistant").unwrap());
}
