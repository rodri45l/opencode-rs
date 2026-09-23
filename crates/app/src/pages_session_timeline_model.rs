//! Session timeline model helpers.
//!
//! Port of `packages/app/src/pages/session/timeline/model.ts` (upstream 18ef3cc).

/// A timeline message.
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub id: String,
    pub role: String,
}

/// Select user messages from a chronological message list.
pub fn select_user_messages(messages: &[Message]) -> Vec<Message> {
    messages
        .iter()
        .filter(|message| message.role == "user")
        .cloned()
        .collect()
}

/// Select the user messages visible before a revert boundary.
pub fn select_visible_user_messages(messages: &[Message], revert: Option<&str>) -> Vec<Message> {
    let Some(revert) = revert else {
        return messages.to_vec();
    };
    match messages.iter().position(|message| message.id == revert) {
        Some(boundary) => messages[..boundary].to_vec(),
        None => messages.to_vec(),
    }
}

/// Whether the timeline has enough data to render.
pub fn is_timeline_ready(messages: &[Message], loading: bool) -> bool {
    messages.iter().any(|message| message.role == "user") || !loading
}

/// The outcome of loading an older timeline page.
#[derive(Clone, Debug, PartialEq)]
pub struct LoadOutcome {
    pub calls: usize,
    pub anchors: Vec<String>,
    pub restored: usize,
}

/// Load one opaque cursor page of older history.
pub fn load_older_timeline(
    session_id_before: &str,
    session_id_after: &str,
    more: bool,
    loading: bool,
    fails: bool,
) -> LoadOutcome {
    let mut outcome = LoadOutcome {
        calls: 0,
        anchors: Vec::new(),
        restored: 0,
    };
    if session_id_before.is_empty() || !more || loading {
        return outcome;
    }
    outcome.anchors.push("before".to_string());
    outcome.calls += 1;

    if fails {
        if session_id_before == session_id_after {
            outcome.anchors.push("after".to_string());
            outcome.anchors.push("true".to_string());
            outcome.restored = 1;
        }
        return outcome;
    }

    if session_id_before != session_id_after {
        return outcome;
    }
    outcome.anchors.push("after".to_string());
    outcome.anchors.push("true".to_string());
    outcome.restored = 1;
    outcome
}
