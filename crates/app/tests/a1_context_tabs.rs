//! Port of packages/app/src/context/tabs.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

const SERVER: &str = "local\nhttp://localhost:4096";

#[derive(Clone, Debug, PartialEq)]
enum Tab {
    Session {
        server: String,
        session_id: String,
    },
    Draft {
        draft_id: String,
        server: String,
        directory: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
enum RawTab {
    Session {
        server: Option<String>,
        session_id: String,
        dir_base64: Option<String>,
    },
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
struct ClosedTab {
    tab: Tab,
    index: i64,
}

#[derive(Clone, Debug, PartialEq)]
struct TakeResult {
    entry: Option<ClosedTab>,
    stack: Vec<ClosedTab>,
}

fn session_tab(session_id: &str) -> Tab {
    Tab::Session {
        server: SERVER.into(),
        session_id: session_id.into(),
    }
}

// Local stubs (fast wave): real module lands later.
fn migrate_tabs(_tabs: Option<Vec<Option<RawTab>>>, _fallback: &str) -> Vec<Tab> {
    Vec::new()
}

fn push_closed_tab(_stack: Vec<ClosedTab>, _tab: Tab, _index: i64) -> Vec<ClosedTab> {
    Vec::new()
}

fn take_closed_tab(_stack: Vec<ClosedTab>, _open: &[Tab]) -> TakeResult {
    TakeResult {
        entry: None,
        stack: Vec::new(),
    }
}

fn remove_closed_tabs(
    _stack: Vec<ClosedTab>,
    _server: &str,
    _session_ids: &[&str],
) -> Vec<ClosedTab> {
    Vec::new()
}

fn next_tab_after_close(_tabs: &[Tab], _index: usize, _navigate: bool) -> Option<Option<Tab>> {
    None
}

#[test]
#[ignore = "porting: context/tabs not implemented"]
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
#[ignore = "porting: context/tabs not implemented"]
fn adds_the_fallback_server_to_valid_legacy_tabs() {
    let input = vec![Some(RawTab::Session {
        server: None,
        session_id: "a".into(),
        dir_base64: Some("legacy".into()),
    })];
    assert_eq!(migrate_tabs(Some(input), SERVER), vec![session_tab("a")]);
}

#[test]
#[ignore = "porting: context/tabs not implemented"]
fn replaces_invalid_top_level_persisted_data() {
    assert_eq!(migrate_tabs(None, SERVER), Vec::<Tab>::new());
    assert_eq!(migrate_tabs(Some(Vec::new()), SERVER), Vec::<Tab>::new());
}

#[test]
#[ignore = "porting: context/tabs not implemented"]
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
#[ignore = "porting: context/tabs not implemented"]
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
#[ignore = "porting: context/tabs not implemented"]
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
#[ignore = "porting: context/tabs not implemented"]
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
#[ignore = "porting: context/tabs not implemented"]
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
#[ignore = "porting: context/tabs not implemented"]
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
#[ignore = "porting: context/tabs not implemented"]
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
#[ignore = "porting: context/tabs not implemented"]
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
