//! Port of packages/tui/test/cli/cmd/tui/dialog-workspace-create.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/component/dialog-workspace-create.tsx; see docs/TEST-PORT.md.

use opencode_tui::dialog_workspace_create::{recent_connected_workspaces, WorkspaceEntry};

#[test]
fn returns_connected_workspaces_sorted_by_time_used() {
    let workspaces = vec![
        WorkspaceEntry {
            id: "wrk_a".to_string(),
            time_used: 700,
        },
        WorkspaceEntry {
            id: "wrk_b".to_string(),
            time_used: 800,
        },
        WorkspaceEntry {
            id: "wrk_c".to_string(),
            time_used: 400,
        },
        WorkspaceEntry {
            id: "wrk_d".to_string(),
            time_used: 300,
        },
        WorkspaceEntry {
            id: "wrk_e".to_string(),
            time_used: 200,
        },
    ];
    let status = |id: &str| -> Option<String> {
        match id {
            "wrk_a" => Some("connected".to_string()),
            "wrk_b" => Some("disconnected".to_string()),
            "wrk_c" => Some("error".to_string()),
            "wrk_d" => Some("connected".to_string()),
            "wrk_e" => Some("connected".to_string()),
            _ => None,
        }
    };

    let result = recent_connected_workspaces(&workspaces, status, None);
    assert_eq!(result.recent, vec!["wrk_a", "wrk_d", "wrk_e"]);
}
