//! Session export helpers (port of packages/app/src/utils/session-export.ts).
//!
//! Only the pure filename derivation is modelled; the client-backed transcript
//! fetch stays with the runtime port.

#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub id: String,
    pub title: Option<String>,
    pub slug: Option<String>,
}

pub fn session_export_filename(session: &Session) -> String {
    let name = session
        .title
        .clone()
        .filter(|title| !title.is_empty())
        .or_else(|| session.slug.clone().filter(|slug| !slug.is_empty()))
        .unwrap_or_else(|| session.id.clone());

    let mut clean = String::new();
    let mut previous_dash = false;
    for character in name.to_lowercase().chars() {
        if character.is_ascii_alphanumeric() || character == '_' || character == '-' {
            clean.push(character);
            previous_dash = character == '-';
        } else if !previous_dash {
            clean.push('-');
            previous_dash = true;
        }
    }
    let clean = clean.trim_matches('-');
    if clean.is_empty() {
        format!("{}.json", session.id)
    } else {
        format!("{clean}.json")
    }
}
