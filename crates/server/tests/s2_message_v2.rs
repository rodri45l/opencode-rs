#![allow(dead_code)]

//! Port of packages/opencode/test/session/message-v2.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the `session.message-v2.latest` selection cases. The
//! `toModelMessage` projection and `fromError` serialization from the same
//! reference file are already covered by `tests/message_v2.rs`; this file adds
//! the remaining pure selection surface (`latest`: contiguous user/assistant/
//! finished boundary plus compaction/subtask tasks). Stubs are local per the
//! fast-wave protocol and return a typed error until the module lands.

#[derive(Debug, Clone, PartialEq, Eq)]
enum S2Error {
    NotImplemented(&'static str),
}

impl std::fmt::Display for S2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S2Error::NotImplemented(what) => write!(f, "not implemented: {what}"),
        }
    }
}

impl std::error::Error for S2Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Part {
    Text,
    Compaction {
        auto: bool,
        tail_start_id: Option<String>,
    },
    Subtask {
        prompt: String,
    },
}

#[derive(Debug, Clone)]
struct Info {
    id: String,
    role: Role,
    created: f64,
    finish: Option<String>,
    summary: bool,
    parent_id: Option<String>,
}

#[derive(Debug, Clone)]
struct WithParts {
    info: Info,
    parts: Vec<Part>,
}

#[derive(Debug, Clone)]
struct Latest {
    user: Option<Info>,
    assistant: Option<Info>,
    finished: Option<Info>,
    tasks: Vec<Part>,
}

fn latest(_msgs: &[WithParts]) -> Result<Latest, S2Error> {
    Err(S2Error::NotImplemented("MessageV2.latest"))
}

fn user(id: &str, created: f64) -> Info {
    Info {
        id: id.to_string(),
        role: Role::User,
        created,
        finish: None,
        summary: false,
        parent_id: None,
    }
}

fn assistant(id: &str, parent: &str, created: f64) -> Info {
    Info {
        id: id.to_string(),
        role: Role::Assistant,
        created,
        finish: Some("stop".to_string()),
        summary: false,
        parent_id: Some(parent.to_string()),
    }
}

fn text_with(id: &str, created: f64) -> WithParts {
    WithParts {
        info: user(id, created),
        parts: vec![Part::Text],
    }
}

#[test]
#[ignore = "porting: MessageV2.latest not implemented"]
fn selects_latest_messages_by_creation_time_when_ids_are_nonmonotonic() {
    let old_user = user("msg_z_user", 100.0);
    let new_user = user("msg_a_user", 200.0);
    let old_assistant = assistant("msg_z_assistant", "msg_z_user", 300.0);
    let new_assistant = assistant("msg_a_assistant", "msg_a_user", 400.0);

    let state = latest(&[
        WithParts {
            info: new_assistant.clone(),
            parts: vec![],
        },
        WithParts {
            info: old_user.clone(),
            parts: vec![],
        },
        WithParts {
            info: old_assistant.clone(),
            parts: vec![],
        },
        WithParts {
            info: new_user.clone(),
            parts: vec![],
        },
    ])
    .expect("latest");

    assert_eq!(
        state.user.as_ref().map(|i| i.id.as_str()),
        Some("msg_a_user")
    );
    assert_eq!(
        state.assistant.as_ref().map(|i| i.id.as_str()),
        Some("msg_a_assistant")
    );
    assert_eq!(
        state.finished.as_ref().map(|i| i.id.as_str()),
        Some("msg_a_assistant")
    );
}

#[test]
#[ignore = "porting: MessageV2.latest not implemented"]
fn uses_id_as_deterministic_tie_breaker_for_equal_creation_times() {
    let lower = user("msg_a_user", 100.0);
    let higher = user("msg_z_user", 100.0);

    let state = latest(&[
        WithParts {
            info: higher.clone(),
            parts: vec![],
        },
        WithParts {
            info: lower.clone(),
            parts: vec![],
        },
    ])
    .expect("latest");

    assert_eq!(state.user.map(|i| i.id), Some("msg_z_user".to_string()));
}

#[test]
#[ignore = "porting: MessageV2.latest not implemented"]
fn finished_is_chronologically_latest_not_array_latest() {
    let tail_user = text_with("msg_001", 0.0);
    let overflow_assistant = WithParts {
        info: assistant("msg_002", "msg_001", 0.0),
        parts: vec![],
    };
    let compaction_user = WithParts {
        info: user("msg_003", 0.0),
        parts: vec![Part::Compaction {
            auto: true,
            tail_start_id: Some("msg_001".to_string()),
        }],
    };
    let summary_assistant = WithParts {
        info: Info {
            finish: Some("stop".to_string()),
            summary: true,
            ..assistant("msg_004", "msg_003", 0.0)
        },
        parts: vec![],
    };
    let continue_user = text_with("msg_005", 0.0);

    let state = latest(&[
        continue_user,
        summary_assistant,
        compaction_user,
        overflow_assistant,
        tail_user,
    ])
    .expect("latest");

    assert_eq!(
        state.finished.as_ref().map(|i| i.id.as_str()),
        Some("msg_004")
    );
    assert!(state.finished.as_ref().is_some_and(|i| i.summary));
    assert_eq!(state.user.as_ref().map(|i| i.id.as_str()), Some("msg_005"));
    assert!(state.tasks.is_empty());
}

#[test]
#[ignore = "porting: MessageV2.latest not implemented"]
fn fresh_compaction_user_newer_than_latest_summary_surfaces_in_tasks() {
    let tail_user = text_with("msg_001", 0.0);
    let overflow_assistant = WithParts {
        info: assistant("msg_002", "msg_001", 0.0),
        parts: vec![],
    };
    let compaction_user = WithParts {
        info: user("msg_003", 0.0),
        parts: vec![Part::Compaction {
            auto: true,
            tail_start_id: Some("msg_001".to_string()),
        }],
    };
    let summary_assistant = WithParts {
        info: Info {
            finish: Some("stop".to_string()),
            summary: true,
            ..assistant("msg_004", "msg_003", 0.0)
        },
        parts: vec![],
    };
    let continue_user = text_with("msg_005", 0.0);
    let new_compaction_user = WithParts {
        info: user("msg_006", 0.0),
        parts: vec![Part::Compaction {
            auto: true,
            tail_start_id: None,
        }],
    };

    let state = latest(&[
        tail_user,
        overflow_assistant,
        compaction_user,
        summary_assistant,
        continue_user,
        new_compaction_user,
    ])
    .expect("latest");

    assert_eq!(
        state.finished.as_ref().map(|i| i.id.as_str()),
        Some("msg_004")
    );
    assert_eq!(state.user.as_ref().map(|i| i.id.as_str()), Some("msg_006"));
    assert_eq!(state.tasks.len(), 1);
    assert_eq!(
        state.tasks[0],
        Part::Compaction {
            auto: true,
            tail_start_id: None
        }
    );
}

#[test]
#[ignore = "porting: MessageV2.latest not implemented"]
fn selects_compaction_and_subtask_work_after_finished_boundary_by_creation_time() {
    let finished = assistant("msg_z_finished", "msg_parent", 200.0);
    let old_task = WithParts {
        info: user("msg_z_old", 100.0),
        parts: vec![Part::Compaction {
            auto: true,
            tail_start_id: None,
        }],
    };
    let new_task = WithParts {
        info: user("msg_a_new", 300.0),
        parts: vec![Part::Subtask {
            prompt: "inspect".to_string(),
        }],
    };

    let state = latest(&[
        new_task,
        WithParts {
            info: finished,
            parts: vec![],
        },
        old_task,
    ])
    .expect("latest");

    assert_eq!(state.tasks.len(), 1);
    assert_eq!(
        state.tasks[0],
        Part::Subtask {
            prompt: "inspect".to_string()
        }
    );
}
