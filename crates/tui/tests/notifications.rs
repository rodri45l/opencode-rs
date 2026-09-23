//! Port of packages/tui/test/cli/cmd/tui/notifications.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/feature-plugins/system/notifications.ts; see docs/TEST-PORT.md.

use opencode_tui::notifications::{
    NotificationEvent, Notifier, NotifyInput, SessionInfo, SessionStatus, SoundConfig,
};

fn session(id: &str, title: &str, parent_id: Option<&str>) -> SessionInfo {
    SessionInfo {
        id: id.to_string(),
        title: title.to_string(),
        parent_id: parent_id.map(str::to_string),
    }
}

fn notifier() -> Notifier {
    Notifier::new(vec![
        session("session", "Demo session", None),
        session("subagent", "Subagent session", Some("session")),
        session("abort", "Abort session", None),
        session("timeout", "Timeout session", None),
    ])
}

fn notification(title: &str, message: &str, blurred: bool, sound: &str) -> NotifyInput {
    NotifyInput {
        title: Some(title.to_string()),
        message: message.to_string(),
        notification: if blurred {
            Some("blurred".to_string())
        } else {
            None
        },
        sound: SoundConfig {
            name: sound.to_string(),
            when: "always".to_string(),
        },
    }
}

fn asked(notifier: &mut Notifier, id: &str, session_id: &str) -> Option<NotifyInput> {
    notifier.emit(NotificationEvent::QuestionAsked {
        id: id.to_string(),
        session_id: session_id.to_string(),
    })
}

#[test]
fn notifies_for_question_and_permission_requests() {
    let mut notifier = notifier();
    let question = asked(&mut notifier, "question-1", "session");
    let permission = notifier.emit(NotificationEvent::PermissionAsked {
        id: "permission-1".to_string(),
        session_id: "session".to_string(),
    });

    assert_eq!(
        vec![question, permission],
        vec![
            Some(notification(
                "Demo session",
                "Question needs input",
                true,
                "question"
            )),
            Some(notification(
                "Demo session",
                "Permission needs input",
                true,
                "permission"
            )),
        ]
    );
}

#[test]
fn dedupes_pending_questions_and_permissions_until_they_are_resolved() {
    let mut notifier = notifier();
    let mut notifications = Vec::new();
    notifications.push(asked(&mut notifier, "question-1", "session"));
    notifications.push(asked(&mut notifier, "question-1", "session"));
    notifier.emit(NotificationEvent::QuestionReplied {
        request_id: "question-1".to_string(),
    });
    notifications.push(asked(&mut notifier, "question-1", "session"));

    notifications.push(notifier.emit(NotificationEvent::PermissionAsked {
        id: "permission-1".to_string(),
        session_id: "session".to_string(),
    }));
    notifications.push(notifier.emit(NotificationEvent::PermissionAsked {
        id: "permission-1".to_string(),
        session_id: "session".to_string(),
    }));
    notifier.emit(NotificationEvent::PermissionReplied {
        request_id: "permission-1".to_string(),
    });
    notifications.push(notifier.emit(NotificationEvent::PermissionAsked {
        id: "permission-1".to_string(),
        session_id: "session".to_string(),
    }));

    assert_eq!(
        notifications,
        vec![
            Some(notification(
                "Demo session",
                "Question needs input",
                true,
                "question"
            )),
            None,
            Some(notification(
                "Demo session",
                "Question needs input",
                true,
                "question"
            )),
            Some(notification(
                "Demo session",
                "Permission needs input",
                true,
                "permission"
            )),
            None,
            Some(notification(
                "Demo session",
                "Permission needs input",
                true,
                "permission"
            )),
        ]
    );
}

#[test]
fn notifies_when_an_active_session_becomes_idle_and_suppresses_no_op_idle() {
    let mut notifier = notifier();
    let notifications = vec![
        notifier.emit(NotificationEvent::SessionStatus {
            session_id: "session".to_string(),
            status: SessionStatus::Idle,
        }),
        notifier.emit(NotificationEvent::SessionStatus {
            session_id: "session".to_string(),
            status: SessionStatus::Busy,
        }),
        notifier.emit(NotificationEvent::SessionStatus {
            session_id: "session".to_string(),
            status: SessionStatus::Idle,
        }),
    ];

    assert_eq!(
        notifications.into_iter().flatten().collect::<Vec<_>>(),
        vec![notification("Demo session", "Session done", true, "done")]
    );
}

#[test]
fn uses_sound_only_notifications_and_subagent_done_sound_for_subagent_sessions() {
    let mut notifier = notifier();
    let notifications = vec![
        asked(&mut notifier, "question-1", "subagent"),
        notifier.emit(NotificationEvent::SessionStatus {
            session_id: "subagent".to_string(),
            status: SessionStatus::Busy,
        }),
        notifier.emit(NotificationEvent::SessionStatus {
            session_id: "subagent".to_string(),
            status: SessionStatus::Idle,
        }),
    ];

    assert_eq!(
        notifications.into_iter().flatten().collect::<Vec<_>>(),
        vec![
            notification(
                "Subagent session",
                "Question needs input",
                false,
                "question"
            ),
            notification("Subagent session", "Session done", false, "subagent_done"),
        ]
    );
}

#[test]
fn notifies_session_errors_once_and_suppresses_the_following_idle_done_notification() {
    let mut notifier = notifier();
    let notifications = vec![
        notifier.emit(NotificationEvent::SessionStatus {
            session_id: "session".to_string(),
            status: SessionStatus::Busy,
        }),
        notifier.emit(NotificationEvent::SessionError {
            session_id: Some("session".to_string()),
            error_name: "UnknownError".to_string(),
            error_message: "boom".to_string(),
        }),
        notifier.emit(NotificationEvent::SessionStatus {
            session_id: "session".to_string(),
            status: SessionStatus::Idle,
        }),
    ];

    assert_eq!(
        notifications.into_iter().flatten().collect::<Vec<_>>(),
        vec![notification("Demo session", "Session error", true, "error")]
    );
}

#[test]
fn special_cases_aborts_and_model_response_timeouts() {
    let mut notifier = notifier();
    let notifications = vec![
        notifier.emit(NotificationEvent::SessionStatus {
            session_id: "abort".to_string(),
            status: SessionStatus::Busy,
        }),
        notifier.emit(NotificationEvent::SessionError {
            session_id: Some("abort".to_string()),
            error_name: "MessageAbortedError".to_string(),
            error_message: "Aborted".to_string(),
        }),
        notifier.emit(NotificationEvent::SessionStatus {
            session_id: "timeout".to_string(),
            status: SessionStatus::Busy,
        }),
        notifier.emit(NotificationEvent::SessionError {
            session_id: Some("timeout".to_string()),
            error_name: "UnknownError".to_string(),
            error_message: "SSE read timed out".to_string(),
        }),
    ];

    assert_eq!(
        notifications.into_iter().flatten().collect::<Vec<_>>(),
        vec![
            notification("Abort session", "Session aborted", true, "error"),
            notification("Timeout session", "Model stopped responding", true, "error"),
        ]
    );
}
