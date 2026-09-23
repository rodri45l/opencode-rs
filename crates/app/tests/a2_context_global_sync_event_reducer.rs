//! Port of packages/app/src/context/global-sync/event-reducer.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
struct Session {
    id: String,
    parent_id: Option<String>,
    archived: Option<i64>,
}

#[derive(Clone, Debug, PartialEq)]
struct Message {
    id: String,
    session_id: String,
    role: String,
    created: i64,
}

#[derive(Clone, Debug, PartialEq)]
struct Part {
    id: String,
    session_id: String,
    message_id: String,
    kind: String,
    text: String,
}

#[derive(Clone, Debug, PartialEq)]
struct PermissionRequest {
    id: String,
    session_id: String,
    permission: String,
}

#[derive(Clone, Debug, PartialEq)]
struct QuestionRequest {
    id: String,
    session_id: String,
    header: String,
}

#[derive(Clone, Debug, PartialEq)]
struct Vcs {
    branch: String,
    default_branch: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct State {
    session: Vec<Session>,
    session_total: i64,
    session_status: HashMap<String, String>,
    session_diff: HashMap<String, Vec<String>>,
    todo: HashMap<String, Vec<String>>,
    permission: HashMap<String, Vec<PermissionRequest>>,
    question: HashMap<String, Vec<QuestionRequest>>,
    message: HashMap<String, Vec<Message>>,
    part: HashMap<String, Vec<Part>>,
    part_text_accum_delta: HashMap<String, String>,
    vcs: Option<Vcs>,
}

#[derive(Clone, Debug)]
struct Project {
    id: String,
}

#[derive(Clone, Debug)]
enum GlobalEvent {
    ProjectUpdated { id: String },
    GlobalDisposed,
    ServerConnected,
}

#[derive(Clone, Debug)]
enum DirectoryEvent {
    MessagePartDelta {
        message_id: String,
        part_id: String,
        field: String,
        delta: String,
    },
    SessionCreated {
        info: Session,
    },
    SessionUpdated {
        info: Session,
    },
    SessionDeleted {
        info: Option<Session>,
        session_id: Option<String>,
    },
    MessageUpdated {
        info: Message,
    },
    MessageRemoved {
        session_id: String,
        message_id: String,
    },
    MessagePartUpdated {
        part: Part,
    },
    MessagePartRemoved {
        message_id: String,
        part_id: String,
    },
    PermissionAsked {
        request: PermissionRequest,
    },
    PermissionReplied {
        session_id: String,
        request_id: String,
    },
    QuestionAsked {
        request: QuestionRequest,
    },
    QuestionRejected {
        session_id: String,
        request_id: String,
    },
    VcsBranchUpdated {
        branch: String,
    },
    ServerInstanceDisposed,
    LspUpdated,
}

#[derive(Default)]
struct GlobalEffects {
    project: Vec<Project>,
    refresh_count: i64,
}

#[derive(Default)]
struct DirectoryEffects {
    pushes: Vec<String>,
    lsp_loads: i64,
    retained_limit: Option<i64>,
    todos: Vec<String>,
    vcs_cache: Option<Vcs>,
}

fn root_session(id: &str, parent_id: Option<&str>, archived: Option<i64>) -> Session {
    Session {
        id: id.into(),
        parent_id: parent_id.map(Into::into),
        archived,
    }
}

fn user_message(id: &str, session_id: &str, created: i64) -> Message {
    Message {
        id: id.into(),
        session_id: session_id.into(),
        role: "user".into(),
        created,
    }
}

fn text_part(id: &str, session_id: &str, message_id: &str) -> Part {
    Part {
        id: id.into(),
        session_id: session_id.into(),
        message_id: message_id.into(),
        kind: "text".into(),
        text: id.into(),
    }
}

fn permission_request(id: &str, session_id: &str, title: &str) -> PermissionRequest {
    PermissionRequest {
        id: id.into(),
        session_id: session_id.into(),
        permission: title.into(),
    }
}

fn question_request(id: &str, session_id: &str, title: &str) -> QuestionRequest {
    QuestionRequest {
        id: id.into(),
        session_id: session_id.into(),
        header: title.into(),
    }
}

// Local stubs (fast wave): real module lands later.
fn apply_global_event(_event: GlobalEvent, _effects: &mut GlobalEffects) {}

fn apply_directory_event(
    _state: &mut State,
    _event: DirectoryEvent,
    _effects: &mut DirectoryEffects,
) {
}

fn cleanup_dropped_session_caches(_state: &mut State, _sessions: &[Session]) {}

#[test]
#[ignore = "porting: context/global-sync/event-reducer not implemented"]
fn upserts_project_updated_in_sorted_position() {
    let mut effects = GlobalEffects {
        project: vec![Project { id: "a".into() }, Project { id: "c".into() }],
        refresh_count: 0,
    };

    apply_global_event(GlobalEvent::ProjectUpdated { id: "b".into() }, &mut effects);

    assert_eq!(
        effects
            .project
            .iter()
            .map(|p| p.id.clone())
            .collect::<Vec<_>>(),
        vec!["a".to_string(), "b".to_string(), "c".to_string()]
    );
    assert_eq!(effects.refresh_count, 0);
}

#[test]
#[ignore = "porting: context/global-sync/event-reducer not implemented"]
fn handles_global_disposed_and_server_connected_by_refreshing() {
    let mut effects = GlobalEffects::default();
    apply_global_event(GlobalEvent::GlobalDisposed, &mut effects);
    assert_eq!(effects.refresh_count, 1);

    let mut effects = GlobalEffects::default();
    apply_global_event(GlobalEvent::ServerConnected, &mut effects);
    assert_eq!(effects.refresh_count, 1);
}

#[test]
#[ignore = "porting: context/global-sync/event-reducer not implemented"]
fn initializes_text_delta_accumulation_from_the_current_part_text() {
    let mut part = text_part("part", "session", "message");
    part.text = "existing".into();
    let mut state = State {
        part: HashMap::from([("message".to_string(), vec![part])]),
        ..Default::default()
    };

    apply_directory_event(
        &mut state,
        DirectoryEvent::MessagePartDelta {
            message_id: "message".into(),
            part_id: "part".into(),
            field: "text".into(),
            delta: " appended".into(),
        },
        &mut DirectoryEffects::default(),
    );

    assert_eq!(
        state.part_text_accum_delta.get("part"),
        Some(&"existing appended".to_string())
    );
    assert_eq!(
        state.part.get("message").unwrap()[0].text,
        "existing appended"
    );
}

#[test]
#[ignore = "porting: context/global-sync/event-reducer not implemented"]
fn preserves_a_home_specific_retained_session_limit() {
    let mut state = State {
        session: vec![
            root_session("a", None, None),
            root_session("b", None, None),
            root_session("c", None, None),
        ],
        ..Default::default()
    };
    let mut effects = DirectoryEffects {
        retained_limit: Some(3),
        ..Default::default()
    };

    apply_directory_event(
        &mut state,
        DirectoryEvent::SessionCreated {
            info: root_session("d", None, None),
        },
        &mut effects,
    );

    assert_eq!(state.session.len(), 3);
}

#[test]
#[ignore = "porting: context/global-sync/event-reducer not implemented"]
fn inserts_root_sessions_in_sorted_order_and_updates_session_total() {
    let mut state = State {
        session: vec![root_session("b", None, None)],
        session_total: 1,
        ..Default::default()
    };

    apply_directory_event(
        &mut state,
        DirectoryEvent::SessionCreated {
            info: root_session("a", None, None),
        },
        &mut DirectoryEffects::default(),
    );

    assert_eq!(
        state
            .session
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>(),
        vec!["a".to_string(), "b".to_string()]
    );
    assert_eq!(state.session_total, 2);

    apply_directory_event(
        &mut state,
        DirectoryEvent::SessionCreated {
            info: root_session("c", Some("a"), None),
        },
        &mut DirectoryEffects::default(),
    );

    assert_eq!(state.session_total, 2);
}

#[test]
#[ignore = "porting: context/global-sync/event-reducer not implemented"]
fn cleans_session_caches_when_archived() {
    let message = user_message("msg_1", "ses_1", 1);
    let mut state = State {
        session: vec![
            root_session("ses_1", None, None),
            root_session("ses_2", None, None),
        ],
        session_total: 2,
        message: HashMap::from([("ses_1".to_string(), vec![message.clone()])]),
        part: HashMap::from([(
            message.id.clone(),
            vec![text_part("prt_1", "ses_1", &message.id)],
        )]),
        session_diff: HashMap::from([("ses_1".to_string(), Vec::new())]),
        todo: HashMap::from([("ses_1".to_string(), Vec::new())]),
        permission: HashMap::from([("ses_1".to_string(), Vec::new())]),
        question: HashMap::from([("ses_1".to_string(), Vec::new())]),
        session_status: HashMap::from([("ses_1".to_string(), "busy".to_string())]),
        ..Default::default()
    };

    apply_directory_event(
        &mut state,
        DirectoryEvent::SessionUpdated {
            info: root_session("ses_1", None, Some(10)),
        },
        &mut DirectoryEffects::default(),
    );

    assert_eq!(
        state
            .session
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>(),
        vec!["ses_2".to_string()]
    );
    assert_eq!(state.session_total, 1);
    assert!(!state.message.contains_key("ses_1"));
    assert!(!state.part.contains_key(&message.id));
    assert!(!state.session_diff.contains_key("ses_1"));
    assert!(!state.todo.contains_key("ses_1"));
    assert!(!state.permission.contains_key("ses_1"));
    assert!(!state.question.contains_key("ses_1"));
    assert!(!state.session_status.contains_key("ses_1"));
}

#[test]
#[ignore = "porting: context/global-sync/event-reducer not implemented"]
fn ignores_an_archived_session_absent_from_a_passive_directory_store() {
    let mut state = State::default();

    apply_directory_event(
        &mut state,
        DirectoryEvent::SessionUpdated {
            info: root_session("missing", None, Some(10)),
        },
        &mut DirectoryEffects::default(),
    );

    assert_eq!(state.session, Vec::new());
    assert_eq!(state.session_total, 0);
}

#[test]
#[ignore = "porting: context/global-sync/event-reducer not implemented"]
fn cleans_session_caches_when_deleted_and_decrements_only_root_totals() {
    for (info, expected_total, current) in [
        (root_session("ses_1", None, None), 1, false),
        (root_session("ses_2", Some("ses_1"), None), 2, true),
    ] {
        let message = user_message("msg_1", &info.id, 1);
        let mut state = State {
            session: vec![
                root_session("ses_1", None, None),
                root_session("ses_2", Some("ses_1"), None),
                root_session("ses_3", None, None),
            ],
            session_total: 2,
            message: HashMap::from([(info.id.clone(), vec![message.clone()])]),
            part: HashMap::from([(
                message.id.clone(),
                vec![text_part("prt_1", &info.id, &message.id)],
            )]),
            session_diff: HashMap::from([(info.id.clone(), Vec::new())]),
            todo: HashMap::from([(info.id.clone(), Vec::new())]),
            permission: HashMap::from([(info.id.clone(), Vec::new())]),
            question: HashMap::from([(info.id.clone(), Vec::new())]),
            session_status: HashMap::from([(info.id.clone(), "busy".to_string())]),
            ..Default::default()
        };

        apply_directory_event(
            &mut state,
            DirectoryEvent::SessionDeleted {
                info: if current { None } else { Some(info.clone()) },
                session_id: if current { Some(info.id.clone()) } else { None },
            },
            &mut DirectoryEffects::default(),
        );

        assert!(state.session.iter().find(|s| s.id == info.id).is_none());
        assert_eq!(state.session_total, expected_total);
        assert!(!state.message.contains_key(&info.id));
        assert!(!state.part.contains_key(&message.id));
        assert!(!state.session_diff.contains_key(&info.id));
        assert!(!state.todo.contains_key(&info.id));
        assert!(!state.permission.contains_key(&info.id));
        assert!(!state.question.contains_key(&info.id));
        assert!(!state.session_status.contains_key(&info.id));
    }
}

#[test]
#[ignore = "porting: context/global-sync/event-reducer not implemented"]
fn cleans_caches_for_trimmed_sessions_on_session_created() {
    let dropped = root_session("ses_b", None, None);
    let kept = root_session("ses_a", None, None);
    let message = user_message("msg_1", &dropped.id, 1);
    let mut state = State {
        session: vec![dropped.clone()],
        message: HashMap::from([(dropped.id.clone(), vec![message.clone()])]),
        part: HashMap::from([(
            message.id.clone(),
            vec![text_part("prt_1", &dropped.id, &message.id)],
        )]),
        session_diff: HashMap::from([(dropped.id.clone(), Vec::new())]),
        todo: HashMap::from([(dropped.id.clone(), Vec::new())]),
        permission: HashMap::from([(dropped.id.clone(), Vec::new())]),
        question: HashMap::from([(dropped.id.clone(), Vec::new())]),
        session_status: HashMap::from([(dropped.id.clone(), "busy".to_string())]),
        ..Default::default()
    };
    let mut effects = DirectoryEffects {
        retained_limit: Some(1),
        ..Default::default()
    };

    apply_directory_event(
        &mut state,
        DirectoryEvent::SessionCreated { info: kept.clone() },
        &mut effects,
    );

    assert_eq!(
        state
            .session
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>(),
        vec![kept.id.clone()]
    );
    assert!(!state.message.contains_key(&dropped.id));
    assert!(!state.part.contains_key(&message.id));
    assert!(!state.session_diff.contains_key(&dropped.id));
    assert!(!state.todo.contains_key(&dropped.id));
    assert!(!state.permission.contains_key(&dropped.id));
    assert!(!state.question.contains_key(&dropped.id));
    assert!(!state.session_status.contains_key(&dropped.id));
    assert_eq!(effects.todos, vec![dropped.id.clone()]);
}

#[test]
#[ignore = "porting: context/global-sync/event-reducer not implemented"]
fn cleanup_dropped_session_caches_clears_part_only_orphan_state() {
    let mut state = State {
        session: vec![root_session("ses_keep", None, None)],
        part: HashMap::from([(
            "msg_1".to_string(),
            vec![text_part("prt_1", "ses_drop", "msg_1")],
        )]),
        ..Default::default()
    };

    let sessions = state.session.clone();
    cleanup_dropped_session_caches(&mut state, &sessions);

    assert!(!state.part.contains_key("msg_1"));
}

#[test]
#[ignore = "porting: context/global-sync/event-reducer not implemented"]
fn upserts_and_removes_messages_while_clearing_orphaned_parts() {
    let session_id = "ses_1";
    let mut state = State {
        message: HashMap::from([(
            session_id.to_string(),
            vec![
                user_message("msg_z", session_id, 1),
                user_message("msg_b", session_id, 3),
            ],
        )]),
        part: HashMap::from([(
            "msg_a".to_string(),
            vec![text_part("prt_1", session_id, "msg_a")],
        )]),
        ..Default::default()
    };

    apply_directory_event(
        &mut state,
        DirectoryEvent::MessageUpdated {
            info: user_message("msg_a", session_id, 2),
        },
        &mut DirectoryEffects::default(),
    );

    assert_eq!(
        state
            .message
            .get(session_id)
            .unwrap()
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>(),
        vec![
            "msg_z".to_string(),
            "msg_a".to_string(),
            "msg_b".to_string()
        ]
    );

    apply_directory_event(
        &mut state,
        DirectoryEvent::MessageRemoved {
            session_id: session_id.into(),
            message_id: "msg_a".into(),
        },
        &mut DirectoryEffects::default(),
    );

    assert_eq!(
        state
            .message
            .get(session_id)
            .unwrap()
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>(),
        vec!["msg_z".to_string(), "msg_b".to_string()]
    );
    assert!(!state.part.contains_key("msg_a"));
}

#[test]
#[ignore = "porting: context/global-sync/event-reducer not implemented"]
fn upserts_and_prunes_message_parts() {
    let session_id = "ses_1";
    let message_id = "msg_1";
    let mut state = State {
        part: HashMap::from([(
            message_id.to_string(),
            vec![
                text_part("prt_1", session_id, message_id),
                text_part("prt_3", session_id, message_id),
            ],
        )]),
        ..Default::default()
    };

    apply_directory_event(
        &mut state,
        DirectoryEvent::MessagePartUpdated {
            part: text_part("prt_2", session_id, message_id),
        },
        &mut DirectoryEffects::default(),
    );
    assert_eq!(
        state
            .part
            .get(message_id)
            .unwrap()
            .iter()
            .map(|p| p.id.clone())
            .collect::<Vec<_>>(),
        vec![
            "prt_1".to_string(),
            "prt_2".to_string(),
            "prt_3".to_string()
        ]
    );

    for part_id in ["prt_1", "prt_2", "prt_3"] {
        apply_directory_event(
            &mut state,
            DirectoryEvent::MessagePartRemoved {
                message_id: message_id.into(),
                part_id: part_id.into(),
            },
            &mut DirectoryEffects::default(),
        );
    }

    assert!(!state.part.contains_key(message_id));
}

#[test]
#[ignore = "porting: context/global-sync/event-reducer not implemented"]
fn tracks_permission_and_question_request_lifecycles() {
    let session_id = "ses_1";
    let mut state = State {
        permission: HashMap::from([(
            session_id.to_string(),
            vec![
                permission_request("perm_1", session_id, "perm_1"),
                permission_request("perm_3", session_id, "perm_3"),
            ],
        )]),
        question: HashMap::from([(
            session_id.to_string(),
            vec![
                question_request("q_1", session_id, "q_1"),
                question_request("q_3", session_id, "q_3"),
            ],
        )]),
        ..Default::default()
    };

    apply_directory_event(
        &mut state,
        DirectoryEvent::PermissionAsked {
            request: permission_request("perm_2", session_id, "perm_2"),
        },
        &mut DirectoryEffects::default(),
    );
    assert_eq!(
        state
            .permission
            .get(session_id)
            .unwrap()
            .iter()
            .map(|p| p.id.clone())
            .collect::<Vec<_>>(),
        vec![
            "perm_1".to_string(),
            "perm_2".to_string(),
            "perm_3".to_string()
        ]
    );

    apply_directory_event(
        &mut state,
        DirectoryEvent::PermissionReplied {
            session_id: session_id.into(),
            request_id: "perm_2".into(),
        },
        &mut DirectoryEffects::default(),
    );
    assert_eq!(
        state
            .permission
            .get(session_id)
            .unwrap()
            .iter()
            .map(|p| p.id.clone())
            .collect::<Vec<_>>(),
        vec!["perm_1".to_string(), "perm_3".to_string()]
    );

    apply_directory_event(
        &mut state,
        DirectoryEvent::QuestionAsked {
            request: question_request("q_2", session_id, "q_2"),
        },
        &mut DirectoryEffects::default(),
    );
    assert_eq!(
        state
            .question
            .get(session_id)
            .unwrap()
            .iter()
            .map(|q| q.id.clone())
            .collect::<Vec<_>>(),
        vec!["q_1".to_string(), "q_2".to_string(), "q_3".to_string()]
    );

    apply_directory_event(
        &mut state,
        DirectoryEvent::QuestionRejected {
            session_id: session_id.into(),
            request_id: "q_2".into(),
        },
        &mut DirectoryEffects::default(),
    );
    assert_eq!(
        state
            .question
            .get(session_id)
            .unwrap()
            .iter()
            .map(|q| q.id.clone())
            .collect::<Vec<_>>(),
        vec!["q_1".to_string(), "q_3".to_string()]
    );
}

#[test]
#[ignore = "porting: context/global-sync/event-reducer not implemented"]
fn updates_vcs_branch_in_store_and_cache() {
    let mut state = State {
        vcs: Some(Vcs {
            branch: "main".into(),
            default_branch: "main".into(),
        }),
        ..Default::default()
    };
    let mut effects = DirectoryEffects {
        vcs_cache: Some(Vcs {
            branch: "main".into(),
            default_branch: "main".into(),
        }),
        ..Default::default()
    };

    apply_directory_event(
        &mut state,
        DirectoryEvent::VcsBranchUpdated {
            branch: "feature/test".into(),
        },
        &mut effects,
    );

    assert_eq!(
        state.vcs,
        Some(Vcs {
            branch: "feature/test".into(),
            default_branch: "main".into(),
        })
    );
    assert_eq!(
        effects.vcs_cache,
        Some(Vcs {
            branch: "feature/test".into(),
            default_branch: "main".into(),
        })
    );
}

#[test]
#[ignore = "porting: context/global-sync/event-reducer not implemented"]
fn routes_disposal_and_lsp_events_to_side_effect_handlers() {
    let mut state = State::default();
    let mut effects = DirectoryEffects::default();

    apply_directory_event(
        &mut state,
        DirectoryEvent::ServerInstanceDisposed,
        &mut effects,
    );
    apply_directory_event(&mut state, DirectoryEvent::LspUpdated, &mut effects);

    assert_eq!(effects.pushes, vec!["/tmp".to_string()]);
    assert_eq!(effects.lsp_loads, 1);
}
