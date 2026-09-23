//! Server scoping keys (port of packages/app/src/utils/server-scope.ts).

use std::collections::BTreeMap;

const SEPARATOR: char = '\u{0000}';

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionRouteKey(pub String);

fn fragment(label: &str, value: &str) -> Result<String, String> {
    if value.contains(SEPARATOR) {
        Err(format!("{label} cannot contain null bytes"))
    } else {
        Ok(value.to_string())
    }
}

pub fn scope_from_server_key(key: &str, canonical_local_server: Option<&str>) -> String {
    if key == "sidecar" || canonical_local_server == Some(key) {
        "local".to_string()
    } else {
        key.to_string()
    }
}

pub fn route_from_route(dir: &str, session_id: &str) -> SessionRouteKey {
    if session_id.is_empty() {
        SessionRouteKey(dir.to_string())
    } else {
        SessionRouteKey(format!("{dir}/{session_id}"))
    }
}

pub fn session_state_key(scope: &str, route: &SessionRouteKey) -> String {
    format!("{scope}{SEPARATOR}{}", route.0)
}

pub fn session_state_route(key: &str) -> String {
    match key.rfind(SEPARATOR) {
        Some(index) => key[index + 1..].to_string(),
        None => key.to_string(),
    }
}

pub fn scoped_key_from(scope: &str, part: &str) -> Result<String, String> {
    let scope = fragment("Server scope", scope)?;
    let part = fragment("Scoped key part", part)?;
    Ok(format!("{scope}{SEPARATOR}{part}"))
}

pub fn migrate_legacy_session_state_keys(
    input: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut scoped: BTreeMap<String, String> = input
        .iter()
        .filter(|(key, _)| key.contains(SEPARATOR))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    for (key, item) in &input {
        if key.contains(SEPARATOR) {
            continue;
        }
        let next = session_state_key("local", &SessionRouteKey(key.clone()));
        scoped.entry(next).or_insert_with(|| item.clone());
    }
    scoped
}
