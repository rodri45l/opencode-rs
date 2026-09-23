#![allow(dead_code)]

//! Port of packages/opencode/test/session/revert-compact.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: pure revert selection/cleanup — remove messages at or after
//! the revert point in chronological order, truncate the target message's parts
//! from `partID` onward, and keep earlier messages. The reference also exercises
//! snapshot-backed file restore and unrevert against git and the live session
//! store; those need the runtime and are dropped here (noted in
//! PORT-STATUS.s2.json). Stubs are local per the fast-wave protocol.

#[derive(Debug, Clone, PartialEq, Eq)]
enum S2Error {
    NotImplemented(&'static str),
}

impl std::fmt::Display for S2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S2Error::NotImplemented(what) => write!(f, "not implemented: {what}"),
        }
    }
}

impl std::error::Error for S2Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PartKind {
    Text,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Part {
    id: String,
    kind: PartKind,
}

#[derive(Debug, Clone)]
struct Msg {
    id: String,
    created: f64,
    parts: Vec<Part>,
}

#[derive(Debug, Clone)]
struct Revert {
    message_id: String,
    part_id: Option<String>,
}

fn revert_cleanup(_msgs: &[Msg], _revert: Option<&Revert>) -> Result<Vec<Msg>, S2Error> {
    Err(S2Error::NotImplemented("SessionRevert.cleanup"))
}

fn revert_kept_ids(_msgs: &[Msg], _target_id: &str) -> Result<Vec<String>, S2Error> {
    Err(S2Error::NotImplemented("SessionRevert.revert"))
}

fn part(id: &str, kind: PartKind) -> Part {
    Part {
        id: id.to_string(),
        kind,
    }
}

fn msg(id: &str, created: f64, parts: Vec<Part>) -> Msg {
    Msg {
        id: id.to_string(),
        created,
        parts,
    }
}

fn ids(msgs: &[Msg]) -> Vec<String> {
    msgs.iter().map(|m| m.id.clone()).collect()
}

#[test]
#[ignore = "porting: SessionRevert.cleanup not implemented"]
fn cleanup_with_part_id_removes_parts_from_the_revert_point_onward() {
    let msgs = vec![msg(
        "u1",
        1.0,
        vec![
            part("p1", PartKind::Text),
            part("p2", PartKind::Tool),
            part("p3", PartKind::Text),
        ],
    )];
    let revert = Revert {
        message_id: "u1".to_string(),
        part_id: Some("p2".to_string()),
    };

    let result = revert_cleanup(&msgs, Some(&revert)).expect("cleanup");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].parts.len(), 1);
    assert_eq!(result[0].parts[0].id, "p1");
}

#[test]
#[ignore = "porting: SessionRevert.cleanup not implemented"]
fn cleanup_removes_messages_after_revert_point_but_keeps_earlier_ones() {
    let msgs = vec![
        msg("u1", 1.0, vec![part("p1", PartKind::Text)]),
        msg("a1", 2.0, vec![part("p2", PartKind::Text)]),
        msg("u2", 3.0, vec![part("p3", PartKind::Text)]),
        msg("a2", 4.0, vec![part("p4", PartKind::Text)]),
    ];
    let revert = Revert {
        message_id: "u2".to_string(),
        part_id: None,
    };

    let result = revert_cleanup(&msgs, Some(&revert)).expect("cleanup");
    assert_eq!(ids(&result), vec!["u1", "a1"]);
}

#[test]
#[ignore = "porting: SessionRevert.revert not implemented"]
fn reverts_chronological_suffixes_on_both_sides_of_mixed_message_id_ordering() {
    let msgs = vec![
        msg("msg_z9-before", 1.0, vec![part("p1", PartKind::Text)]),
        msg("msg_z1-before-wrap", 2.0, vec![part("p2", PartKind::Text)]),
        msg("msg_a0-after-wrap", 3.0, vec![part("p3", PartKind::Text)]),
        msg("msg_a1-after", 4.0, vec![part("p4", PartKind::Text)]),
    ];

    let kept_one = revert_kept_ids(&msgs, "msg_z1-before-wrap").expect("revert");
    assert_eq!(creates_from_ids(&msgs, &kept_one), vec![1.0]);

    let kept_two = revert_kept_ids(&msgs, "msg_a0-after-wrap").expect("revert");
    assert_eq!(creates_from_ids(&msgs, &kept_two), vec![1.0, 2.0]);
}

fn creates_from_ids(msgs: &[Msg], kept: &[String]) -> Vec<f64> {
    msgs.iter()
        .filter(|m| kept.contains(&m.id))
        .map(|m| m.created)
        .collect()
}

#[test]
#[ignore = "porting: SessionRevert.cleanup not implemented"]
fn cleanup_is_a_no_op_when_session_has_no_revert_state() {
    let msgs = vec![msg("u1", 1.0, vec![part("p1", PartKind::Text)])];
    let result = revert_cleanup(&msgs, None).expect("cleanup");
    assert_eq!(ids(&result), vec!["u1"]);
}
