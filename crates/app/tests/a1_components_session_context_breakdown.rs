//! Port of packages/app/src/components/session/session-context-breakdown.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
struct Segment {
    key: String,
    tokens: i64,
    width: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct Message {
    id: String,
    role: String,
}

#[derive(Clone, Debug, PartialEq)]
struct TextPart {
    text: String,
}

// Local stub (fast wave): real module lands later.
fn estimate_session_context_breakdown(
    _messages: &[Message],
    _parts: &BTreeMap<String, Vec<TextPart>>,
    _input: i64,
    _system_prompt: &str,
) -> Vec<Segment> {
    Vec::new()
}

fn user(id: &str) -> Message {
    Message {
        id: id.into(),
        role: "user".into(),
    }
}

fn assistant(id: &str) -> Message {
    Message {
        id: id.into(),
        role: "assistant".into(),
    }
}

#[test]
#[ignore = "porting: components/session-context-breakdown not implemented"]
fn estimates_tokens_and_keeps_remaining_tokens_as_other() {
    let messages = vec![user("u1"), assistant("a1")];
    let mut parts = BTreeMap::new();
    parts.insert(
        "u1".to_string(),
        vec![TextPart {
            text: "hello world".into(),
        }],
    );
    parts.insert(
        "a1".to_string(),
        vec![TextPart {
            text: "assistant response".into(),
        }],
    );

    let output = estimate_session_context_breakdown(&messages, &parts, 20, "system prompt");
    let map: BTreeMap<String, i64> = output.iter().map(|s| (s.key.clone(), s.tokens)).collect();
    assert_eq!(map.get("system"), Some(&4));
    assert_eq!(map.get("user"), Some(&3));
    assert_eq!(map.get("assistant"), Some(&5));
    assert_eq!(map.get("other"), Some(&8));
}

#[test]
#[ignore = "porting: components/session-context-breakdown not implemented"]
fn scales_segments_when_estimates_exceed_input() {
    let messages = vec![user("u1"), assistant("a1")];
    let mut parts = BTreeMap::new();
    parts.insert(
        "u1".to_string(),
        vec![TextPart {
            text: "x".repeat(400),
        }],
    );
    parts.insert(
        "a1".to_string(),
        vec![TextPart {
            text: "y".repeat(400),
        }],
    );

    let output = estimate_session_context_breakdown(&messages, &parts, 10, &"z".repeat(200));
    let total: i64 = output.iter().map(|s| s.tokens).sum();
    assert!(total <= 10);
    assert!(output.iter().all(|s| s.width <= 100.0));
}
