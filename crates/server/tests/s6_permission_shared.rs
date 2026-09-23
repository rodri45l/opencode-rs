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

fn permission_run(state: PermissionBodyState, id: &str, action: &str) -> RunResult {
    match state.stage {
        Stage::Permission => match action {
            "always" => RunResult {
                state: PermissionBodyState {
                    stage: Stage::Always,
                    selected: Selected::Confirm,
                    ..state
                },
                reply: None,
            },
            "reject" => RunResult {
                state: PermissionBodyState {
                    stage: Stage::Reject,
                    selected: Selected::Reject,
                    ..state
                },
                reply: None,
            },
            _ => RunResult {
                state,
                reply: Some(Reply {
                    request_id: id.to_string(),
                    reply: "once".to_string(),
                    message: None,
                }),
            },
        },
        Stage::Always => {
            if action == "cancel" {
                RunResult {
                    state: PermissionBodyState {
                        stage: Stage::Permission,
                        selected: Selected::Always,
                        ..state
                    },
                    reply: None,
                }
            } else {
                RunResult {
                    state,
                    reply: Some(Reply {
                        request_id: id.to_string(),
                        reply: "always".to_string(),
                        message: None,
                    }),
                }
            }
        }
        Stage::Reject => RunResult { state, reply: None },
    }
}

fn permission_reject(state: PermissionBodyState, id: &str) -> Reply {
    let message = state.message.trim();
    Reply {
        request_id: id.to_string(),
        reply: "reject".to_string(),
        message: if message.is_empty() {
            None
        } else {
            Some(message.to_string())
        },
    }
}

fn permission_cancel(state: PermissionBodyState) -> PermissionBodyState {
    PermissionBodyState {
        stage: Stage::Permission,
        selected: Selected::Reject,
        ..state
    }
}

fn permission_escape(state: PermissionBodyState) -> PermissionBodyState {
    if state.stage == Stage::Always {
        PermissionBodyState {
            stage: Stage::Permission,
            selected: Selected::Always,
            ..state
        }
    } else {
        PermissionBodyState {
            stage: Stage::Reject,
            selected: Selected::Reject,
            ..state
        }
    }
}

fn titlecase(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn permission_info(req: &PermissionRequest) -> PermissionInfo {
    let get = |key: &str| req.metadata.get(key).cloned().unwrap_or_default();
    match req.permission.as_str() {
        "bash" => {
            let command = {
                let direct = get("command");
                if direct.is_empty() {
                    get("input.command")
                } else {
                    direct
                }
            };
            let lines = if command.is_empty() {
                req.patterns
                    .iter()
                    .map(|item| format!("- {item}"))
                    .collect()
            } else {
                vec![format!("$ {command}")]
            };
            PermissionInfo {
                title: "Shell command".to_string(),
                lines,
            }
        }
        "task" => {
            let subagent = {
                let direct = get("subagent_type");
                if direct.is_empty() {
                    "general".to_string()
                } else {
                    direct
                }
            };
            let description = get("description");
            PermissionInfo {
                title: format!("{} Task", titlecase(&subagent)),
                lines: if description.is_empty() {
                    Vec::new()
                } else {
                    vec![format!("◉ {description}")]
                },
            }
        }
        "external_directory" => {
            let raw = {
                let parent = get("parentDir");
                if !parent.is_empty() {
                    parent
                } else {
                    let filepath = get("filepath");
                    if !filepath.is_empty() {
                        filepath
                    } else {
                        req.patterns.first().cloned().unwrap_or_default()
                    }
                }
            };
            let dir = if let Some(index) = raw.find('*') {
                raw[..index].trim_end_matches(['/', '\\']).to_string()
            } else {
                raw
            };
            PermissionInfo {
                title: format!("Access external directory {dir}"),
                lines: req
                    .patterns
                    .iter()
                    .map(|item| format!("- {item}"))
                    .collect(),
            }
        }
        "doom_loop" => PermissionInfo {
            title: "Continue after repeated failures".to_string(),
            lines: vec!["This keeps the session running despite repeated failures.".to_string()],
        },
        other => PermissionInfo {
            title: format!("Call tool {other}"),
            lines: vec![format!("Tool: {other}")],
        },
    }
}

fn permission_always_lines(req: &PermissionRequest) -> Vec<String> {
    if req.always.len() == 1 && req.always[0] == "*" {
        return vec![format!(
            "This will allow {} until OpenCode is restarted.",
            req.permission
        )];
    }
    let mut lines =
        vec!["This will allow the following patterns until OpenCode is restarted.".to_string()];
    lines.extend(req.always.iter().map(|item| format!("- {item}")));
    lines
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
