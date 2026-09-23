//! Port of packages/app/src/context/global-sync/session-trim.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use std::collections::BTreeSet;

use opencode_app::session_trim::{trim_sessions, Session, TrimOptions};

fn session(
    id: &str,
    parent_id: Option<&str>,
    created: i64,
    updated: Option<i64>,
    archived: Option<i64>,
) -> Session {
    Session {
        id: id.into(),
        parent_id: parent_id.map(str::to_string),
        created,
        updated,
        archived,
    }
}

#[test]
fn keeps_base_roots_and_recent_roots_beyond_the_limit() {
    let now = 1_000_000;
    let list = vec![
        session("a", None, now - 100_000, None, None),
        session("b", None, now - 90_000, None, None),
        session("c", None, now - 80_000, None, None),
        session("d", None, now - 70_000, Some(now - 1_000), None),
        session("e", None, now - 60_000, None, Some(now - 10)),
    ];
    let result = trim_sessions(
        &list,
        &TrimOptions {
            limit: 2,
            permission: BTreeSet::new(),
            now,
        },
    );
    assert_eq!(
        result.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
        vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string()
        ]
    );
}

#[test]
fn keeps_children_when_root_is_kept_permission_exists_or_child_is_recent() {
    let now = 1_000_000;
    let list = vec![
        session("root-1", None, now - 1000, None, None),
        session("root-2", None, now - 2000, None, None),
        session("z-root", None, now - 30_000_000, None, None),
        session(
            "child-kept-by-root",
            Some("root-1"),
            now - 20_000_000,
            None,
            None,
        ),
        session(
            "child-kept-by-permission",
            Some("z-root"),
            now - 20_000_000,
            None,
            None,
        ),
        session(
            "child-kept-by-recency",
            Some("z-root"),
            now - 500,
            None,
            None,
        ),
        session(
            "child-trimmed",
            Some("z-root"),
            now - 20_000_000,
            None,
            None,
        ),
    ];
    let mut permission = BTreeSet::new();
    permission.insert("child-kept-by-permission".to_string());
    let result = trim_sessions(
        &list,
        &TrimOptions {
            limit: 2,
            permission,
            now,
        },
    );
    assert_eq!(
        result.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
        vec![
            "child-kept-by-permission".to_string(),
            "child-kept-by-recency".to_string(),
            "child-kept-by-root".to_string(),
            "root-1".to_string(),
            "root-2".to_string(),
        ]
    );
}
