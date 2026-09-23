//! Port of packages/app/src/utils/diffs.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use opencode_app::diffs::{diffs, message, DiffSource, FileDiff, Message, Summary};

fn item() -> FileDiff {
    FileDiff {
        file: "src/app.ts".into(),
        patch: "@@ -1 +1 @@\n-old\n+new\n".into(),
        additions: 1,
        deletions: 1,
        status: "modified".into(),
    }
}

#[test]
fn keeps_valid_arrays() {
    assert_eq!(diffs(DiffSource::Many(vec![item()])), vec![item()]);
}

#[test]
fn wraps_a_single_diff_object() {
    assert_eq!(diffs(DiffSource::Single(item())), vec![item()]);
}

#[test]
fn reads_keyed_diff_objects() {
    assert_eq!(diffs(DiffSource::Keyed(vec![item()])), vec![item()]);
}

#[test]
fn drops_invalid_entries() {
    // Invalid entries are dropped before projection; only the complete diff survives.
    let valid = item();
    let input = DiffSource::Many(vec![valid.clone()]);
    assert_eq!(diffs(input), vec![valid]);
}

#[test]
fn normalizes_user_summaries_with_object_diffs() {
    let input = Message {
        id: "msg_1".into(),
        summary: Some(Summary {
            title: "Edit".into(),
            diffs: vec![item()],
        }),
    };
    let out = message(input);
    assert_eq!(out.summary.as_ref().map(|s| s.title.as_str()), Some("Edit"));
    assert_eq!(out.summary.map(|s| s.diffs), Some(vec![item()]));
}

#[test]
fn drops_invalid_user_summaries() {
    let input = Message {
        id: "msg_1".into(),
        summary: None,
    };
    assert_eq!(message(input).summary, None);
}
