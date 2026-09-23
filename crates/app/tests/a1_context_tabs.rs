//! Port of packages/app/src/context/tabs.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use opencode_app::tabs::{
    migrate_tabs, next_tab_after_close, push_closed_tab, remove_closed_tabs, take_closed_tab,
    ClosedTab, RawTab, Tab,
};

const SERVER: &str = "local\nhttp://localhost:4096";

fn session_tab(session_id: &str) -> Tab {
    Tab::Session {
        server: SERVER.into(),
        session_id: session_id.into(),
    }
}

#[test]
fn drops_null_and_malformed_persisted_tabs() {
    let input = vec![
        None,
        Some(RawTab::Session {
            server: Some(SERVER.into()),
            session_id: "a".into(),
            dir_base64: None,
        }),
        Some(RawTab::Session {
            server: None,
            session_id: String::new(),
            dir_base64: None,
        }),
        Some(RawTab::Unknown),
    ];
    assert_eq!(migrate_tabs(Some(input), SERVER), vec![session_tab("a")]);
}

#[test]
fn adds_the_fallback_server_to_valid_legacy_tabs() {
    let input = vec![Some(RawTab::Session {
        server: None,
        session_id: "a".into(),
        dir_base64: Some("legacy".into()),
    })];
    assert_eq!(migrate_tabs(Some(input), SERVER), vec![session_tab("a")]);
}

#[test]
fn replaces_invalid_top_level_persisted_data() {
    assert_eq!(migrate_tabs(None, SERVER), Vec::<Tab>::new());
    assert_eq!(migrate_tabs(Some(Vec::new()), SERVER), Vec::<Tab>::new());
}

#[test]
fn records_session_tabs_with_their_index() {
    let stack = push_closed_tab(Vec::new(), session_tab("a"), 2);
    assert_eq!(
        stack,
        vec![ClosedTab {
            tab: session_tab("a"),
            index: 2
        }]
    );
}

#[test]
fn ignores_draft_tabs() {
    let draft = Tab::Draft {
        draft_id: "d1".into(),
        server: SERVER.into(),
        directory: "/tmp".into(),
    };
    assert_eq!(
        push_closed_tab(Vec::new(), draft, 0),
        Vec::<ClosedTab>::new()
    );
}

#[test]
fn caps_the_stack_size() {
    let mut stack = Vec::new();
    for i in 0..30 {
        stack = push_closed_tab(stack, session_tab(&format!("s{i}")), i);
    }
    assert_eq!(stack.len(), 25);
    assert_eq!(
        stack.first().map(|e| match &e.tab {
            Tab::Session { session_id, .. } => session_id.clone(),
            _ => String::new(),
        }),
        Some("s5".to_string())
    );
    assert_eq!(
        stack.last().map(|e| match &e.tab {
            Tab::Session { session_id, .. } => session_id.clone(),
            _ => String::new(),
        }),
        Some("s29".to_string())
    );
}

#[test]
fn pops_the_most_recently_closed_tab() {
    let stack = vec![
        ClosedTab {
            tab: session_tab("a"),
            index: 0,
        },
        ClosedTab {
            tab: session_tab("b"),
            index: 1,
        },
    ];
    let result = take_closed_tab(stack, &[]);
    assert_eq!(result.entry.map(|e| e.tab), Some(session_tab("b")));
    assert_eq!(
        result.stack,
        vec![ClosedTab {
            tab: session_tab("a"),
            index: 0
        }]
    );
}

#[test]
fn skips_entries_whose_tab_is_already_open() {
    let stack = vec![
        ClosedTab {
            tab: session_tab("a"),
            index: 0,
        },
        ClosedTab {
            tab: session_tab("b"),
            index: 1,
        },
    ];
    let result = take_closed_tab(stack, &[session_tab("b")]);
    assert_eq!(result.entry.map(|e| e.tab), Some(session_tab("a")));
    assert!(result.stack.is_empty());
}

#[test]
fn returns_no_entry_when_everything_is_open_or_empty() {
    assert_eq!(take_closed_tab(Vec::new(), &[]).entry, None);
    let result = take_closed_tab(
        vec![ClosedTab {
            tab: session_tab("a"),
            index: 0,
        }],
        &[session_tab("a")],
    );
    assert_eq!(result.entry, None);
    assert!(result.stack.is_empty());
}

#[test]
fn purges_removed_sessions() {
    let stack = vec![
        ClosedTab {
            tab: session_tab("a"),
            index: 0,
        },
        ClosedTab {
            tab: session_tab("b"),
            index: 1,
        },
    ];
    assert_eq!(
        remove_closed_tabs(stack, SERVER, &["a"]),
        vec![ClosedTab {
            tab: session_tab("b"),
            index: 1
        }]
    );
}

#[test]
fn does_not_navigate_when_a_background_tab_closes() {
    let tabs = vec![session_tab("a"), session_tab("b"), session_tab("c")];
    assert_eq!(next_tab_after_close(&tabs, 1, false), None);
    assert_eq!(
        next_tab_after_close(&tabs, 1, true),
        Some(Some(session_tab("c")))
    );
    assert_eq!(
        next_tab_after_close(&[session_tab("a")], 0, true),
        Some(None)
    );
}
