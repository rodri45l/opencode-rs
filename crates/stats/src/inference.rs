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

const DAY_MS: i64 = 86_400_000;
const WEEK_MS: i64 = 7 * DAY_MS;
const LIVE_SOURCE_START: &str = "2026-08-11T10:57:48.186Z";

use crate::model_normalization::{
    EXCLUDED_MODELS, FREE_MODELS, MODEL_AUTHOR_RULES, MODEL_NAME_ALIASES, MODEL_NAME_MAX_LENGTH,
    RETIRED_STAT_PROVIDERS, STEALTH_MODELS,
};

fn sql_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn sql_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (if month <= 2 { y + 1 } else { y }, month, day)
}

fn parse_iso_ms(value: &str) -> i64 {
    let year: i64 = value[0..4].parse().unwrap_or(0);
    let month: i64 = value[5..7].parse().unwrap_or(1);
    let day: i64 = value[8..10].parse().unwrap_or(1);
    let hour: i64 = value.get(11..13).and_then(|v| v.parse().ok()).unwrap_or(0);
    let minute: i64 = value.get(14..16).and_then(|v| v.parse().ok()).unwrap_or(0);
    let second: i64 = value.get(17..19).and_then(|v| v.parse().ok()).unwrap_or(0);
    let milli: i64 = value.get(20..23).and_then(|v| v.parse().ok()).unwrap_or(0);
    days_from_civil(year, month, day) * DAY_MS
        + hour * 3_600_000
        + minute * 60_000
        + second * 1_000
        + milli
}

fn utc_parts(ms: i64) -> (i64, i64, i64, i64, i64, i64, i64) {
    let days = ms.div_euclid(DAY_MS);
    let rem = ms.rem_euclid(DAY_MS);
    let (year, month, day) = civil_from_days(days);
    let hour = rem / 3_600_000;
    let minute = (rem % 3_600_000) / 60_000;
    let second = (rem % 60_000) / 1_000;
    let milli = rem % 1_000;
    (year, month, day, hour, minute, second, milli)
}

fn format_iso(ms: i64) -> String {
    let (year, month, day, hour, minute, second, milli) = utc_parts(ms);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{milli:03}Z")
}

fn utc_date_id(ms: i64) -> String {
    let (year, month, day, _, _, _, _) = utc_parts(ms);
    format!("{year:04}-{month:02}-{day:02}")
}

fn start_of_utc_day(ms: i64) -> i64 {
    ms.div_euclid(DAY_MS) * DAY_MS
}

fn start_of_iso_week(ms: i64) -> i64 {
    let day = ms.div_euclid(DAY_MS);
    let weekday = (day + 4).rem_euclid(7);
    let back = if weekday == 0 { 7 } else { weekday };
    (day - back + 1) * DAY_MS
}

fn iso_week_id(ms: i64) -> String {
    let day = ms.div_euclid(DAY_MS);
    let weekday = (day + 4).rem_euclid(7);
    let back = if weekday == 0 { 7 } else { weekday };
    let thursday = day + 4 - back;
    let (year, _, _) = civil_from_days(thursday);
    let jan1 = days_from_civil(year, 1, 1);
    let week = (thursday - jan1 + 7) / 7;
    format!("{year:04}-W{week:02}")
}

struct StatPeriod {
    grain: &'static str,
    key: String,
    start: i64,
    end: i64,
}

fn stat_periods(grain: &'static str, period_start: i64, period_end: i64) -> Vec<StatPeriod> {
    let interval = if grain == "day" { DAY_MS } else { WEEK_MS };
    let first = if grain == "day" {
        start_of_utc_day(period_start)
    } else {
        start_of_iso_week(period_start)
    };
    let span = period_end - first;
    let count = if span <= 0 {
        0
    } else {
        (span + interval - 1) / interval
    };
    (0..count)
        .map(|index| {
            let start = first + index * interval;
            let end = (start + interval).min(period_end);
            StatPeriod {
                grain,
                key: if grain == "day" {
                    utc_date_id(start)
                } else {
                    iso_week_id(start)
                },
                start,
                end,
            }
        })
        .collect()
}

struct RetentionPeriod {
    start: i64,
    return_start: i64,
    return_end: i64,
}

fn retention_periods(period_start: i64, period_end: i64) -> Vec<RetentionPeriod> {
    let first = start_of_iso_week(period_start);
    let complete_end = start_of_iso_week(period_end);
    let count = ((complete_end - first) / WEEK_MS - 1).max(0);
    (0..count)
        .map(|index| {
            let start = first + index * WEEK_MS;
            let end = start + WEEK_MS;
            RetentionPeriod {
                start,
                return_start: end,
                return_end: end + WEEK_MS,
            }
        })
        .collect()
}

fn stat_model_sql(model: &str, provider_model: &str) -> String {
    let normalized = format!(
        "regexp_replace(CASE\n      WHEN lower({model}) = 'big-pickle' THEN regexp_replace(NULLIF({provider_model}, ''), '^.*/', '')\n      ELSE {model}\n    END, '(-free|:free|:global)+$', '')"
    );
    let mut alias_lines = String::new();
    for (from, to) in MODEL_NAME_ALIASES {
        alias_lines.push_str(&format!(
            "\n      WHEN lower({normalized}) = {} THEN {}",
            sql_string(from),
            sql_string(to)
        ));
    }
    let value = format!("CASE{alias_lines}\n      ELSE {normalized}\n    END");
    format!(
        "CASE\n      WHEN length({value}) > {MODEL_NAME_MAX_LENGTH} THEN 'unknown'\n      ELSE COALESCE(NULLIF({value}, ''), 'unknown')\n    END"
    )
}

fn free_tier_sql(tier: &str, model: &str) -> String {
    let models: Vec<String> = FREE_MODELS.iter().map(|m| sql_string(m)).collect();
    format!(
        "lower(COALESCE({tier}, '')) = 'free'\n        OR lower({model}) IN ({})\n        OR lower({model}) LIKE '%-free'\n        OR lower({model}) LIKE '%-free:global'",
        models.join(", ")
    )
}

fn stat_provider_sql(model: &str, provider_model: &str, provider: &str) -> String {
    let stealth: Vec<String> = STEALTH_MODELS.iter().map(|m| sql_string(m)).collect();
    let mut lines = format!(
        "\n      WHEN lower({model}) IN ({}) THEN 'unknown'",
        stealth.join(", ")
    );
    for (needle, author) in MODEL_AUTHOR_RULES {
        lines.push_str(&format!(
            "\n      WHEN strpos(lower({provider_model}), {}) > 0 THEN {}",
            sql_string(needle),
            sql_string(author)
        ));
    }
    for (needle, author) in MODEL_AUTHOR_RULES {
        lines.push_str(&format!(
            "\n      WHEN strpos(lower({model}), {}) > 0 THEN {}",
            sql_string(needle),
            sql_string(author)
        ));
    }
    let retired: Vec<String> = RETIRED_STAT_PROVIDERS
        .iter()
        .map(|p| sql_string(p))
        .collect();
    lines.push_str(&format!(
        "\n      WHEN {provider} <> '' AND lower({provider}) NOT IN ({}) THEN {provider}",
        retired.join(", ")
    ));
    lines.push_str("\n      ELSE 'unknown'\n    END");
    format!("CASE{lines}")
}

fn build_stats_query(period: &StatPeriod, source: &StatsQuerySource, family: &str) -> String {
    let period_start_value = sql_string(&format_iso(period.start));
    let period_end_value = sql_string(&format_iso(period.end));
    let ingest_end_value = sql_string(&format_iso(period.end + DAY_MS));
    let source_table = format!(
        "{}.{}",
        sql_identifier(&source.namespace),
        sql_identifier(&source.table)
    );
    let source_free_tier = free_tier_sql("model_tier", "model_requested");
    let stat_model = stat_model_sql("model_requested", "route_model");
    let stat_provider = stat_provider_sql("model", "provider_model", "raw_provider");
    let free_tier = free_tier_sql("raw_tier", "raw_model");
    let excluded: Vec<String> = EXCLUDED_MODELS.iter().map(|m| sql_string(m)).collect();

    let (dimensions, distinct_columns, grouping_sets) = if family == "usage" {
        (
            "CASE WHEN grouping(model) = 0 THEN 'model' ELSE 'provider' END AS dimension,\n  tier,\n  provider,\n  CASE WHEN grouping(model) = 0 THEN model END AS model,\n  CASE WHEN grouping(model) = 0 THEN COALESCE(MAX(NULLIF(provider_model, '')), '') END AS provider_model,\n  null AS country,\n  null AS continent",
            "approx_distinct(session) AS sessions,\n    approx_distinct(user_key) AS unique_users",
            "(tier, provider, model),\n  (tier, provider)",
        )
    } else {
        (
            "CASE WHEN grouping(model) = 0 THEN 'geo_model' ELSE 'geo' END AS dimension,\n  tier,\n  CASE WHEN grouping(model) = 0 THEN provider ELSE 'all' END AS provider,\n  CASE WHEN grouping(model) = 0 THEN model ELSE 'all' END AS model,\n  null AS provider_model,\n  country,\n  COALESCE(MAX(NULLIF(continent, '')), '') AS continent",
            "0 AS sessions,\n    0 AS unique_users",
            "(tier, country),\n  (tier, provider, model, country)",
        )
    };
    let aggregate_columns = format!(
        "\n    {distinct_columns},\n    COUNT(*) AS requests,\n    COALESCE(SUM(tokens_input), 0) AS input_tokens,\n    COALESCE(SUM(tokens_output), 0) AS output_tokens,\n    COALESCE(SUM(tokens_reasoning), 0) AS reasoning_tokens,\n    COALESCE(SUM(tokens_cache_read), 0) AS cache_read_tokens,\n    COALESCE(SUM(tokens_total), 0) AS total_tokens,\n    COALESCE(SUM(cost_input_microcents), 0) AS input_cost_microcents,\n    COALESCE(SUM(cost_output_microcents), 0) AS output_cost_microcents,\n    COALESCE(SUM(cost_total_microcents), 0) AS total_cost_microcents,\n    AVG(duration_ms) AS avg_duration_ms,\n    null AS p50_duration_ms,\n    null AS p95_duration_ms,\n    AVG(ttfb_ms) AS avg_ttfb_ms,\n    null AS p50_ttfb_ms,\n    null AS p95_ttfb_ms,\n    AVG(output_tps) AS avg_output_tps,\n    SUM(CASE WHEN outcome = 'succeeded' THEN 1 ELSE 0 END) AS success_count,\n    SUM(CASE WHEN outcome = 'failed' THEN 1 ELSE 0 END) AS error_count,\n    COUNT(*) AS sample_count"
    );

    let mut sql = String::new();
    sql.push_str("\nWITH normalized AS (\n  SELECT\n    model_requested AS raw_model,\n    COALESCE(NULLIF(lower(model_tier), ''), '') AS raw_tier,\n    ");
    sql.push_str(&stat_model);
    sql.push_str(" AS model,\n    COALESCE(NULLIF(route_model, ''), '') AS provider_model,\n    COALESCE(NULLIF(provider_id, ''), '') AS raw_provider,\n    UPPER(COALESCE(NULLIF(country, ''), 'ZZ')) AS country,\n    COALESCE(NULLIF(continent, ''), '') AS continent,\n    session_id AS session,\n    COALESCE(NULLIF(workspace_id, ''), '') AS workspace,\n    COALESCE(NULLIF(service_api_key_id, ''), '') AS api_key,\n    COALESCE(NULLIF(user_id, ''), '') AS user_id,\n    outcome,\n    duration_ms,\n    time_to_first_token_ms AS ttfb_ms,\n    CASE\n      WHEN first_token_at IS NULL OR last_token_at IS NULL THEN null\n      ELSE date_part('epoch', last_token_at) - date_part('epoch', first_token_at)\n    END AS output_seconds,\n    tokens_input,\n    tokens_output,\n    tokens_reasoning,\n    tokens_cache_read,\n    tokens_cache_write,\n    cost_input AS cost_input_microcents,\n    cost_output AS cost_output_microcents,\n    cost_total AS cost_total_microcents\n  FROM ");
    sql.push_str(&source_table);
    sql.push_str("\n  WHERE event_type = 'generation.completed'\n    AND source IN ('inference', 'inference-legacy')\n    AND (\n      (source = 'inference-legacy' AND started_at < ");
    sql.push_str(&sql_string(LIVE_SOURCE_START));
    sql.push_str(")\n      OR (source = 'inference' AND started_at >= ");
    sql.push_str(&sql_string(LIVE_SOURCE_START));
    sql.push_str(")\n    )\n    AND (product = 'go' OR (");
    sql.push_str(&source_free_tier);
    sql.push_str("))\n    AND model_requested IS NOT NULL\n    AND model_requested <> ''\n    AND __ingest_ts >= ");
    sql.push_str(&period_start_value);
    sql.push_str("\n    AND __ingest_ts < ");
    sql.push_str(&ingest_end_value);
    sql.push_str("\n    AND started_at >= ");
    sql.push_str(&period_start_value);
    sql.push_str("\n    AND started_at < ");
    sql.push_str(&period_end_value);
    sql.push_str("\n), filtered AS (\n  SELECT\n    CASE\n      WHEN ");
    sql.push_str(&free_tier);
    sql.push_str("\n      THEN 'Free'\n      ELSE 'Go'\n    END AS tier,\n    ");
    sql.push_str(&stat_provider);
    sql.push_str(" AS provider,\n    provider_model,\n    model,\n    country,\n    continent,\n    session,\n    COALESCE(NULLIF(user_id, ''), NULLIF(workspace, ''), NULLIF(api_key, '')) AS user_key,\n    outcome,\n    duration_ms,\n    ttfb_ms,\n    CASE\n      WHEN output_seconds < 0.1 THEN null\n      ELSE CAST(tokens_output AS double) / output_seconds\n    END AS output_tps,\n    tokens_input,\n    tokens_output,\n    tokens_reasoning,\n    tokens_cache_read,\n    COALESCE(tokens_cache_read, 0) + COALESCE(tokens_cache_write, 0) + COALESCE(tokens_input, 0) + COALESCE(tokens_output, 0) AS tokens_total,\n    cost_input_microcents,\n    cost_output_microcents,\n    cost_total_microcents\n  FROM normalized\n  WHERE lower(model) NOT IN (");
    sql.push_str(&excluded.join(", "));
    sql.push_str(")\n)\nSELECT\n  ");
    sql.push_str(&sql_string(period.grain));
    sql.push_str(" AS grain,\n  ");
    sql.push_str(&sql_string(&period.key));
    sql.push_str(" AS period_key,\n  ");
    sql.push_str(&sql_string(&source.dataset));
    sql.push_str(" AS dataset,\n  ");
    sql.push_str(dimensions);
    sql.push_str(",\n  ");
    sql.push_str(&aggregate_columns);
    sql.push_str("\nFROM filtered\nGROUP BY GROUPING SETS (\n  ");
    sql.push_str(grouping_sets);
    sql.push_str("\n)\n");
    sql
}

fn default_source(input: Option<&StatsQuerySource>) -> StatsQuerySource {
    input.cloned().unwrap_or(StatsQuerySource {
        namespace: "inference".into(),
        table: "generation".into(),
        dataset: "zen".into(),
    })
}

/// Build the per-day/per-week usage and geo queries for a window.
pub fn build_stats_queries(
    period_start: &str,
    period_end: &str,
    input: Option<&StatsQuerySource>,
) -> Result<Vec<String>, StatsError> {
    let source = default_source(input);
    let start = parse_iso_ms(period_start);
    let end = parse_iso_ms(period_end);
    let mut periods = stat_periods("week", start, end);
    periods.extend(stat_periods("day", start, end));
    let mut queries = Vec::new();
    for period in &periods {
        queries.push(build_stats_query(period, &source, "usage"));
        queries.push(build_stats_query(period, &source, "geo"));
    }
    Ok(queries)
}

fn build_retention_query(periods: &[RetentionPeriod], source: &StatsQuerySource) -> String {
    let first = &periods[0];
    let last = periods.last().expect("retention periods");
    let scan_start_value = sql_string(&format_iso(first.start));
    let scan_end_value = sql_string(&format_iso(last.return_end));
    let ingest_end_value = sql_string(&format_iso(last.return_end + DAY_MS));
    let source_table = format!(
        "{}.{}",
        sql_identifier(&source.namespace),
        sql_identifier(&source.table)
    );
    let mut activity_weeks: Vec<i64> = Vec::new();
    for period in periods {
        for date in [period.start, period.return_start] {
            if !activity_weeks.contains(&date) {
                activity_weeks.push(date);
            }
        }
    }
    activity_weeks.sort_unstable();
    let mut activity_lines = String::new();
    for date in &activity_weeks {
        activity_lines.push_str(&format!(
            "\n      WHEN started_at >= {} AND started_at < {} THEN {}",
            sql_string(&format_iso(*date)),
            sql_string(&format_iso(date + WEEK_MS)),
            sql_string(&utc_date_id(*date))
        ));
    }
    let activity_week_sql = format!("CASE{activity_lines}\n      ELSE null\n    END");
    let cohort_dates = periods
        .iter()
        .map(|period| sql_string(&utc_date_id(period.start)))
        .collect::<Vec<_>>()
        .join(", ");
    let return_dates = periods
        .iter()
        .map(|period| sql_string(&utc_date_id(period.return_start)))
        .collect::<Vec<_>>()
        .join(", ");
    let mut return_lines = String::new();
    for period in periods {
        return_lines.push_str(&format!(
            "\n      WHEN {} THEN {}",
            sql_string(&utc_date_id(period.return_start)),
            sql_string(&utc_date_id(period.start))
        ));
    }
    let return_cohort_sql = format!("CASE activity_week{return_lines}\n    END");
    let stat_model = stat_model_sql("model_requested", "route_model");
    let stat_provider = stat_provider_sql("model", "provider_model", "raw_provider");
    let excluded: Vec<String> = EXCLUDED_MODELS.iter().map(|m| sql_string(m)).collect();

    let mut sql = String::new();
    sql.push_str("\nWITH normalized AS (\n  SELECT\n    ");
    sql.push_str(&activity_week_sql);
    sql.push_str(" AS activity_week,\n    ");
    sql.push_str(&stat_model);
    sql.push_str(" AS model,\n    COALESCE(NULLIF(route_model, ''), '') AS provider_model,\n    COALESCE(NULLIF(provider_id, ''), '') AS raw_provider,\n    COALESCE(NULLIF(user_id, ''), NULLIF(workspace_id, ''), NULLIF(service_api_key_id, '')) AS user_key\n  FROM ");
    sql.push_str(&source_table);
    sql.push_str("\n  WHERE event_type = 'generation.completed'\n    AND source IN ('inference', 'inference-legacy')\n    AND (\n      (source = 'inference-legacy' AND started_at < ");
    sql.push_str(&sql_string(LIVE_SOURCE_START));
    sql.push_str(")\n      OR (source = 'inference' AND started_at >= ");
    sql.push_str(&sql_string(LIVE_SOURCE_START));
    sql.push_str(")\n    )\n    AND product = 'go'\n    AND model_requested IS NOT NULL\n    AND model_requested <> ''\n    AND __ingest_ts >= ");
    sql.push_str(&scan_start_value);
    sql.push_str("\n    AND __ingest_ts < ");
    sql.push_str(&ingest_end_value);
    sql.push_str("\n    AND started_at >= ");
    sql.push_str(&scan_start_value);
    sql.push_str("\n    AND started_at < ");
    sql.push_str(&scan_end_value);
    sql.push_str("\n), filtered AS (\n  SELECT\n    activity_week,\n    ");
    sql.push_str(&stat_provider);
    sql.push_str(" AS provider,\n    model,\n    user_key\n  FROM normalized\n  WHERE activity_week IS NOT NULL\n    AND user_key <> ''\n    AND lower(model) NOT IN (");
    sql.push_str(&excluded.join(", "));
    sql.push_str(")\n), model_usage AS (\n  SELECT\n    activity_week AS cohort_date,\n    user_key,\n    provider,\n    model,\n    COUNT(*) AS model_requests\n  FROM filtered\n  WHERE activity_week IN (");
    sql.push_str(&cohort_dates);
    sql.push_str(")\n  GROUP BY activity_week, user_key, provider, model\n), user_totals AS (\n  SELECT\n    cohort_date,\n    user_key,\n    SUM(model_requests) AS total_requests,\n    MAX(model_requests) AS max_model_requests\n  FROM model_usage\n  GROUP BY cohort_date, user_key\n), primary_models AS (\n  SELECT model_usage.cohort_date, model_usage.user_key, model_usage.provider, model_usage.model\n  FROM model_usage\n  INNER JOIN user_totals ON model_usage.cohort_date = user_totals.cohort_date\n    AND model_usage.user_key = user_totals.user_key\n    AND model_usage.model_requests = user_totals.max_model_requests\n  WHERE user_totals.total_requests >= 10\n    AND CAST(model_usage.model_requests AS double) / NULLIF(user_totals.total_requests, 0) >= 0.8\n), returned AS (\n  SELECT\n    ");
    sql.push_str(&return_cohort_sql);
    sql.push_str(" AS cohort_date,\n    user_key\n  FROM filtered\n  WHERE activity_week IN (");
    sql.push_str(&return_dates);
    sql.push_str(")\n  GROUP BY ");
    sql.push_str(&return_cohort_sql);
    sql.push_str(", user_key\n)\nSELECT\n  primary_models.cohort_date,\n  ");
    sql.push_str(&sql_string(&source.dataset));
    sql.push_str(" AS dataset,\n  'Go' AS tier,\n  primary_models.provider,\n  primary_models.model,\n  COUNT(*) AS eligible_users,\n  SUM(CASE WHEN returned.user_key IS NULL THEN 0 ELSE 1 END) AS retained_users\nFROM primary_models\nLEFT JOIN returned ON primary_models.user_key = returned.user_key\n  AND primary_models.cohort_date = returned.cohort_date\nGROUP BY primary_models.cohort_date, primary_models.provider, primary_models.model\nLIMIT 10000\n");
    sql
}

/// Build the week-over-week retention queries for a window.
pub fn build_retention_queries(
    period_start: &str,
    period_end: &str,
    input: Option<&StatsQuerySource>,
) -> Result<Vec<RetentionQuery>, StatsError> {
    let source = default_source(input);
    let periods = retention_periods(parse_iso_ms(period_start), parse_iso_ms(period_end));
    Ok(periods
        .iter()
        .map(|period| RetentionQuery {
            cohort_dates: vec![utc_date_id(period.start)],
            query: build_retention_query(std::slice::from_ref(period), &source),
        })
        .collect())
}
