//! Port of packages/app/src/context/global-sync/session-cache.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq)]
struct Message {
    id: String,
    session_id: String,
}

#[derive(Clone, Debug, PartialEq)]
struct Part {
    id: String,
    session_id: String,
    message_id: String,
}

#[derive(Default, Clone, Debug, PartialEq)]
struct Store {
    session_status: BTreeMap<String, String>,
    session_diff: BTreeMap<String, Vec<String>>,
    todo: BTreeMap<String, Vec<String>>,
    message: BTreeMap<String, Vec<Message>>,
    session_message: BTreeMap<String, Vec<String>>,
    part: BTreeMap<String, Vec<Part>>,
    permission: BTreeMap<String, Vec<String>>,
    question: BTreeMap<String, Vec<String>>,
    part_text_accum_delta: BTreeMap<String, String>,
}

// Local stub (fast wave): real module lands later.
fn drop_session_caches(_store: &mut Store, _session_ids: &[&str]) {}

fn pick_session_cache_evictions(
    _seen: &mut BTreeSet<String>,
    _keep: &str,
    _limit: usize,
    _preserve: &[&str],
) -> Vec<String> {
    Vec::new()
}

fn msg(id: &str, session_id: &str) -> Message {
    Message {
        id: id.into(),
        session_id: session_id.into(),
    }
}

fn part(id: &str, session_id: &str, message_id: &str) -> Part {
    Part {
        id: id.into(),
        session_id: session_id.into(),
        message_id: message_id.into(),
    }
}

#[test]
#[ignore = "porting: context/global-sync/session-cache not implemented"]
fn drop_session_caches_clears_orphaned_parts_without_message_rows() {
    let mut store = Store::default();
    store.session_status.insert("ses_1".into(), "busy".into());
    store.session_diff.insert("ses_1".into(), Vec::new());
    store.todo.insert("ses_1".into(), Vec::new());
    store
        .part
        .insert("msg_1".into(), vec![part("prt_1", "ses_1", "msg_1")]);
    store.permission.insert("ses_1".into(), Vec::new());
    store.question.insert("ses_1".into(), Vec::new());
    store
        .part_text_accum_delta
        .insert("prt_1".into(), "streamed text".into());

    drop_session_caches(&mut store, &["ses_1"]);

    assert!(!store.message.contains_key("ses_1"));
    assert!(!store.part.contains_key("msg_1"));
    assert!(!store.part_text_accum_delta.contains_key("prt_1"));
    assert!(!store.todo.contains_key("ses_1"));
    assert!(!store.session_diff.contains_key("ses_1"));
    assert!(!store.session_status.contains_key("ses_1"));
    assert!(!store.permission.contains_key("ses_1"));
    assert!(!store.question.contains_key("ses_1"));
}

#[test]
#[ignore = "porting: context/global-sync/session-cache not implemented"]
fn drop_session_caches_clears_message_backed_parts() {
    let mut store = Store::default();
    let m = msg("msg_1", "ses_1");
    store.message.insert("ses_1".into(), vec![m.clone()]);
    store
        .part
        .insert(m.id.clone(), vec![part("prt_1", "ses_1", &m.id)]);

    drop_session_caches(&mut store, &["ses_1"]);

    assert!(!store.message.contains_key("ses_1"));
    assert!(!store.part.contains_key(&m.id));
}

#[test]
#[ignore = "porting: context/global-sync/session-cache not implemented"]
fn pick_session_cache_evictions_preserves_requested_sessions() {
    let mut seen: BTreeSet<String> = ["ses_1", "ses_2", "ses_3"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let stale = pick_session_cache_evictions(&mut seen, "ses_4", 2, &["ses_1"]);
    assert_eq!(stale, vec!["ses_2".to_string(), "ses_3".to_string()]);
    assert_eq!(
        seen.iter().cloned().collect::<Vec<_>>(),
        vec!["ses_1".to_string(), "ses_4".to_string()]
    );
}
