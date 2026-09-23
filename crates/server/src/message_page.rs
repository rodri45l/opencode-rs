//! Message cursor pagination and compaction-boundary filtering.
//!
//! Ports the observable behaviour of `MessageV2.cursor`, `MessageV2.page` and
//! `MessageV2.filterCompacted` from
//! `packages/opencode/src/session/message-v2.ts`.

use base64::Engine;
use serde_json::json;

/// A message role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// User message.
    User,
    /// Assistant message.
    Assistant,
}

/// A message part relevant to compaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    /// A plain text part.
    Text,
    /// A compaction marker.
    Compaction {
        /// Whether compaction was automatic.
        auto: bool,
        /// The retained tail's start message id.
        tail_start_id: Option<String>,
    },
}

/// Message metadata.
#[derive(Debug, Clone)]
pub struct Info {
    /// Message id.
    pub id: String,
    /// Message role.
    pub role: Role,
    /// Creation time.
    pub created: f64,
    /// Finish reason.
    pub finish: Option<String>,
    /// Whether this assistant message is a compaction summary.
    pub summary: bool,
    /// Whether this assistant message errored.
    pub has_error: bool,
    /// Parent user message id.
    pub parent_id: Option<String>,
}

/// A message together with its parts.
#[derive(Debug, Clone)]
pub struct WithParts {
    /// Message metadata.
    pub info: Info,
    /// Message parts.
    pub parts: Vec<Part>,
}

/// A page item.
#[derive(Debug, Clone, PartialEq)]
pub struct PageItem {
    /// Message id.
    pub id: String,
    /// Creation time.
    pub created: f64,
}

/// A page of messages.
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    /// Chronological page items.
    pub items: Vec<PageItem>,
    /// Whether more items remain.
    pub more: bool,
    /// Cursor for the next page.
    pub cursor: Option<String>,
}

/// Encode an `{ id, time }` cursor as base64url JSON.
pub fn cursor_encode(id: &str, time: f64) -> String {
    let payload = json!({ "id": id, "time": time }).to_string();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload)
}

/// Decode a base64url `{ id, time }` cursor.
pub fn cursor_decode(encoded: &str) -> (String, f64) {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .expect("valid base64url cursor");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("valid cursor json");
    let id = value
        .get("id")
        .and_then(serde_json::Value::as_str)
        .expect("cursor id");
    let time = value
        .get("time")
        .and_then(serde_json::Value::as_f64)
        .expect("cursor time");
    (id.to_string(), time)
}

/// Page backwards through messages, newest first.
pub fn page(items: &[PageItem], limit: usize, before: Option<&str>) -> Page {
    let before = before.map(cursor_decode);

    let mut rows: Vec<PageItem> = items
        .iter()
        .filter(|item| match &before {
            Some((id, time)) => item.created < *time || (item.created == *time && item.id < *id),
            None => true,
        })
        .cloned()
        .collect();
    rows.sort_by(|a, b| {
        b.created
            .partial_cmp(&a.created)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.id.cmp(&a.id))
    });

    let more = rows.len() > limit;
    let mut slice: Vec<PageItem> = rows.into_iter().take(limit).collect();
    let cursor = if more {
        slice
            .last()
            .map(|tail| cursor_encode(&tail.id, tail.created))
    } else {
        None
    };
    slice.reverse();

    Page {
        items: slice,
        more,
        cursor,
    }
}

/// Reorder messages for model consumption across a compaction boundary.
pub fn filter_compacted(msgs: &[WithParts]) -> Vec<WithParts> {
    let mut result: Vec<WithParts> = Vec::new();
    let mut completed: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut retain: Option<String> = None;

    for msg in msgs {
        result.push(msg.clone());
        if let Some(target) = &retain {
            if msg.info.id == *target {
                break;
            }
            continue;
        }
        if msg.info.role == Role::User && completed.contains(&msg.info.id) {
            let compaction_part = msg.parts.iter().find_map(|part| match part {
                Part::Compaction { tail_start_id, .. } => Some(tail_start_id.clone()),
                Part::Text => None,
            });
            let Some(tail) = compaction_part else {
                continue;
            };
            match tail {
                None => break,
                Some(tail) => {
                    retain = Some(tail.clone());
                    if msg.info.id == tail {
                        break;
                    }
                    continue;
                }
            }
        }
        if msg.info.role == Role::Assistant
            && msg.info.summary
            && msg.info.finish.is_some()
            && !msg.info.has_error
        {
            if let Some(parent) = &msg.info.parent_id {
                completed.insert(parent.clone());
            }
        }
    }

    result.reverse();

    let compaction_index = result.iter().rposition(|msg| {
        msg.info.role == Role::User
            && msg.parts.iter().any(|part| {
                matches!(
                    part,
                    Part::Compaction {
                        tail_start_id: Some(_),
                        ..
                    }
                )
            })
    });
    let Some(compaction_index) = compaction_index else {
        return result;
    };
    let compaction = &result[compaction_index];
    let tail_start = compaction.parts.iter().find_map(|part| match part {
        Part::Compaction {
            tail_start_id: Some(id),
            ..
        } => Some(id.clone()),
        _ => None,
    });
    let summary_index = result.iter().enumerate().position(|(index, msg)| {
        index > compaction_index
            && msg.info.role == Role::Assistant
            && msg.info.summary
            && msg.info.parent_id.as_deref() == Some(compaction.info.id.as_str())
    });
    let Some(summary_index) = summary_index else {
        return result;
    };
    let tail_index = tail_start
        .as_ref()
        .and_then(|id| result.iter().position(|msg| msg.info.id == *id));

    if let Some(tail_index) = tail_index {
        if tail_index < compaction_index && summary_index > compaction_index {
            let mut reordered = Vec::new();
            reordered.extend(result[compaction_index..=summary_index].iter().cloned());
            reordered.extend(result[tail_index..compaction_index].iter().cloned());
            reordered.extend(result[summary_index + 1..].iter().cloned());
            return reordered;
        }
    }
    result
}
