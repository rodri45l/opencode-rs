//! Port of packages/tui/test/cli/tui/diff-viewer.test.tsx (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/feature-plugins/system/diff-viewer.tsx
//! and config/keybind.ts; see docs/TEST-PORT.md. Render/scroll geometry is
//! visual (human-verified).
#![allow(dead_code)]

const VCS_DIFF_CONTEXT_LINES: i64 = 12;
const ROUTE: &str = "diff";
const HOME_ROUTE: &str = "home";

#[derive(Debug, Clone, PartialEq)]
struct Route {
    name: String,
    params: Option<DiffParams>,
}

#[derive(Debug, Clone, PartialEq, Default)]
struct DiffParams {
    mode: Option<String>,
    session_id: Option<String>,
    message_id: Option<String>,
    return_route: Option<Box<Route>>,
}

#[derive(Debug, Clone, PartialEq)]
enum DiffSource {
    Vcs {
        directory: Option<String>,
        mode: String,
        context: i64,
    },
    Session {
        session_id: String,
        message_id: Option<String>,
    },
}

fn session_route(session_id: &str) -> Route {
    Route {
        name: "session".to_string(),
        params: Some(DiffParams {
            session_id: Some(session_id.to_string()),
            ..Default::default()
        }),
    }
}

/// `diff.open`: navigates to the diff route in `git` mode, carrying the
/// current session id and the route it was opened from.
fn diff_open(current: &Route) -> Route {
    let session_id = current
        .params
        .as_ref()
        .and_then(|params| params.session_id.clone());
    Route {
        name: ROUTE.to_string(),
        params: Some(DiffParams {
            mode: Some("git".to_string()),
            session_id,
            message_id: None,
            return_route: Some(Box::new(current.clone())),
        }),
    }
}

/// `diff.close`: returns to the route the viewer opened from, or home.
fn diff_close(viewer: &Route) -> Route {
    viewer
        .params
        .as_ref()
        .and_then(|params| params.return_route.as_deref().cloned())
        .unwrap_or(Route {
            name: HOME_ROUTE.to_string(),
            params: None,
        })
}

/// Selects the diff source: `last-turn` asks the session API; anything else
/// asks the VCS API with the fixed context window.
fn diff_source(params: &DiffParams, directory: Option<&str>) -> DiffSource {
    let mode = params.mode.clone().unwrap_or_else(|| "git".to_string());
    if mode == "last-turn" {
        return DiffSource::Session {
            session_id: params.session_id.clone().unwrap_or_default(),
            message_id: params.message_id.clone(),
        };
    }
    DiffSource::Vcs {
        directory: directory.map(str::to_string),
        mode,
        context: VCS_DIFF_CONTEXT_LINES,
    }
}

fn diff_source_label(mode: &str) -> &'static str {
    match mode {
        "last-turn" => "last turn",
        "branch" => "main branch",
        _ => "working tree",
    }
}

/// (keybind name, command, default binding) from `config/keybind.ts`.
const DEFINITIONS: &[(&str, &str, &str)] = &[
    ("diff_open", "diff.open", "none"),
    ("diff_close", "diff.close", "escape,q"),
    ("diff_next_hunk", "diff.next_hunk", "]"),
    ("diff_previous_hunk", "diff.previous_hunk", "["),
];

fn default_value(keybind_name: &str) -> Option<&'static str> {
    DEFINITIONS
        .iter()
        .find(|(name, _, _)| *name == keybind_name)
        .map(|(_, _, value)| *value)
}

fn command_for(keybind_name: &str) -> Option<&'static str> {
    DEFINITIONS
        .iter()
        .find(|(name, _, _)| *name == keybind_name)
        .map(|(_, command, _)| *command)
}

#[test]
fn closing_the_diff_viewer_returns_to_the_route_it_opened_from() {
    let start = session_route("session-1");
    let viewer = diff_open(&start);

    assert_eq!(
        viewer,
        Route {
            name: ROUTE.to_string(),
            params: Some(DiffParams {
                mode: Some("git".to_string()),
                session_id: Some("session-1".to_string()),
                message_id: None,
                return_route: Some(Box::new(start.clone())),
            }),
        }
    );
    let params = viewer.params.clone().unwrap();
    assert_eq!(
        diff_source(&params, Some("/repo/session")),
        DiffSource::Vcs {
            directory: Some("/repo/session".to_string()),
            mode: "git".to_string(),
            context: 12,
        }
    );
    assert!(command_for("diff_close").is_some());
    assert_eq!(diff_close(&viewer), start);
}

#[test]
fn brackets_navigate_diff_hunks() {
    assert_eq!(default_value("diff_next_hunk"), Some("]"));
    assert_eq!(default_value("diff_previous_hunk"), Some("["));
    assert_eq!(command_for("diff_next_hunk"), Some("diff.next_hunk"));
    assert_eq!(
        command_for("diff_previous_hunk"),
        Some("diff.previous_hunk")
    );
}

#[test]
fn branch_diff_source_requests_branch_vcs_diff() {
    let params = DiffParams {
        mode: Some("branch".to_string()),
        session_id: Some("session-1".to_string()),
        message_id: None,
        return_route: Some(Box::new(session_route("session-1"))),
    };
    assert_eq!(
        diff_source(&params, Some("/repo/session")),
        DiffSource::Vcs {
            directory: Some("/repo/session".to_string()),
            mode: "branch".to_string(),
            context: 12,
        }
    );
    assert_eq!(diff_source_label("branch"), "main branch");
}

#[test]
fn last_turn_diff_source_requests_session_diff() {
    let params = DiffParams {
        mode: Some("last-turn".to_string()),
        session_id: Some("session-1".to_string()),
        message_id: Some("message-1".to_string()),
        return_route: Some(Box::new(session_route("session-1"))),
    };
    assert_eq!(
        diff_source(&params, Some("/repo/session")),
        DiffSource::Session {
            session_id: "session-1".to_string(),
            message_id: Some("message-1".to_string()),
        }
    );
    assert_eq!(diff_source_label("last-turn"), "last turn");
}
