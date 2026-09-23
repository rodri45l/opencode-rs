//! Live session hydration model.
//!
//! Port of packages/tui/src/cli/cmd/tui/sync-live-hydration.test.tsx behaviour
//! (upstream 18ef3cc), re-derived from the Solid runtime as a pure reducer.

use std::collections::{HashMap, HashSet};

/// The message window size.
pub const MAX_MESSAGES: usize = 100;

/// A session message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageInfo {
    pub id: String,
    pub session_id: String,
    pub created: i64,
}

/// A message part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartInfo {
    pub id: String,
    pub message_id: String,
    pub kind: String,
    pub text: String,
}

/// Live session state with hydration merging.
#[derive(Debug, Clone, Default)]
pub struct SessionData {
    pub messages: HashMap<String, Vec<MessageInfo>>,
    pub parts: HashMap<String, Vec<PartInfo>>,
    removed: HashMap<String, HashSet<String>>,
}

impl SessionData {
    /// Create empty session data.
    pub fn new() -> Self {
        SessionData::default()
    }

    /// Apply a live `message.updated` event.
    pub fn message_updated(&mut self, info: MessageInfo) {
        let session_id = info.session_id.clone();
        let list = self.messages.entry(session_id.clone()).or_default();
        match list.iter_mut().find(|message| message.id == info.id) {
            Some(existing) => *existing = info,
            None => list.push(info),
        }
        sort_and_trim(&mut self.messages, &mut self.parts, &session_id);
    }

    /// Apply a live `message.removed` event.
    pub fn message_removed(&mut self, session_id: &str, message_id: &str) {
        if let Some(list) = self.messages.get_mut(session_id) {
            list.retain(|message| message.id != message_id);
        }
        self.parts.remove(message_id);
        self.removed
            .entry(session_id.to_string())
            .or_default()
            .insert(message_id.to_string());
    }

    /// Apply a live `message.part.updated` event.
    pub fn part_updated(&mut self, part: PartInfo) {
        let list = self.parts.entry(part.message_id.clone()).or_default();
        match list.iter_mut().find(|existing| existing.id == part.id) {
            Some(existing) => *existing = part,
            None => list.push(part),
        }
    }

    /// Apply a live `message.part.delta` event, ignoring orphans.
    pub fn part_delta(&mut self, message_id: &str, part_id: &str, field: &str, delta: &str) {
        if field != "text" {
            return;
        }
        if let Some(list) = self.parts.get_mut(message_id) {
            if let Some(part) = list.iter_mut().find(|part| part.id == part_id) {
                part.text.push_str(delta);
            }
        }
    }

    /// Merge a hydrated snapshot, preserving live messages and parts.
    pub fn hydrate(&mut self, session_id: &str, hydrated: Vec<(MessageInfo, Vec<PartInfo>)>) {
        let removed = self.removed.remove(session_id).unwrap_or_default();
        for (info, parts) in hydrated {
            if removed.contains(&info.id) {
                continue;
            }
            let message_id = info.id.clone();
            let list = self.messages.entry(session_id.to_string()).or_default();
            if !list.iter().any(|message| message.id == message_id) {
                list.push(info);
            }
            let existing = self.parts.entry(message_id).or_default();
            for part in parts {
                if !existing.iter().any(|item| item.id == part.id) {
                    existing.push(part);
                }
            }
        }
        sort_and_trim(&mut self.messages, &mut self.parts, session_id);
    }

    /// The ordered message ids for a session.
    pub fn message_ids(&self, session_id: &str) -> Vec<String> {
        self.messages
            .get(session_id)
            .map(|list| list.iter().map(|message| message.id.clone()).collect())
            .unwrap_or_default()
    }
}

fn sort_and_trim(
    messages: &mut HashMap<String, Vec<MessageInfo>>,
    parts: &mut HashMap<String, Vec<PartInfo>>,
    session_id: &str,
) {
    let Some(list) = messages.get_mut(session_id) else {
        return;
    };
    list.sort_by(|left, right| {
        left.created
            .cmp(&right.created)
            .then_with(|| left.id.cmp(&right.id))
    });
    while list.len() > MAX_MESSAGES {
        let dropped = list.remove(0);
        parts.remove(&dropped.id);
    }
}
