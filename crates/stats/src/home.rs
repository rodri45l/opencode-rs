//! Home-page retention aggregation.
//!
//! Derived from the observable behaviour pinned by
//! `packages/stats/core/src/domain/home.ts` (upstream 18ef3cc).

use std::collections::BTreeMap;

/// The number of most-recent weekly cohorts pooled into a retention rate.
const RETENTION_COHORT_WEEKS: usize = 7;
/// The minimum pooled eligible user-weeks for a model to receive a rank.
const RETENTION_MIN_ELIGIBLE_USER_WEEKS: i64 = 100;

/// A single weekly retention cohort row as stored in `model_retention`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionMetricRow {
    pub cohort_date: String,
    pub updated_at: i64,
    pub provider: String,
    pub model: String,
    pub eligible_users: i64,
    pub retained_users: i64,
}

/// A pooled retention entry with its derived rate and rank.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionEntry {
    pub model: String,
    pub provider: String,
    pub author: String,
    pub rate: f64,
    pub eligible_user_weeks: i64,
    pub retained_user_weeks: i64,
    pub rank: Option<usize>,
}

fn format_provider(provider: &str) -> String {
    const KNOWN: &[(&str, &str)] = &[
        ("anthropic", "Anthropic"),
        ("deepseek", "DeepSeek"),
        ("google", "Google"),
        ("minimax", "MiniMax"),
        ("meta", "Meta"),
        ("moonshot", "Moonshot"),
        ("moonshotai", "Moonshot"),
        ("nvidia", "NVIDIA"),
        ("opencode", "opencode"),
        ("openai", "OpenAI"),
        ("qwen", "Qwen"),
        ("tencent", "Tencent"),
        ("xai", "xAI"),
        ("xiaomi", "Xiaomi"),
        ("zhipu", "Zhipu"),
        ("zhipuai", "Zhipu"),
    ];
    let normalized: String = provider
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    if let Some((_, label)) = KNOWN.iter().find(|(key, _)| *key == normalized) {
        return (*label).to_string();
    }
    provider
        .replace(['-', '_'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn round(value: f64, digits: i32) -> f64 {
    let factor = 10f64.powi(digits);
    (value * factor).round() / factor
}

/// Pool the latest seven weekly cohorts per model and rank models above the
/// sample floor by retention rate.
pub fn build_retention_entries(rows: &[RetentionMetricRow]) -> Vec<RetentionEntry> {
    let mut cohort_dates: Vec<&str> = rows.iter().map(|row| row.cohort_date.as_str()).collect();
    cohort_dates.sort_unstable();
    cohort_dates.dedup();
    let keep: Vec<&str> = cohort_dates
        .into_iter()
        .rev()
        .take(RETENTION_COHORT_WEEKS)
        .collect();

    struct Aggregate {
        provider: String,
        eligible: i64,
        retained: i64,
    }
    let mut aggregate: BTreeMap<String, Aggregate> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for row in rows {
        if !keep.contains(&row.cohort_date.as_str()) {
            continue;
        }
        let entry = aggregate.entry(row.model.clone()).or_insert_with(|| {
            order.push(row.model.clone());
            Aggregate {
                provider: row.provider.clone(),
                eligible: 0,
                retained: 0,
            }
        });
        entry.eligible += row.eligible_users;
        entry.retained += row.retained_users;
    }

    let mut entries: Vec<RetentionEntry> = order
        .into_iter()
        .map(|model| {
            let item = &aggregate[&model];
            RetentionEntry {
                model: model.clone(),
                provider: item.provider.clone(),
                author: format_provider(&item.provider),
                rate: if item.eligible > 0 {
                    round((item.retained as f64 / item.eligible as f64) * 100.0, 1)
                } else {
                    0.0
                },
                eligible_user_weeks: item.eligible,
                retained_user_weeks: item.retained,
                rank: None,
            }
        })
        .collect();

    let mut ranked: Vec<&RetentionEntry> = entries
        .iter()
        .filter(|entry| entry.eligible_user_weeks >= RETENTION_MIN_ELIGIBLE_USER_WEEKS)
        .collect();
    ranked.sort_by(|a, b| {
        b.rate
            .partial_cmp(&a.rate)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.eligible_user_weeks.cmp(&a.eligible_user_weeks))
            .then_with(|| a.model.cmp(&b.model))
    });
    let mut ranks: BTreeMap<String, usize> = BTreeMap::new();
    for (index, entry) in ranked.iter().enumerate() {
        ranks.insert(entry.model.clone(), index + 1);
    }
    for entry in entries.iter_mut() {
        entry.rank = ranks.get(&entry.model).copied();
    }

    entries.sort_by_key(|entry| entry.rank.unwrap_or(usize::MAX));
    entries
}
