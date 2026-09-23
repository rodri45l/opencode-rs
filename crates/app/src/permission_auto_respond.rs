//! Permission auto-respond lineage resolution.
//!
//! Port of `packages/app/src/context/permission-auto-respond.ts` (upstream
//! 18ef3cc): a permission is auto-accepted when the session or one of its
//! ancestors has an override, falling back to the directory-level override.

use std::collections::BTreeMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

/// A session lineage node.
#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub id: String,
    pub parent_id: Option<String>,
}

/// The auto-accept override map.
pub type AutoAccept = BTreeMap<String, bool>;

fn encode(directory: &str) -> String {
    URL_SAFE_NO_PAD.encode(directory.as_bytes())
}

/// The directory-scoped key for a session override.
pub fn accept_key(session_id: &str, directory: Option<&str>) -> String {
    match directory {
        Some(directory) if !directory.is_empty() => format!("{}/{}", encode(directory), session_id),
        _ => session_id.to_string(),
    }
}

/// The directory-level override key.
pub fn directory_accept_key(directory: &str) -> String {
    format!("{}/*", encode(directory))
}

fn accepted(auto_accept: &AutoAccept, session_id: &str, directory: &str) -> Option<bool> {
    let key = accept_key(session_id, Some(directory));
    auto_accept
        .get(&key)
        .copied()
        .or_else(|| auto_accept.get(session_id).copied())
}

/// Whether the directory has an enabled auto-accept override.
pub fn is_directory_auto_accepting(auto_accept: &AutoAccept, directory: &str) -> bool {
    auto_accept
        .get(&directory_accept_key(directory))
        .copied()
        .unwrap_or(false)
}

fn session_lineage(sessions: &[Session], session_id: &str) -> Vec<String> {
    let mut parent: BTreeMap<&str, &str> = BTreeMap::new();
    for session in sessions {
        if let Some(parent_id) = &session.parent_id {
            parent.insert(&session.id, parent_id);
        }
    }

    let mut seen: Vec<String> = vec![session_id.to_string()];
    let mut ids: Vec<String> = vec![session_id.to_string()];
    let mut index = 0;
    while index < ids.len() {
        let id = ids[index].clone();
        index += 1;
        let Some(parent_id) = parent.get(id.as_str()) else {
            continue;
        };
        if seen.iter().any(|seen| seen == parent_id) {
            continue;
        }
        seen.push((*parent_id).to_string());
        ids.push((*parent_id).to_string());
    }
    ids
}

/// The nearest lineage override for a session, if any.
pub fn session_auto_accept(
    auto_accept: &AutoAccept,
    sessions: &[Session],
    session_id: &str,
    directory: &str,
) -> Option<bool> {
    session_lineage(sessions, session_id)
        .into_iter()
        .find_map(|id| accepted(auto_accept, &id, directory))
}

/// Whether a permission should be auto-accepted.
pub fn auto_responds_permission(
    auto_accept: &AutoAccept,
    sessions: &[Session],
    session_id: &str,
    directory: &str,
) -> bool {
    match session_auto_accept(auto_accept, sessions, session_id, directory) {
        Some(value) => value,
        None if !directory.is_empty() => is_directory_auto_accepting(auto_accept, directory),
        None => false,
    }
}
