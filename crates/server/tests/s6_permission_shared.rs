//! Port of packages/opencode/test/cli/run/permission.shared.test.ts (upstream 18ef3cc).
//!
//! RED-first: the run permission-body state machine in
//! `cli/cmd/run/permission.shared` is not implemented in this crate. Reference
//! transitions are pinned against local typed stubs.

#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Permission,
    Always,
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Selected {
    Once,
    Always,
    Reject,
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PermissionBodyState {
    stage: Stage,
    selected: Selected,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PermissionRequest {
    id: String,
    permission: String,
    patterns: Vec<String>,
    always: Vec<String>,
    metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Reply {
    request_id: String,
    reply: String,
    message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunResult {
    state: PermissionBodyState,
    reply: Option<Reply>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PermissionInfo {
    title: String,
    lines: Vec<String>,
}

fn create_permission_body_state(id: &str) -> PermissionBodyState {
    let _ = id;
    PermissionBodyState {
        stage: Stage::Permission,
        selected: Selected::Once,
        message: String::new(),
    }
}

fn permission_run(state: PermissionBodyState, _id: &str, _action: &str) -> RunResult {
    RunResult { state, reply: None }
}

fn permission_reject(state: PermissionBodyState, _id: &str) -> Reply {
    let _ = state;
    Reply {
        request_id: String::new(),
        reply: String::new(),
        message: None,
    }
}

fn permission_cancel(state: PermissionBodyState) -> PermissionBodyState {
    state
}

fn permission_escape(state: PermissionBodyState) -> PermissionBodyState {
    state
}

fn permission_info(_req: &PermissionRequest) -> PermissionInfo {
    PermissionInfo {
        title: String::new(),
        lines: Vec::new(),
    }
}

fn permission_always_lines(_req: &PermissionRequest) -> Vec<String> {
    Vec::new()
}

fn req(input: &[(&str, &str)]) -> PermissionRequest {
    PermissionRequest {
        id: "perm-1".to_string(),
        permission: "read".to_string(),
        patterns: Vec::new(),
        always: Vec::new(),
        metadata: input
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    }
}

#[test]
#[ignore = "porting: cli run permission body not implemented"]
fn replies_immediately_for_allow_once() {
    let out = permission_run(create_permission_body_state("perm-1"), "perm-1", "once");
    assert_eq!(
        out.reply,
        Some(Reply {
            request_id: "perm-1".to_string(),
            reply: "once".to_string(),
            message: None,
        })
    );
}

#[test]
#[ignore = "porting: cli run permission body not implemented"]
fn requires_confirmation_for_allow_always() {
    let next = permission_run(create_permission_body_state("perm-1"), "perm-1", "always");
    assert_eq!(next.state.stage, Stage::Always);
    assert_eq!(next.state.selected, Selected::Confirm);
    assert_eq!(next.reply, None);

    assert_eq!(
        permission_run(next.state.clone(), "perm-1", "confirm")
            .reply
            .map(|r| r.reply),
        Some("always".to_string())
    );

    let cancelled = permission_run(next.state, "perm-1", "cancel").state;
    assert_eq!(cancelled.stage, Stage::Permission);
    assert_eq!(cancelled.selected, Selected::Always);
}

#[test]
#[ignore = "porting: cli run permission body not implemented"]
fn builds_trimmed_reject_replies_and_stage_transitions() {
    let next = permission_run(create_permission_body_state("perm-1"), "perm-1", "reject");
    assert_eq!(next.state.stage, Stage::Reject);

    let out = permission_reject(
        PermissionBodyState {
            message: "  use rg  ".to_string(),
            ..next.state.clone()
        },
        "perm-1",
    );
    assert_eq!(
        out,
        Reply {
            request_id: "perm-1".to_string(),
            reply: "reject".to_string(),
            message: Some("use rg".to_string()),
        }
    );

    let cancelled = permission_cancel(next.state);
    assert_eq!(cancelled.stage, Stage::Permission);
    assert_eq!(cancelled.selected, Selected::Reject);

    let escaped = permission_escape(create_permission_body_state("perm-1"));
    assert_eq!(escaped.stage, Stage::Reject);
    assert_eq!(escaped.selected, Selected::Reject);

    let back = permission_escape(PermissionBodyState {
        stage: Stage::Always,
        selected: Selected::Confirm,
        message: String::new(),
    });
    assert_eq!(back.stage, Stage::Permission);
    assert_eq!(back.selected, Selected::Always);
}

#[test]
#[ignore = "porting: cli run permission body not implemented"]
fn maps_supported_permission_types_into_display_info() {
    let info = permission_info(&PermissionRequest {
        permission: "bash".to_string(),
        metadata: [(
            "input.command".to_string(),
            "git status --short".to_string(),
        )]
        .into_iter()
        .collect(),
        ..req(&[])
    });
    assert_eq!(info.title, "Shell command");
    assert_eq!(info.lines, vec!["$ git status --short"]);

    let info = permission_info(&PermissionRequest {
        permission: "task".to_string(),
        metadata: [
            ("description".to_string(), "investigate stream".to_string()),
            ("subagent_type".to_string(), "general".to_string()),
        ]
        .into_iter()
        .collect(),
        ..req(&[])
    });
    assert_eq!(info.title, "General Task");
    assert_eq!(info.lines, vec!["◉ investigate stream"]);

    let info = permission_info(&PermissionRequest {
        permission: "external_directory".to_string(),
        patterns: vec![
            "/tmp/work/**/*.ts".to_string(),
            "/tmp/work/**/*.tsx".to_string(),
        ],
        ..req(&[])
    });
    assert_eq!(info.title, "Access external directory /tmp/work");
    assert_eq!(
        info.lines,
        vec!["- /tmp/work/**/*.ts", "- /tmp/work/**/*.tsx"]
    );

    assert_eq!(
        permission_info(&PermissionRequest {
            permission: "doom_loop".to_string(),
            ..req(&[])
        })
        .title,
        "Continue after repeated failures"
    );

    let info = permission_info(&PermissionRequest {
        permission: "custom_tool".to_string(),
        ..req(&[])
    });
    assert_eq!(info.title, "Call tool custom_tool");
    assert_eq!(info.lines, vec!["Tool: custom_tool"]);
}

#[test]
#[ignore = "porting: cli run permission body not implemented"]
fn formats_always_allow_copy_for_wildcard_and_explicit_patterns() {
    assert_eq!(
        permission_always_lines(&PermissionRequest {
            permission: "bash".to_string(),
            always: vec!["*".to_string()],
            ..req(&[])
        }),
        vec!["This will allow bash until OpenCode is restarted."]
    );

    assert_eq!(
        permission_always_lines(&PermissionRequest {
            always: vec!["src/**/*.ts".to_string(), "src/**/*.tsx".to_string()],
            ..req(&[])
        }),
        vec![
            "This will allow the following patterns until OpenCode is restarted.",
            "- src/**/*.ts",
            "- src/**/*.tsx",
        ]
    );
}
