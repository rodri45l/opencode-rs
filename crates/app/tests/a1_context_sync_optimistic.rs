//! Port of packages/app/src/context/sync-optimistic.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
struct Message {
    id: String,
    session_id: String,
    created: i64,
}

#[derive(Clone, Debug, PartialEq)]
struct Part {
    id: String,
    session_id: String,
    message_id: String,
    text: String,
}

#[derive(Clone, Debug, PartialEq)]
struct OptimisticAdd {
    session_id: String,
    message: Message,
    parts: Vec<Part>,
}

#[derive(Clone, Debug, PartialEq)]
struct OptimisticRemove {
    session_id: String,
    message_id: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Draft {
    message: BTreeMap<String, Vec<Message>>,
    part: BTreeMap<String, Vec<Part>>,
}

#[derive(Clone, Debug, PartialEq)]
struct PagePart {
    id: String,
    part: Vec<Part>,
}

#[derive(Clone, Debug, PartialEq)]
struct FetchedPage {
    session: Vec<Message>,
    part: Vec<PagePart>,
    complete: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct MergedPage {
    session: Vec<Message>,
    part: Vec<PagePart>,
    confirmed: Vec<String>,
    complete: bool,
}

// Local stubs (fast wave): real module lands later.
fn apply_optimistic_add(_draft: &mut Draft, _add: OptimisticAdd) {}

fn apply_optimistic_remove(_draft: &mut Draft, _remove: OptimisticRemove) {}

fn merge_optimistic_page(_page: FetchedPage, _optimistic: &[OptimisticAdd]) -> MergedPage {
    MergedPage {
        session: Vec::new(),
        part: Vec::new(),
        confirmed: Vec::new(),
        complete: false,
    }
}

fn user_message(id: &str, session_id: &str, created: i64) -> Message {
    Message {
        id: id.into(),
        session_id: session_id.into(),
        created,
    }
}

fn text_part(id: &str, session_id: &str, message_id: &str) -> Part {
    Part {
        id: id.into(),
        session_id: session_id.into(),
        message_id: message_id.into(),
        text: id.into(),
    }
}

#[test]
#[ignore = "porting: context/sync-optimistic not implemented"]
fn apply_optimistic_add_inserts_by_creation_time() {
    let session_id = "ses_1";
    let mut draft = Draft::default();
    draft.message.insert(
        session_id.into(),
        vec![user_message("msg_z", session_id, 1)],
    );

    apply_optimistic_add(
        &mut draft,
        OptimisticAdd {
            session_id: session_id.into(),
            message: user_message("msg_a", session_id, 2),
            parts: vec![
                text_part("prt_2", session_id, "msg_a"),
                text_part("prt_1", session_id, "msg_a"),
            ],
        },
    );

    assert_eq!(
        draft.message[session_id]
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>(),
        vec!["msg_z".to_string(), "msg_a".to_string()]
    );
    assert_eq!(
        draft.part["msg_a"]
            .iter()
            .map(|p| p.id.clone())
            .collect::<Vec<_>>(),
        vec!["prt_1".to_string(), "prt_2".to_string()]
    );
}

#[test]
#[ignore = "porting: context/sync-optimistic not implemented"]
fn apply_optimistic_remove_removes_message_and_part_entries() {
    let session_id = "ses_1";
    let mut draft = Draft::default();
    draft.message.insert(
        session_id.into(),
        vec![
            user_message("msg_1", session_id, 1),
            user_message("msg_2", session_id, 1),
        ],
    );
    draft.part.insert(
        "msg_1".into(),
        vec![text_part("prt_1", session_id, "msg_1")],
    );
    draft.part.insert(
        "msg_2".into(),
        vec![text_part("prt_2", session_id, "msg_2")],
    );

    apply_optimistic_remove(
        &mut draft,
        OptimisticRemove {
            session_id: session_id.into(),
            message_id: "msg_1".into(),
        },
    );

    assert_eq!(
        draft.message[session_id]
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>(),
        vec!["msg_2".to_string()]
    );
    assert!(!draft.part.contains_key("msg_1"));
    assert_eq!(draft.part["msg_2"].len(), 1);
}

#[test]
#[ignore = "porting: context/sync-optimistic not implemented"]
fn merge_optimistic_page_keeps_pending_messages_in_fetched_timelines() {
    let session_id = "ses_1";
    let page = merge_optimistic_page(
        FetchedPage {
            session: vec![user_message("msg_z", session_id, 1)],
            part: vec![PagePart {
                id: "msg_z".into(),
                part: vec![text_part("prt_1", session_id, "msg_z")],
            }],
            complete: true,
        },
        &[OptimisticAdd {
            session_id: session_id.into(),
            message: user_message("msg_a", session_id, 2),
            parts: vec![text_part("prt_2", session_id, "msg_a")],
        }],
    );

    assert_eq!(
        page.session
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>(),
        vec!["msg_z".to_string(), "msg_a".to_string()]
    );
    assert_eq!(
        page.part.iter().find(|p| p.id == "msg_a").map(|p| p
            .part
            .iter()
            .map(|x| x.id.clone())
            .collect::<Vec<_>>()),
        Some(vec!["prt_2".to_string()])
    );
    assert!(page.confirmed.is_empty());
    assert!(page.complete);
}

#[test]
#[ignore = "porting: context/sync-optimistic not implemented"]
fn merge_optimistic_page_uses_ids_only_to_break_equal_time_ties() {
    let session_id = "ses_1";
    let page = merge_optimistic_page(
        FetchedPage {
            session: vec![user_message("msg_z", session_id, 1)],
            part: Vec::new(),
            complete: true,
        },
        &[OptimisticAdd {
            session_id: session_id.into(),
            message: user_message("msg_a", session_id, 1),
            parts: Vec::new(),
        }],
    );
    assert_eq!(
        page.session
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>(),
        vec!["msg_a".to_string(), "msg_z".to_string()]
    );
}

#[test]
#[ignore = "porting: context/sync-optimistic not implemented"]
fn merge_optimistic_page_keeps_missing_optimistic_parts_until_the_server_has_them() {
    let session_id = "ses_1";
    let page = merge_optimistic_page(
        FetchedPage {
            session: vec![user_message("msg_2", session_id, 1)],
            part: vec![PagePart {
                id: "msg_2".into(),
                part: vec![text_part("prt_2", session_id, "msg_2")],
            }],
            complete: true,
        },
        &[OptimisticAdd {
            session_id: session_id.into(),
            message: user_message("msg_2", session_id, 1),
            parts: vec![
                text_part("prt_1", session_id, "msg_2"),
                text_part("prt_2", session_id, "msg_2"),
            ],
        }],
    );
    assert_eq!(
        page.part.iter().find(|p| p.id == "msg_2").map(|p| p
            .part
            .iter()
            .map(|x| x.id.clone())
            .collect::<Vec<_>>()),
        Some(vec!["prt_1".to_string(), "prt_2".to_string()])
    );
    assert!(page.confirmed.is_empty());
}

#[test]
#[ignore = "porting: context/sync-optimistic not implemented"]
fn merge_optimistic_page_confirms_echoed_messages_once_all_parts_arrive() {
    let session_id = "ses_1";
    let mut server_part = text_part("prt_1", session_id, "msg_2");
    server_part.text = "server".into();
    let page = merge_optimistic_page(
        FetchedPage {
            session: vec![user_message("msg_2", session_id, 1)],
            part: vec![PagePart {
                id: "msg_2".into(),
                part: vec![server_part, text_part("prt_2", session_id, "msg_2")],
            }],
            complete: true,
        },
        &[OptimisticAdd {
            session_id: session_id.into(),
            message: user_message("msg_2", session_id, 1),
            parts: vec![
                text_part("prt_1", session_id, "msg_2"),
                text_part("prt_2", session_id, "msg_2"),
            ],
        }],
    );

    assert_eq!(page.confirmed, vec!["msg_2".to_string()]);
    assert_eq!(
        page.part
            .iter()
            .find(|p| p.id == "msg_2")
            .map(|p| p.part.clone()),
        Some(vec![
            Part {
                id: "prt_1".into(),
                session_id: session_id.into(),
                message_id: "msg_2".into(),
                text: "server".into()
            },
            text_part("prt_2", session_id, "msg_2")
        ])
    );
}
