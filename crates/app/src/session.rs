//! Session projection (port of packages/app/src/utils/session.ts).

#[derive(Clone, Debug, PartialEq)]
pub struct Tokens {
    pub input: i64,
    pub output: i64,
    pub reasoning: i64,
    pub cache_read: i64,
    pub cache_write: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub id: String,
    pub provider_id: String,
    pub variant: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SessionInfo {
    pub id: String,
    pub parent_id: Option<String>,
    pub project_id: String,
    pub workspace_id: Option<String>,
    pub directory: String,
    pub subpath: Option<String>,
    pub agent: Option<String>,
    pub model: Option<Model>,
    pub cost: f64,
    pub tokens: Tokens,
    pub title: Option<String>,
    pub created: i64,
    pub updated: i64,
    pub archived: Option<i64>,
    pub revert: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppSession {
    pub id: String,
    pub slug: String,
    pub project_id: String,
    pub workspace_id: Option<String>,
    pub directory: String,
    pub path: Option<String>,
    pub parent_id: Option<String>,
    pub cost: f64,
    pub tokens: Tokens,
    pub title: String,
    pub agent: Option<String>,
    pub model: Option<Model>,
    pub version: String,
    pub created: i64,
    pub updated: i64,
    pub revert: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Page {
    pub data: Vec<SessionInfo>,
    pub next: Option<String>,
}

pub fn zero_tokens() -> Tokens {
    Tokens {
        input: 0,
        output: 0,
        reasoning: 0,
        cache_read: 0,
        cache_write: 0,
    }
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

fn iso_from_millis(millis: i64) -> String {
    let seconds = millis.div_euclid(1000);
    let ms = millis.rem_euclid(1000);
    let days = seconds.div_euclid(86_400);
    let secs_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{ms:03}Z")
}

fn with_timestamped_fallback(info: &SessionInfo) -> String {
    match &info.title {
        Some(title) => title.clone(),
        None => {
            let prefix = if info.parent_id.is_some() {
                "Child"
            } else {
                "New"
            };
            format!("{prefix} session - {}", iso_from_millis(info.created))
        }
    }
}

pub fn normalize_session_info(info: &SessionInfo) -> AppSession {
    AppSession {
        id: info.id.clone(),
        slug: info.id.clone(),
        project_id: info.project_id.clone(),
        workspace_id: info.workspace_id.clone(),
        directory: info.directory.clone(),
        path: info.subpath.clone(),
        parent_id: info.parent_id.clone(),
        cost: info.cost,
        tokens: info.tokens.clone(),
        title: with_timestamped_fallback(info),
        agent: info.agent.clone(),
        model: info.model.clone(),
        version: String::new(),
        created: info.created,
        updated: info.updated,
        revert: info.revert.clone(),
    }
}

pub fn list_all_sessions(pages: &[Page], _query: &str) -> Vec<SessionInfo> {
    pages
        .iter()
        .flat_map(|page| page.data.iter().cloned())
        .collect()
}
