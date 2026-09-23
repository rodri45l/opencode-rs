//! Port of packages/app/src/context/comments.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::comments::{CommentSession, Focus, LineComment};

fn line(file: &str, id: &str, time: i64) -> LineComment {
    LineComment {
        id: id.into(),
        file: file.into(),
        comment: id.into(),
        time,
    }
}

fn session(files: &[(&str, Vec<LineComment>)]) -> CommentSession {
    let mut s = CommentSession::default();
    for (file, comments) in files {
        s.files.insert((*file).to_string(), comments.clone());
    }
    s
}

#[test]
fn keeps_file_list_behavior_and_aggregate_chronological_order() {
    let mut comments = session(&[
        (
            "a.ts",
            vec![
                line("a.ts", "a-late", 20_000),
                line("a.ts", "a-early", 1_000),
            ],
        ),
        ("b.ts", vec![line("b.ts", "b-mid", 10_000)]),
    ]);
    assert_eq!(
        comments
            .list("a.ts")
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>(),
        vec!["a-late".to_string(), "a-early".to_string()]
    );
    assert_eq!(
        comments
            .all()
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>(),
        vec![
            "a-early".to_string(),
            "b-mid".to_string(),
            "a-late".to_string()
        ]
    );
    let next = comments.add("b.ts", "next");
    assert_eq!(
        comments.list("b.ts").last().map(|c| c.id.clone()),
        Some(next.id)
    );
    let times: Vec<i64> = comments.all().iter().map(|c| c.time).collect();
    let mut sorted = times.clone();
    sorted.sort();
    assert_eq!(times, sorted);
}

#[test]
fn remove_updates_file_and_aggregate_indexes_consistently() {
    let mut comments = session(&[
        (
            "a.ts",
            vec![line("a.ts", "a1", 10), line("a.ts", "shared", 20)],
        ),
        ("b.ts", vec![line("b.ts", "shared", 30)]),
    ]);
    comments.set_focus(Some(Focus {
        file: "a.ts".into(),
        id: "shared".into(),
    }));
    comments.set_active(Some(Focus {
        file: "a.ts".into(),
        id: "shared".into(),
    }));
    comments.remove("a.ts", "shared");
    assert_eq!(
        comments
            .list("a.ts")
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>(),
        vec!["a1".to_string()]
    );
    assert_eq!(
        comments
            .all()
            .iter()
            .filter(|c| c.id == "shared")
            .map(|c| c.file.clone())
            .collect::<Vec<_>>(),
        vec!["b.ts".to_string()]
    );
    assert_eq!(comments.focus(), None);
    assert_eq!(
        comments.active(),
        Some(Focus {
            file: "a.ts".into(),
            id: "shared".into()
        })
    );
}

#[test]
fn clear_resets_file_and_aggregate_indexes_plus_focus_state() {
    let mut comments = session(&[("a.ts", vec![line("a.ts", "a1", 10)])]);
    let next = comments.add("b.ts", "next");
    comments.set_active(Some(Focus {
        file: "b.ts".into(),
        id: next.id,
    }));
    comments.clear();
    assert!(comments.list("a.ts").is_empty());
    assert!(comments.list("b.ts").is_empty());
    assert!(comments.all().is_empty());
    assert_eq!(comments.focus(), None);
    assert_eq!(comments.active(), None);
}

#[test]
fn remove_keeps_focus_when_same_comment_id_exists_in_another_file() {
    let mut comments = session(&[
        ("a.ts", vec![line("a.ts", "shared", 10)]),
        ("b.ts", vec![line("b.ts", "shared", 20)]),
    ]);
    comments.set_focus(Some(Focus {
        file: "b.ts".into(),
        id: "shared".into(),
    }));
    comments.remove("a.ts", "shared");
    assert_eq!(
        comments.focus(),
        Some(Focus {
            file: "b.ts".into(),
            id: "shared".into()
        })
    );
    assert!(comments.list("a.ts").is_empty());
    assert_eq!(
        comments
            .list("b.ts")
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>(),
        vec!["shared".to_string()]
    );
}

#[test]
fn set_focus_and_set_active_updater_callbacks_receive_current_state() {
    let mut comments = CommentSession::default();
    comments.set_focus(Some(Focus {
        file: "a.ts".into(),
        id: "a1".into(),
    }));
    comments.set_focus(Some(Focus {
        file: "b.ts".into(),
        id: "b1".into(),
    }));
    comments.set_active(Some(Focus {
        file: "c.ts".into(),
        id: "c1".into(),
    }));
    comments.set_active(None);
    assert_eq!(
        comments.focus(),
        Some(Focus {
            file: "b.ts".into(),
            id: "b1".into()
        })
    );
    assert_eq!(comments.active(), None);
}

#[test]
fn update_changes_only_the_targeted_comment_body() {
    let mut comments = session(&[("a.ts", vec![line("a.ts", "a1", 10), line("a.ts", "a2", 20)])]);
    comments.update("a.ts", "a2", "edited");
    assert_eq!(
        comments
            .list("a.ts")
            .iter()
            .map(|c| c.comment.clone())
            .collect::<Vec<_>>(),
        vec!["a1".to_string(), "edited".to_string()]
    );
}

#[test]
fn replace_swaps_comment_state_and_clears_focus_state() {
    let mut comments = session(&[("a.ts", vec![line("a.ts", "a1", 10)])]);
    comments.set_focus(Some(Focus {
        file: "a.ts".into(),
        id: "a1".into(),
    }));
    comments.set_active(Some(Focus {
        file: "a.ts".into(),
        id: "a1".into(),
    }));
    comments.replace(vec![line("b.ts", "b1", 30)]);
    assert!(comments.list("a.ts").is_empty());
    assert_eq!(
        comments
            .list("b.ts")
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>(),
        vec!["b1".to_string()]
    );
    assert_eq!(comments.focus(), None);
    assert_eq!(comments.active(), None);
}
