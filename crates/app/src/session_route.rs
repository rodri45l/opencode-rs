//! Session route helpers (port of packages/app/src/utils/session-route.ts).

use std::collections::{HashMap, HashSet};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

#[derive(Clone, Debug, PartialEq)]
pub struct Tab {
    pub kind: String,
    pub server: String,
    pub session_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub id: String,
    pub parent_id: Option<String>,
}

fn encode_b64url(value: &str) -> String {
    URL_SAFE_NO_PAD.encode(value.as_bytes())
}

fn decode_b64url(segment: &str) -> Option<String> {
    let bytes = URL_SAFE_NO_PAD.decode(segment).ok()?;
    String::from_utf8(bytes).ok()
}

pub fn session_href(server: &str, session_id: &str) -> String {
    format!("/server/{}/session/{}", encode_b64url(server), session_id)
}

pub fn legacy_session_href(directory: &str, session_id: &str) -> String {
    format!("/{}/session/{}", encode_b64url(directory), session_id)
}

pub fn require_server_key(segment: &str) -> Result<String, String> {
    match decode_b64url(segment) {
        Some(key) if encode_b64url(&key) == segment => Ok(key),
        _ => Err("Invalid server route".to_string()),
    }
}

pub fn legacy_session_server(tabs: &[Tab], session_id: &str, active: &str) -> String {
    let matches: Vec<&Tab> = tabs
        .iter()
        .filter(|tab| tab.session_id == session_id)
        .collect();
    if let Some(tab) = matches.iter().find(|tab| tab.server == active) {
        return tab.server.clone();
    }
    if matches.len() == 1 {
        return matches[0].server.clone();
    }
    active.to_string()
}

pub fn root_session(
    session: Session,
    sessions: &HashMap<String, Session>,
) -> Result<Session, String> {
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert(session.id.clone());
    let mut current = session;
    while let Some(parent_id) = current.parent_id.clone() {
        if seen.contains(&parent_id) {
            return Err(format!("Session parent cycle: {parent_id}"));
        }
        seen.insert(parent_id.clone());
        current = sessions
            .get(&parent_id)
            .cloned()
            .ok_or_else(|| format!("Missing session: {parent_id}"))?;
    }
    Ok(current)
}
