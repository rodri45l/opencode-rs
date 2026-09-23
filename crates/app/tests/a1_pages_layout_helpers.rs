//! Port of packages/app/src/pages/layout/helpers.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//! DOM/effect-only cases (none here) are covered by the pure helpers below.
#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
struct NewSessionDeepLink {
    directory: String,
    prompt: Option<String>,
}

// Local stubs (fast wave): real module lands later.
fn parse_deep_link(_url: &str) -> Option<String> {
    None
}

fn collect_open_project_deep_links(_links: &[&str]) -> Vec<String> {
    Vec::new()
}

fn parse_new_session_deep_link(_url: &str) -> Option<NewSessionDeepLink> {
    None
}

fn collect_new_session_deep_links(_links: &[&str]) -> Vec<NewSessionDeepLink> {
    Vec::new()
}

#[allow(clippy::ptr_arg)]
fn drain_pending_deep_links(_links: &mut Vec<String>) -> Vec<String> {
    Vec::new()
}

fn path_key(path: &str) -> String {
    path.to_string()
}

fn effective_workspace_order(_local: &str, _known: &[&str], _order: &[&str]) -> Vec<String> {
    Vec::new()
}

#[derive(Clone, Debug, PartialEq)]
struct Session {
    id: String,
    directory: String,
    parent_id: Option<String>,
    archived: Option<i64>,
    created: i64,
    updated: i64,
}

fn latest_root_session(_groups: &[Vec<Session>], _window_ms: i64) -> Option<Session> {
    None
}

fn sorted_root_sessions(_sessions: Vec<Session>, _limit: usize) -> Vec<Session> {
    Vec::new()
}

fn compare_session_time(_a: &Session, _b: &Session) -> std::cmp::Ordering {
    std::cmp::Ordering::Equal
}

fn has_project_permissions(_map: &BTreeMap<String, Vec<String>>) -> bool {
    false
}

fn child_session_on_path(_list: &[Session], _root: &str, _target: &str) -> Option<Session> {
    None
}

fn display_name(_worktree: &str, _name: Option<&str>) -> String {
    String::new()
}

#[derive(Clone, Debug, PartialEq)]
struct ProjectSelection {
    server: String,
    directory: Option<String>,
}

fn toggle_home_project_selection(
    _current: Option<ProjectSelection>,
    _server: &str,
    _directory: &str,
) -> ProjectSelection {
    ProjectSelection {
        server: String::new(),
        directory: None,
    }
}

fn close_home_project(
    _current: ProjectSelection,
    _active_server: &str,
    _closed: &mut Vec<String>,
    _directory: &str,
) -> ProjectSelection {
    ProjectSelection {
        server: String::new(),
        directory: None,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Navigation {
    server: Option<String>,
    href: String,
}

fn home_project_navigation(_sidecar: &str, _active: &str, _href: &str) -> Navigation {
    Navigation {
        server: None,
        href: String::new(),
    }
}

fn home_project_directories(_input: Option<Vec<&str>>) -> Vec<String> {
    Vec::new()
}

#[derive(Clone, Debug, PartialEq)]
struct ServerStatus {
    working: bool,
    tint: Option<String>,
}

fn home_session_server_status(_active: bool) -> ServerStatus {
    ServerStatus {
        working: false,
        tint: None,
    }
}

fn error_message(_error: &str, _fallback: &str) -> String {
    String::new()
}

fn session(
    id: &str,
    directory: &str,
    parent_id: Option<&str>,
    archived: Option<i64>,
    updated: i64,
) -> Session {
    Session {
        id: id.into(),
        directory: directory.into(),
        parent_id: parent_id.map(str::to_string),
        archived,
        created: 1,
        updated,
    }
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn parses_open_project_deep_links() {
    assert_eq!(
        parse_deep_link("opencode://open-project?directory=/tmp/demo"),
        Some("/tmp/demo".into())
    );
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn ignores_non_project_deep_links() {
    assert_eq!(
        parse_deep_link("opencode://other?directory=/tmp/demo"),
        None
    );
    assert_eq!(parse_deep_link("https://example.com"), None);
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn ignores_malformed_deep_links_safely() {
    assert_eq!(parse_deep_link("opencode://open-project/%E0%A4%A%"), None);
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn ignores_open_project_deep_links_without_directory() {
    assert_eq!(parse_deep_link("opencode://open-project"), None);
    assert_eq!(parse_deep_link("opencode://open-project?directory="), None);
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn collects_only_valid_open_project_directories() {
    assert_eq!(
        collect_open_project_deep_links(&[
            "opencode://open-project?directory=/a",
            "opencode://other?directory=/b",
            "opencode://open-project?directory=/c",
        ]),
        vec!["/a".to_string(), "/c".to_string()]
    );
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn parses_new_session_deep_links_with_optional_prompt() {
    assert_eq!(
        parse_new_session_deep_link("opencode://new-session?directory=/tmp/demo"),
        Some(NewSessionDeepLink {
            directory: "/tmp/demo".into(),
            prompt: None
        })
    );
    assert_eq!(
        parse_new_session_deep_link(
            "opencode://new-session?directory=/tmp/demo&prompt=hello%20world"
        ),
        Some(NewSessionDeepLink {
            directory: "/tmp/demo".into(),
            prompt: Some("hello world".into())
        })
    );
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn ignores_new_session_deep_links_without_directory() {
    assert_eq!(parse_new_session_deep_link("opencode://new-session"), None);
    assert_eq!(
        parse_new_session_deep_link("opencode://new-session?directory="),
        None
    );
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn collects_only_valid_new_session_deep_links() {
    assert_eq!(
        collect_new_session_deep_links(&[
            "opencode://new-session?directory=/a",
            "opencode://open-project?directory=/b",
            "opencode://new-session?directory=/c&prompt=ship%20it",
        ]),
        vec![
            NewSessionDeepLink {
                directory: "/a".into(),
                prompt: None
            },
            NewSessionDeepLink {
                directory: "/c".into(),
                prompt: Some("ship it".into())
            },
        ]
    );
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn drains_global_deep_links_once() {
    let mut links = vec!["opencode://open-project?directory=/a".to_string()];
    assert_eq!(
        drain_pending_deep_links(&mut links),
        vec!["opencode://open-project?directory=/a".to_string()]
    );
    assert!(drain_pending_deep_links(&mut links).is_empty());
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn normalizes_trailing_slash_in_workspace_key() {
    assert_eq!(path_key("/tmp/demo///"), "/tmp/demo");
    assert_eq!(path_key("C:\\tmp\\demo\\\\"), "C:/tmp/demo");
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn preserves_posix_and_drive_roots_in_workspace_key() {
    assert_eq!(path_key("/"), "/");
    assert_eq!(path_key("///"), "/");
    assert_eq!(path_key("C:\\"), "C:/");
    assert_eq!(path_key("C://"), "C:/");
    assert_eq!(path_key("C:///"), "C:/");
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn keeps_local_first_while_preserving_known_order() {
    assert_eq!(
        effective_workspace_order(
            "/root",
            &["/root", "/b", "/c"],
            &["/root", "/c", "/a", "/b"]
        ),
        vec!["/root".to_string(), "/c".to_string(), "/b".to_string()]
    );
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn finds_the_latest_root_session_across_workspaces() {
    let groups = vec![
        vec![session("root", "/root", None, None, 1)],
        vec![session("workspace", "/workspace", None, None, 2)],
    ];
    assert_eq!(
        latest_root_session(&groups, 120_000).map(|s| s.id),
        Some("workspace".into())
    );
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn sorts_recent_sessions_by_persisted_update_time_instead_of_id() {
    let result = sorted_root_sessions(
        vec![
            session("ses_z", "/workspace", None, None, 2),
            session("ses_a", "/workspace", None, None, 3),
        ],
        3,
    );
    assert_eq!(
        result.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
        vec!["ses_a".to_string(), "ses_z".to_string()]
    );
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn uses_id_only_to_break_equal_session_timestamps() {
    let mut sessions = [
        session("ses_z", "/workspace", None, None, 2),
        session("ses_a", "/workspace", None, None, 2),
    ];
    sessions.sort_by(compare_session_time);
    assert_eq!(
        sessions.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
        vec!["ses_a".to_string(), "ses_z".to_string()]
    );
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn detects_project_permissions_with_a_filter() {
    let mut map = BTreeMap::new();
    map.insert(
        "root".to_string(),
        vec!["perm-root".to_string(), "perm-hidden".to_string()],
    );
    map.insert("child".to_string(), vec!["perm-child".to_string()]);
    assert!(has_project_permissions(&map));
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn ignores_project_permissions_filtered_out() {
    let mut map = BTreeMap::new();
    map.insert("root".to_string(), vec!["perm-root".to_string()]);
    assert!(!has_project_permissions(&map));
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn ignores_archived_and_child_sessions_when_finding_latest_root_session() {
    let groups = vec![vec![
        session("archived", "/workspace", None, Some(10), 10),
        session("child", "/workspace", Some("parent"), None, 20),
        session("root", "/workspace", None, None, 30),
    ]];
    assert_eq!(
        latest_root_session(&groups, 120_000).map(|s| s.id),
        Some("root".into())
    );
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn finds_the_direct_child_on_the_active_session_path() {
    let list = vec![
        session("root", "/workspace", None, None, 1),
        session("child", "/workspace", Some("root"), None, 1),
        session("leaf", "/workspace", Some("child"), None, 1),
    ];
    assert_eq!(
        child_session_on_path(&list, "root", "leaf").map(|s| s.id),
        Some("child".into())
    );
    assert_eq!(
        child_session_on_path(&list, "child", "leaf").map(|s| s.id),
        Some("leaf".into())
    );
    assert_eq!(child_session_on_path(&list, "root", "root"), None);
    assert_eq!(child_session_on_path(&list, "root", "other"), None);
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn formats_fallback_project_display_name() {
    assert_eq!(display_name("/tmp/app", None), "app");
    assert_eq!(display_name("/tmp/app", Some("My App")), "My App");
    assert_eq!(display_name("/", None), "/");
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn scopes_home_project_selection_by_server() {
    assert_eq!(
        toggle_home_project_selection(None, "https://debian.example", "/home/luke/repos/amazon"),
        ProjectSelection {
            server: "https://debian.example".into(),
            directory: Some("/home/luke/repos/amazon".into())
        }
    );
    assert_eq!(
        toggle_home_project_selection(
            Some(ProjectSelection {
                server: "https://windows.example".into(),
                directory: Some("/home/luke/repos/amazon".into())
            }),
            "https://debian.example",
            "/home/luke/repos/amazon"
        ),
        ProjectSelection {
            server: "https://debian.example".into(),
            directory: Some("/home/luke/repos/amazon".into())
        }
    );
    assert_eq!(
        toggle_home_project_selection(
            Some(ProjectSelection {
                server: "https://debian.example".into(),
                directory: Some("/home/luke/repos/amazon".into())
            }),
            "https://debian.example",
            "/home/luke/repos/amazon"
        ),
        ProjectSelection {
            server: "https://debian.example".into(),
            directory: None
        }
    );
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn closes_a_home_project_through_its_server_context() {
    let mut closed: Vec<String> = Vec::new();
    assert_eq!(
        close_home_project(
            ProjectSelection {
                server: "https://windows.example".into(),
                directory: Some("/shared".into())
            },
            "https://debian.example",
            &mut closed,
            "/shared"
        ),
        ProjectSelection {
            server: "https://windows.example".into(),
            directory: Some("/shared".into())
        }
    );
    assert_eq!(closed, vec!["/shared".to_string()]);
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn defers_home_project_navigation_until_its_server_is_active() {
    assert_eq!(
        home_project_navigation("sidecar", "https://debian.example", "/YW1hem9u/session"),
        Navigation {
            server: Some("https://debian.example".into()),
            href: "/YW1hem9u/session".into()
        }
    );
    assert_eq!(
        home_project_navigation(
            "https://debian.example",
            "https://debian.example",
            "/YW1hem9u/session"
        ),
        Navigation {
            server: None,
            href: "/YW1hem9u/session".into()
        }
    );
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn preserves_picker_order_when_adding_multiple_projects() {
    assert_eq!(
        home_project_directories(Some(vec!["/first", "/second"])),
        vec!["/first".to_string(), "/second".to_string()]
    );
    assert_eq!(
        home_project_directories(Some(vec!["/only"])),
        vec!["/only".to_string()]
    );
    assert!(home_project_directories(None).is_empty());
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn hides_status_derived_from_an_inactive_server() {
    assert_eq!(
        home_session_server_status(false),
        ServerStatus {
            working: false,
            tint: None
        }
    );
    assert_eq!(
        home_session_server_status(true),
        ServerStatus {
            working: true,
            tint: Some("red".into())
        }
    );
}

#[test]
#[ignore = "porting: pages/layout/helpers not implemented"]
fn extracts_api_error_message_and_fallback() {
    assert_eq!(error_message("boom", "fallback"), "boom");
    assert_eq!(error_message("unknown", "fallback"), "fallback");
}
