//! Port of packages/app/src/context/terminal.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

const SEP: char = '\u{0}';

#[derive(Clone, Debug, Default, PartialEq)]
struct TerminalEntry {
    id: Option<String>,
    title: Option<String>,
    title_number: Option<i64>,
    rows: Option<i64>,
    cols: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct TerminalState {
    active: Option<String>,
    all: Vec<TerminalEntry>,
}

// Local stubs (fast wave): real module lands later.
fn get_workspace_terminal_cache_key(_dir: &str, _scope: Option<&str>) -> String {
    String::new()
}

fn get_legacy_terminal_storage_keys(_dir: &str, _legacy_session_id: Option<&str>) -> Vec<String> {
    Vec::new()
}

fn migrate_terminal_state(_value: &TerminalState) -> TerminalState {
    TerminalState::default()
}

#[test]
#[ignore = "porting: context/terminal not implemented"]
fn uses_workspace_only_directory_cache_key() {
    assert_eq!(
        get_workspace_terminal_cache_key("/repo", None),
        format!("local{SEP}/repo{SEP}__workspace__")
    );
}

#[test]
#[ignore = "porting: context/terminal not implemented"]
fn can_include_a_server_scope() {
    assert_eq!(
        get_workspace_terminal_cache_key("/repo", Some("ssh:debian")),
        format!("ssh:debian{SEP}/repo{SEP}__workspace__")
    );
}

#[test]
#[ignore = "porting: context/terminal not implemented"]
fn keeps_workspace_storage_path_when_no_legacy_session_id() {
    assert_eq!(
        get_legacy_terminal_storage_keys("/repo", None),
        vec!["/repo/terminal.v1".to_string()]
    );
}

#[test]
#[ignore = "porting: context/terminal not implemented"]
fn includes_legacy_session_path_before_workspace_path() {
    assert_eq!(
        get_legacy_terminal_storage_keys("/repo", Some("session-123")),
        vec![
            "/repo/terminal/session-123.v1".to_string(),
            "/repo/terminal.v1".to_string(),
        ]
    );
}

#[test]
#[ignore = "porting: context/terminal not implemented"]
fn drops_invalid_terminals_and_restores_a_valid_active_terminal() {
    let input = TerminalState {
        active: Some("missing".into()),
        all: vec![
            TerminalEntry::default(),
            TerminalEntry {
                id: Some("one".into()),
                title: Some("Terminal 2".into()),
                ..Default::default()
            },
            TerminalEntry {
                id: Some("one".into()),
                title: Some("duplicate".into()),
                title_number: Some(9),
                ..Default::default()
            },
            TerminalEntry {
                id: Some("two".into()),
                title: Some("logs".into()),
                title_number: Some(4),
                rows: Some(24),
                cols: Some(80),
            },
            TerminalEntry {
                title: Some("no-id".into()),
                ..Default::default()
            },
        ],
    };

    assert_eq!(
        migrate_terminal_state(&input),
        TerminalState {
            active: Some("one".into()),
            all: vec![
                TerminalEntry {
                    id: Some("one".into()),
                    title: Some("Terminal 2".into()),
                    title_number: Some(2),
                    ..Default::default()
                },
                TerminalEntry {
                    id: Some("two".into()),
                    title: Some("logs".into()),
                    title_number: Some(4),
                    rows: Some(24),
                    cols: Some(80),
                },
            ],
        }
    );
}

#[test]
#[ignore = "porting: context/terminal not implemented"]
fn keeps_a_valid_active_id() {
    let input = TerminalState {
        active: Some("two".into()),
        all: vec![
            TerminalEntry {
                id: Some("one".into()),
                title: Some("Terminal 1".into()),
                ..Default::default()
            },
            TerminalEntry {
                id: Some("two".into()),
                title: Some("shell".into()),
                title_number: Some(7),
                ..Default::default()
            },
        ],
    };

    assert_eq!(
        migrate_terminal_state(&input),
        TerminalState {
            active: Some("two".into()),
            all: vec![
                TerminalEntry {
                    id: Some("one".into()),
                    title: Some("Terminal 1".into()),
                    title_number: Some(1),
                    ..Default::default()
                },
                TerminalEntry {
                    id: Some("two".into()),
                    title: Some("shell".into()),
                    title_number: Some(7),
                    ..Default::default()
                },
            ],
        }
    );
}
