//! Port of packages/app/src/pages/session/timeline/projection.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use opencode_app::timeline_projection::{reuse_timeline_rows, row_key, TimelineRow};

fn context(key: &str, part_ids: &[&str], user_message_id: &str) -> TimelineRow {
    TimelineRow::AssistantPart {
        user_message_id: user_message_id.to_string(),
        group_key: key.to_string(),
        part_ids: part_ids.iter().map(|s| (*s).to_string()).collect(),
    }
}

fn user(user_message_id: &str) -> TimelineRow {
    TimelineRow::UserMessage {
        user_message_id: user_message_id.to_string(),
    }
}

struct Case {
    name: &'static str,
    previous: Vec<TimelineRow>,
    rows: Vec<TimelineRow>,
    expected: Vec<String>,
    reused: Vec<(usize, usize)>,
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            name: "reuses an unchanged context group",
            previous: vec![context("context:a", &["a", "b"], "user-1")],
            rows: vec![context("context:a", &["a", "b"], "user-1")],
            expected: vec!["assistant-part:user-1:context:a".into()],
            reused: vec![(0, 0)],
        },
        Case {
            name: "preserves the group key when a member is appended",
            previous: vec![context("context:a", &["a"], "user-1")],
            rows: vec![context("context:a", &["a", "b"], "user-1")],
            expected: vec!["assistant-part:user-1:context:a".into()],
            reused: vec![],
        },
        Case {
            name: "preserves the group key when the first member is removed",
            previous: vec![context("context:a", &["a", "b"], "user-1")],
            rows: vec![context("context:b", &["b"], "user-1")],
            expected: vec!["assistant-part:user-1:context:a".into()],
            reused: vec![],
        },
        Case {
            name: "lets only the natural owner retain an old key after a split",
            previous: vec![context("context:a", &["a", "b"], "user-1")],
            rows: vec![
                context("context:a", &["a"], "user-1"),
                context("context:b", &["b"], "user-1"),
            ],
            expected: vec![
                "assistant-part:user-1:context:a".into(),
                "assistant-part:user-1:context:b".into(),
            ],
            reused: vec![],
        },
        Case {
            name: "chooses the earliest prior key when groups merge",
            previous: vec![
                context("context:a", &["a"], "user-1"),
                context("context:b", &["b"], "user-1"),
            ],
            rows: vec![context("context:b", &["b", "a"], "user-1")],
            expected: vec!["assistant-part:user-1:context:a".into()],
            reused: vec![],
        },
        Case {
            name: "reserves an old key for its natural owner when two new groups compete",
            previous: vec![context("context:a", &["a", "b"], "user-1")],
            rows: vec![
                context("context:b", &["b"], "user-1"),
                context("context:a", &["a"], "user-1"),
            ],
            expected: vec![
                "assistant-part:user-1:context:b".into(),
                "assistant-part:user-1:context:a".into(),
            ],
            reused: vec![],
        },
        Case {
            name: "does not reuse context identity across user messages",
            previous: vec![context("context:a", &["a", "b"], "user-1")],
            rows: vec![context("context:b", &["b"], "user-2")],
            expected: vec!["assistant-part:user-2:context:b".into()],
            reused: vec![],
        },
        Case {
            name: "reuses an unaffected ordinary row",
            previous: vec![user("user-1")],
            rows: vec![user("user-1")],
            expected: vec!["user-message:user-1".into()],
            reused: vec![(0, 0)],
        },
        Case {
            name: "does not create accidental key collisions",
            previous: vec![context("context:a", &["a", "b", "c"], "user-1")],
            rows: vec![
                context("context:b", &["b"], "user-1"),
                context("context:a", &["a"], "user-1"),
                context("context:c", &["c"], "user-1"),
            ],
            expected: vec![
                "assistant-part:user-1:context:b".into(),
                "assistant-part:user-1:context:a".into(),
                "assistant-part:user-1:context:c".into(),
            ],
            reused: vec![],
        },
    ]
}

#[test]
fn reuse_timeline_rows_matches_the_reference_table() {
    for case in cases() {
        let result = reuse_timeline_rows(case.previous.clone(), case.rows.clone());
        assert_eq!(
            result.iter().map(row_key).collect::<Vec<_>>(),
            case.expected,
            "case: {}",
            case.name
        );
        let unique: std::collections::BTreeSet<String> = result.iter().map(row_key).collect();
        assert_eq!(
            unique.len(),
            result.len(),
            "case: {} keys must be unique",
            case.name
        );
        for (result_index, previous_index) in case.reused {
            assert_eq!(
                result[result_index], case.previous[previous_index],
                "case: {}",
                case.name
            );
        }
    }
}
