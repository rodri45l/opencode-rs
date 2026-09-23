//! Inference statistics query builders and result mappers.
//!
//! Derived from the observable behaviour pinned by
//! `packages/stats/core/src/domain/inference.ts` (upstream 18ef3cc). The
//! normalization and aggregate mappers are ported green; the exact SQL string
//! builders are red-first until the R2 query surface lands.

use crate::model_normalization::{stat_model, stat_provider};
use crate::r2_sql::R2SqlData;
use crate::StatsError;

/// A query source (namespace/table/dataset) for the stats warehouse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatsQuerySource {
    pub namespace: String,
    pub table: String,
    pub dataset: String,
}

/// A retention query paired with the cohort dates it covers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionQuery {
    pub cohort_dates: Vec<String>,
    pub query: String,
}

/// Aggregated usage for one model.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelAggregate {
    pub grain: String,
    pub period_key: String,
    pub dataset: String,
    pub tier: String,
    pub provider: String,
    pub model: String,
    pub provider_model: String,
    pub sessions: i64,
    pub requests: i64,
    pub unique_users: i64,
    pub total_tokens: i64,
}

/// Aggregated usage for one provider.
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderAggregate {
    pub grain: String,
    pub period_key: String,
    pub dataset: String,
    pub tier: String,
    pub provider: String,
    pub requests: i64,
}

/// Aggregated usage for one geo dimension.
#[derive(Debug, Clone, PartialEq)]
pub struct GeoAggregate {
    pub grain: String,
    pub period_key: String,
    pub dataset: String,
    pub tier: String,
    pub provider: String,
    pub model: String,
    pub country: String,
    pub continent: String,
    pub requests: i64,
}

/// A mapped retention aggregate row.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAggregate {
    pub cohort_date: String,
    pub dataset: String,
    pub tier: String,
    pub provider: String,
    pub model: String,
    pub eligible_users: i64,
    pub retained_users: i64,
}

struct BaseAggregate {
    grain: String,
    period_key: String,
    dataset: String,
    tier: String,
    requests: i64,
}

fn integer(data: &R2SqlData, key: &str) -> i64 {
    number(data, key).round() as i64
}

fn number(data: &R2SqlData, key: &str) -> f64 {
    data.get(key)
        .map(|value| value.parse::<f64>().unwrap_or(0.0))
        .filter(|value| value.is_finite())
        .unwrap_or(0.0)
}

fn base_aggregate(data: &R2SqlData) -> Option<BaseAggregate> {
    let grain = data.get("grain").map(String::as_str).unwrap_or("");
    if grain != "day" && grain != "week" {
        return None;
    }
    let period_key = data.get("period_key")?.clone();
    if period_key.is_empty() {
        return None;
    }
    Some(BaseAggregate {
        grain: grain.to_string(),
        period_key,
        dataset: data.get("dataset").cloned().unwrap_or_default(),
        tier: data
            .get("tier")
            .cloned()
            .unwrap_or_else(|| "unknown".into()),
        requests: integer(data, "requests"),
    })
}

/// Map an R2 row into a model aggregate, dropping excluded models.
pub fn to_model_aggregate(data: &R2SqlData) -> Vec<ModelAggregate> {
    let model = stat_model(
        data.get("model").map(String::as_str),
        data.get("provider_model").map(String::as_str),
    );
    let provider = match stat_provider(
        Some(model.as_str()),
        data.get("provider_model").map(String::as_str),
        data.get("provider").map(String::as_str),
    ) {
        Some(provider) => provider,
        None => return Vec::new(),
    };
    base_aggregate(data)
        .into_iter()
        .map(|base| ModelAggregate {
            grain: base.grain,
            period_key: base.period_key,
            dataset: base.dataset,
            tier: base.tier,
            provider: provider.clone(),
            model: model.clone(),
            provider_model: data.get("provider_model").cloned().unwrap_or_default(),
            sessions: integer(data, "sessions"),
            requests: base.requests,
            unique_users: integer(data, "unique_users"),
            total_tokens: integer(data, "total_tokens"),
        })
        .collect()
}

/// Map an R2 row into a provider aggregate.
pub fn to_provider_aggregate(data: &R2SqlData) -> Vec<ProviderAggregate> {
    let provider = stat_provider(
        data.get("model").map(String::as_str),
        data.get("provider_model").map(String::as_str),
        data.get("provider").map(String::as_str),
    )
    .unwrap_or_else(|| "unknown".to_string());
    base_aggregate(data)
        .into_iter()
        .map(|base| ProviderAggregate {
            grain: base.grain,
            period_key: base.period_key,
            dataset: base.dataset,
            tier: base.tier,
            provider: provider.clone(),
            requests: base.requests,
        })
        .collect()
}

fn normalize_country(value: Option<&String>) -> String {
    let raw = value.map(String::as_str).unwrap_or("").trim();
    if raw.is_empty() {
        "ZZ".to_string()
    } else {
        raw.to_uppercase()
    }
}

/// Map an R2 row into a geo aggregate.
pub fn to_geo_aggregate(data: &R2SqlData) -> Vec<GeoAggregate> {
    let provider = stat_provider(
        data.get("model").map(String::as_str),
        data.get("provider_model").map(String::as_str),
        data.get("provider").map(String::as_str),
    )
    .unwrap_or_else(|| "all".to_string());
    let model = stat_model(
        Some(data.get("model").map(String::as_str).unwrap_or("all")),
        data.get("provider_model").map(String::as_str),
    );
    base_aggregate(data)
        .into_iter()
        .map(|base| GeoAggregate {
            grain: base.grain,
            period_key: base.period_key,
            dataset: base.dataset,
            tier: base.tier,
            provider: provider.clone(),
            model: model.clone(),
            country: normalize_country(data.get("country")),
            continent: data.get("continent").cloned().unwrap_or_default(),
            requests: base.requests,
        })
        .collect()
}

/// Map an R2 row into a retention aggregate.
pub fn to_retention_aggregate(data: &R2SqlData) -> Vec<RetentionAggregate> {
    let cohort_date = match data.get("cohort_date") {
        Some(value) if !value.is_empty() => value.clone(),
        _ => return Vec::new(),
    };
    let raw_model = match data.get("model") {
        Some(value) if !value.is_empty() => value.clone(),
        _ => return Vec::new(),
    };
    vec![RetentionAggregate {
        cohort_date,
        dataset: data.get("dataset").cloned().unwrap_or_default(),
        tier: data.get("tier").cloned().unwrap_or_else(|| "all".into()),
        provider: stat_provider(
            Some(raw_model.as_str()),
            Some(""),
            data.get("provider").map(String::as_str),
        )
        .unwrap_or_else(|| "unknown".to_string()),
        model: stat_model(Some(raw_model.as_str()), None),
        eligible_users: integer(data, "eligible_users"),
        retained_users: integer(data, "retained_users"),
    }]
}

/// Build the per-day/per-week usage and geo queries for a window.
pub fn build_stats_queries(
    _period_start: &str,
    _period_end: &str,
    _input: Option<&StatsQuerySource>,
) -> Result<Vec<String>, StatsError> {
    Err(StatsError::NotImplemented("stats query builder"))
}

/// Build the week-over-week retention queries for a window.
pub fn build_retention_queries(
    _period_start: &str,
    _period_end: &str,
    _input: Option<&StatsQuerySource>,
) -> Result<Vec<RetentionQuery>, StatsError> {
    Err(StatsError::NotImplemented("retention query builder"))
}
