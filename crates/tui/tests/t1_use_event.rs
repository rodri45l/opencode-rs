//! Port of packages/tui/test/cli/tui/use-event.test.tsx (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/context/event.ts and the SDK event
//! stream; see docs/TEST-PORT.md. The Solid provider mount is visual
//! (human-verified).
#![allow(dead_code)]

const CURRENT_PROJECT: &str = "proj_test";

#[derive(Debug, Clone, PartialEq)]
struct GlobalEvent {
    directory: String,
    project: Option<String>,
    workspace: Option<String>,
    payload_type: String,
}

/// Returns the workspace metadata when the event should reach the current
/// project, or `None` when it is filtered out. `sync` events are always
/// skipped; `global` events bypass the project check; project events are
/// delivered regardless of the active workspace.
fn deliver(event: &GlobalEvent) -> Option<Option<String>> {
    if event.payload_type == "sync" {
        return None;
    }
    let global = event.directory == "global";
    let project_match = event.project.as_deref() == Some(CURRENT_PROJECT);
    if global || project_match {
        Some(event.workspace.clone())
    } else {
        None
    }
}

fn event(
    directory: &str,
    project: Option<&str>,
    workspace: Option<&str>,
    payload_type: &str,
) -> GlobalEvent {
    GlobalEvent {
        directory: directory.to_string(),
        project: project.map(str::to_string),
        workspace: workspace.map(str::to_string),
        payload_type: payload_type.to_string(),
    }
}

#[test]
fn delivers_events_for_the_current_project() {
    let emitted = event(
        "/tmp/other",
        Some(CURRENT_PROJECT),
        Some("ws_a"),
        "vcs.branch.updated",
    );
    assert_eq!(deliver(&emitted), Some(Some("ws_a".to_string())));
}

#[test]
fn delivers_current_project_events_regardless_of_active_workspace() {
    let emitted = event(
        "/tmp/other",
        Some(CURRENT_PROJECT),
        Some("ws_b"),
        "vcs.branch.updated",
    );
    assert_eq!(deliver(&emitted), Some(Some("ws_b".to_string())));
}

#[test]
fn delivers_truly_global_events_even_when_a_workspace_is_active() {
    let emitted = event("global", None, None, "installation.update-available");
    assert_eq!(deliver(&emitted), Some(None));
}

#[test]
fn drops_sync_and_other_project_events() {
    assert_eq!(deliver(&event("global", None, None, "sync")), None);
    assert_eq!(
        deliver(&event(
            "/tmp/other",
            Some("other_project"),
            None,
            "vcs.branch.updated"
        )),
        None
    );
}
