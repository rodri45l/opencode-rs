//! Port of packages/app/src/utils/session.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use opencode_app::session::{
    list_all_sessions, normalize_session_info, zero_tokens, Model, Page, SessionInfo,
};

#[test]
fn adapts_a_current_session_to_the_app_session_shape() {
    let info = SessionInfo {
        id: "session-1".into(),
        parent_id: None,
        project_id: "project-1".into(),
        workspace_id: Some("workspace-1".into()),
        directory: "/repo/worktree".into(),
        subpath: Some("worktree".into()),
        agent: Some("build".into()),
        model: Some(Model {
            id: "gpt-5".into(),
            provider_id: "openai".into(),
            variant: Some("high".into()),
        }),
        cost: 0.0,
        tokens: zero_tokens(),
        title: Some("New session".into()),
        created: 1,
        updated: 1,
        archived: None,
        revert: Some("message-1/part-1/snapshot".into()),
    };

    assert_eq!(
        normalize_session_info(&info),
        opencode_app::session::AppSession {
            id: "session-1".into(),
            slug: "session-1".into(),
            project_id: "project-1".into(),
            workspace_id: Some("workspace-1".into()),
            directory: "/repo/worktree".into(),
            path: Some("worktree".into()),
            parent_id: None,
            cost: 0.0,
            tokens: zero_tokens(),
            title: "New session".into(),
            agent: Some("build".into()),
            model: Some(Model {
                id: "gpt-5".into(),
                provider_id: "openai".into(),
                variant: Some("high".into())
            }),
            version: String::new(),
            created: 1,
            updated: 1,
            revert: Some("message-1/part-1/snapshot".into()),
        }
    );
}

#[test]
fn supplies_timestamped_titles_for_untitled_current_sessions() {
    assert_eq!(
        normalize_session_info(&current("session-1", None)).title,
        "New session - 1970-01-01T00:00:00.000Z"
    );
    assert_eq!(
        normalize_session_info(&current("session-2", Some("session-1"))).title,
        "Child session - 1970-01-01T00:00:00.000Z"
    );
}

#[test]
fn loads_every_page_in_server_order_and_retains_the_query() {
    let pages = vec![
        Page {
            data: vec![session_info("session-3"), session_info("session-2")],
            next: Some("next".into()),
        },
        Page {
            data: vec![session_info_archived("session-1")],
            next: None,
        },
    ];
    let result = list_all_sessions(&pages, "directory=/repo&order=desc");
    assert_eq!(
        result.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
        vec!["session-3", "session-2", "session-1"]
    );
    assert_eq!(result[2].archived, Some(2));
}

#[test]
fn requests_the_terminal_empty_page_when_the_server_returns_a_next_cursor() {
    let pages = vec![
        Page {
            data: vec![session_info("session-1")],
            next: Some("terminal".into()),
        },
        Page {
            data: Vec::new(),
            next: None,
        },
    ];
    let result = list_all_sessions(&pages, "directory=/repo&limit=25");
    assert_eq!(
        result.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
        vec!["session-1"]
    );
}

fn session_info(id: &str) -> SessionInfo {
    SessionInfo {
        id: id.into(),
        parent_id: None,
        project_id: "project-1".into(),
        workspace_id: None,
        directory: "/repo".into(),
        subpath: None,
        agent: Some("build".into()),
        model: Some(Model {
            id: "model-1".into(),
            provider_id: "provider-1".into(),
            variant: None,
        }),
        cost: 0.0,
        tokens: zero_tokens(),
        title: Some(id.into()),
        created: 1,
        updated: 1,
        archived: None,
        revert: None,
    }
}

fn session_info_archived(id: &str) -> SessionInfo {
    let mut info = session_info(id);
    info.archived = Some(2);
    info
}

fn current(id: &str, parent_id: Option<&str>) -> SessionInfo {
    SessionInfo {
        id: id.into(),
        parent_id: parent_id.map(str::to_string),
        project_id: "project-1".into(),
        workspace_id: None,
        directory: "/repo".into(),
        subpath: None,
        agent: None,
        model: None,
        cost: 0.0,
        tokens: zero_tokens(),
        title: None,
        created: 0,
        updated: 0,
        archived: None,
        revert: None,
    }
}
