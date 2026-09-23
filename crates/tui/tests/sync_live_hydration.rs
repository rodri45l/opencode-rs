//! Port of packages/tui/test/cli/cmd/tui/sync-live-hydration.test.tsx (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/cli/cmd/tui/sync.tsx; see docs/TEST-PORT.md.
//! The Solid runtime is re-derived as a pure hydration model.

use opencode_tui::sync_hydration::{MessageInfo, PartInfo, SessionData};

const SESSION: &str = "ses_hydration_race";
const MESSAGE: &str = "msg_hydration_race";
const PART: &str = "prt_hydration_race";

fn message(id: &str, created: i64) -> MessageInfo {
    MessageInfo {
        id: id.to_string(),
        session_id: SESSION.to_string(),
        created,
    }
}

fn part(id: &str, message_id: &str, text: &str) -> PartInfo {
    PartInfo {
        id: id.to_string(),
        message_id: message_id.to_string(),
        kind: "text".to_string(),
        text: text.to_string(),
    }
}

#[test]
fn live_messages_use_creation_time_with_an_id_tie_break() {
    let mut data = SessionData::new();
    for info in [
        message("msg_a", 30),
        message("msg_z", 10),
        message("msg_m", 20),
        message("msg_b", 20),
    ] {
        data.message_updated(info);
    }

    assert_eq!(
        data.message_ids(SESSION),
        vec!["msg_z", "msg_b", "msg_m", "msg_a"]
    );
}

#[test]
fn stale_session_hydration_does_not_overwrite_live_message_parts() {
    let mut data = SessionData::new();
    data.message_updated(message(MESSAGE, 1));
    data.part_updated(part(PART, MESSAGE, "visible live content"));
    data.hydrate(
        SESSION,
        vec![(message(MESSAGE, 1), vec![part(PART, MESSAGE, "")])],
    );

    assert_eq!(data.parts[MESSAGE][0].text, "visible live content");
}

#[test]
fn orphan_live_deltas_do_not_suppress_hydrated_parts() {
    let mut data = SessionData::new();
    data.part_delta(MESSAGE, PART, "text", "ignored until part exists");
    data.hydrate(
        SESSION,
        vec![(message(MESSAGE, 1), vec![part(PART, MESSAGE, "hydrated")])],
    );

    assert_eq!(data.parts[MESSAGE][0].text, "hydrated");
}

#[test]
fn hydration_does_not_clear_text_streamed_before_it_starts() {
    let mut data = SessionData::new();
    data.message_updated(message(MESSAGE, 1));
    data.part_updated(part(PART, MESSAGE, ""));
    data.part_delta(MESSAGE, PART, "text", "visible streamed content");
    data.hydrate(
        SESSION,
        vec![(message(MESSAGE, 1), vec![part(PART, MESSAGE, "")])],
    );

    assert_eq!(data.parts[MESSAGE][0].text, "visible streamed content");
}

#[test]
fn live_messages_merged_during_hydration_retain_the_100_message_window() {
    let mut data = SessionData::new();
    data.message_updated(message("msg_z_live", 1));
    let hydrated = (0..100)
        .map(|index| {
            let id = format!("msg_{index:03}");
            (message(&id, 1), vec![part(&format!("prt_{id}"), &id, &id)])
        })
        .collect::<Vec<_>>();
    data.hydrate(SESSION, hydrated);

    let ids = data.message_ids(SESSION);
    assert_eq!(ids.len(), 100);
    assert_eq!(ids.last().unwrap(), "msg_z_live");
    assert!(!ids.contains(&"msg_000".to_string()));
    assert!(!data.parts.contains_key("msg_000"));
}

#[test]
fn a_message_removed_during_hydration_does_not_regain_stale_parts() {
    let mut data = SessionData::new();
    data.message_updated(message(MESSAGE, 1));
    data.message_removed(SESSION, MESSAGE);
    data.hydrate(
        SESSION,
        vec![(message(MESSAGE, 1), vec![part(PART, MESSAGE, "stale")])],
    );

    assert!(data.message_ids(SESSION).is_empty());
    assert!(!data.parts.contains_key(MESSAGE));
}
