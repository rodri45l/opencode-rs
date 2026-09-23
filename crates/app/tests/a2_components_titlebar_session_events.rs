//! Port of packages/app/src/components/titlebar-session-events.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

const SESSION_TABS_REMOVED_EVENT: &str = "opencode:session-tabs-removed";

#[derive(Clone, Debug, PartialEq)]
enum RawId {
    Str(String),
    Num(i64),
}

#[derive(Clone, Debug, PartialEq)]
struct RawDetail {
    server: Option<String>,
    directory: Option<String>,
    session_ids: Vec<RawId>,
}

#[derive(Clone, Debug, Default)]
struct SessionTabsEvent {
    kind: String,
    detail: Option<RawDetail>,
}

#[derive(Clone, Debug, PartialEq)]
struct RemovedDetail {
    server: String,
    directory: String,
    session_ids: Vec<String>,
}

// Local stub (fast wave): real module lands later.
fn read_session_tabs_removed_detail(_event: &SessionTabsEvent) -> Option<RemovedDetail> {
    None
}

#[test]
#[ignore = "porting: components/titlebar-session-events not implemented"]
fn reads_valid_removed_session_tab_details() {
    let event = SessionTabsEvent {
        kind: SESSION_TABS_REMOVED_EVENT.into(),
        detail: Some(RawDetail {
            server: Some("remote".into()),
            directory: Some("/tmp/project".into()),
            session_ids: vec![
                RawId::Str("ses_1".into()),
                RawId::Str("ses_2".into()),
                RawId::Num(1),
            ],
        }),
    };

    assert_eq!(
        read_session_tabs_removed_detail(&event),
        Some(RemovedDetail {
            server: "remote".into(),
            directory: "/tmp/project".into(),
            session_ids: vec!["ses_1".into(), "ses_2".into()],
        })
    );
}

#[test]
#[ignore = "porting: components/titlebar-session-events not implemented"]
fn ignores_invalid_removed_session_tab_details() {
    assert_eq!(
        read_session_tabs_removed_detail(&SessionTabsEvent {
            kind: SESSION_TABS_REMOVED_EVENT.into(),
            detail: None,
        }),
        None
    );
    assert_eq!(
        read_session_tabs_removed_detail(&SessionTabsEvent {
            kind: SESSION_TABS_REMOVED_EVENT.into(),
            detail: Some(RawDetail {
                server: None,
                directory: Some("/tmp/project".into()),
                session_ids: Vec::new(),
            }),
        }),
        None
    );
}
