//! Port of packages/stats/core/src/domain/inference.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/stats/core/src/domain/inference.ts; see docs/TEST-PORT.md.
//! Normalization and aggregate mappers are ported green; exact R2 SQL string
//! builders are re-derived red-first (`#[ignore]`) until that surface lands.

use opencode_stats::inference::{
    build_retention_queries, build_stats_queries, to_geo_aggregate, to_model_aggregate,
    to_provider_aggregate, to_retention_aggregate, StatsQuerySource,
};
use opencode_stats::model_normalization::{
    model_author, normalize_inference_model, stat_model, stat_provider, RETIRED_STAT_MODELS,
};
use opencode_stats::r2_sql::R2SqlData;
use std::collections::BTreeMap;

fn aggregate(model: &str, provider: &str) -> R2SqlData {
    let mut data = BTreeMap::new();
    data.insert("grain".into(), "day".into());
    data.insert("period_key".into(), "2026-05-20".into());
    data.insert("dataset".into(), "zen".into());
    data.insert("tier".into(), "Paid".into());
    data.insert("provider".into(), provider.into());
    data.insert("model".into(), model.into());
    data.insert("sessions".into(), "1".into());
    data.insert("requests".into(), "1".into());
    data.insert("sample_count".into(), "1".into());
    data
}

fn with(mut data: R2SqlData, key: &str, value: &str) -> R2SqlData {
    data.insert(key.into(), value.into());
    data
}

fn source() -> StatsQuerySource {
    StatsQuerySource {
        namespace: "inference".into(),
        table: "generation".into(),
        dataset: "zen".into(),
    }
}

#[test]
fn normalizes_model_suffixes_used_by_router_provider_variants() {
    assert_eq!(normalize_inference_model(Some("GPT-5-Free")), "gpt-5");
    assert_eq!(
        normalize_inference_model(Some("deepseek-v4-flash-free")),
        "deepseek-v4-flash"
    );
    assert_eq!(
        normalize_inference_model(Some("deepseek-v4-flash:global")),
        "deepseek-v4-flash"
    );
    assert_eq!(
        normalize_inference_model(Some("mimo-v2.5-free")),
        "mimo-v2.5"
    );
    assert_eq!(
        normalize_inference_model(Some("nemotron-3-super-free")),
        "nemotron-3-super"
    );
    assert_eq!(
        normalize_inference_model(Some("mimo-v2.5-free:global")),
        "mimo-v2.5"
    );
    assert_eq!(
        normalize_inference_model(Some("hy3-preview:free")),
        "hy3-preview"
    );
}

#[test]
fn maps_normalized_model_ids_to_public_authors() {
    assert_eq!(model_author(Some("big-pickle")), Some("unknown"));
    assert_eq!(model_author(Some("claude-sonnet-4-5")), Some("anthropic"));
    assert_eq!(model_author(Some("deepseek-v4-pro")), Some("deepseek"));
    assert_eq!(model_author(Some("gemini-3.5-flash")), Some("google"));
    assert_eq!(model_author(Some("glm-5.1")), Some("zhipu"));
    assert_eq!(model_author(Some("gpt-5.5-pro")), Some("openai"));
    assert_eq!(model_author(Some("grok-build-0.1")), Some("xai"));
    assert_eq!(model_author(Some("hy3-preview")), Some("tencent"));
    assert_eq!(model_author(Some("kimi-k2.6")), Some("moonshot"));
    assert_eq!(model_author(Some("mimo-v2-omni")), Some("xiaomi"));
    assert_eq!(model_author(Some("minimax-m2.7")), Some("minimax"));
    assert_eq!(
        model_author(Some("muse-spark-1.2-contributor")),
        Some("meta")
    );
    assert_eq!(model_author(Some("nemotron-3-super-free")), Some("nvidia"));
    assert_eq!(model_author(Some("qwen3.7-max")), Some("qwen"));
    assert_eq!(model_author(Some("alpha-gpt-next")), None);
    assert_eq!(model_author(Some("omen-alpha")), Some("unknown"));
    assert_eq!(
        model_author(Some("OMEN-ALPHA-free:global")),
        Some("unknown")
    );
}

#[test]
fn uses_provider_model_to_resolve_opencode_route_providers() {
    assert_eq!(
        stat_model(Some("big-pickle"), Some("claude-sonnet-4-5")),
        "claude-sonnet-4-5"
    );
    assert_eq!(stat_model(Some("big-pickle"), Some("gpt-5-free")), "gpt-5");
    assert_eq!(
        stat_model(Some("big-pickle"), Some("xiaomi/mimo-v2.5")),
        "mimo-v2.5"
    );
    assert_eq!(stat_model(Some("big-pickle"), Some("")), "unknown");
    assert_eq!(
        stat_provider(
            Some("big-pickle"),
            Some("claude-sonnet-4-5"),
            Some("opencode")
        ),
        Some("anthropic".to_string())
    );
    assert_eq!(
        stat_provider(Some("big-pickle"), Some("gpt-5"), Some("opencode")),
        Some("openai".to_string())
    );
    assert_eq!(
        stat_provider(Some("big-pickle"), Some(""), Some("opencode")),
        Some("unknown".to_string())
    );
    assert_eq!(
        stat_provider(Some("unknown"), Some(""), Some("custom-provider")),
        Some("custom-provider".to_string())
    );
}

#[test]
fn maps_oversized_model_ids_to_unknown_before_aggregation() {
    let at_limit = "x".repeat(256);
    let over_limit = "x".repeat(257);
    let provider_model = format!("provider/{over_limit}");
    assert_eq!(stat_model(Some(at_limit.as_str()), Some("")), at_limit);
    assert_eq!(stat_model(Some(over_limit.as_str()), Some("")), "unknown");
    assert_eq!(
        stat_model(Some("big-pickle"), Some(provider_model.as_str())),
        "unknown"
    );
}

#[test]
#[ignore = "porting: stats query builder not implemented"]
fn emits_length_guards_in_generated_r2_sql() {
    let queries = build_stats_queries(
        "2026-09-16T00:00:00.000Z",
        "2026-09-16T04:00:00.000Z",
        Some(&source()),
    )
    .unwrap();
    assert!(queries[0].contains("WHEN length("));
    assert!(queries[0].contains(") > 256 THEN 'unknown'"));
}

#[test]
fn keeps_stealth_model_usage_without_exposing_the_route_provider() {
    assert_eq!(
        stat_provider(
            Some("omen-alpha"),
            Some("gpt-test-model"),
            Some("test-provider")
        ),
        Some("unknown".to_string())
    );
    assert_eq!(
        stat_provider(
            Some("OMEN-ALPHA-free:global"),
            Some("gpt-test-model"),
            Some("test-provider")
        ),
        Some("unknown".to_string())
    );
    assert_eq!(
        stat_provider(Some("omen-alpha"), Some(""), Some("test-provider")),
        Some("unknown".to_string())
    );

    let row = with(
        aggregate("omen-alpha", "test-provider"),
        "provider_model",
        "gpt-test-model",
    );
    let models = to_model_aggregate(&row);
    let model = &models[0];
    assert_eq!(model.model, "omen-alpha");
    assert_eq!(model.provider, "unknown");
    assert_eq!(model.requests, 1);
    assert_eq!(to_provider_aggregate(&row)[0].provider, "unknown");
    assert_eq!(to_provider_aggregate(&row)[0].requests, 1);
    let geo_rows = to_geo_aggregate(&with(row.clone(), "country", "US"));
    let geo = &geo_rows[0];
    assert_eq!(geo.model, "omen-alpha");
    assert_eq!(geo.provider, "unknown");
    assert_eq!(geo.country, "US");
    assert_eq!(geo.requests, 1);
    let retention_rows = to_retention_aggregate(&with(
        with(row.clone(), "cohort_date", "2026-08-10"),
        "eligible_users",
        "12",
    ));
    let retention = &retention_rows[0];
    assert_eq!(retention.model, "omen-alpha");
    assert_eq!(retention.provider, "unknown");
    assert_eq!(retention.eligible_users, 12);

    for model in ["opencode-go/union-alpha", "opencode/union-alpha"] {
        assert_eq!(stat_model(Some(model), Some("")), "union-alpha");
        assert_eq!(
            stat_provider(Some(model), Some("gpt-test-model"), Some("test-provider")),
            Some("unknown".to_string())
        );
        let row = with(
            aggregate(model, "test-provider"),
            "provider_model",
            "gpt-test-model",
        );
        assert_eq!(to_model_aggregate(&row)[0].model, "union-alpha");
        assert_eq!(to_model_aggregate(&row)[0].provider, "unknown");
        assert_eq!(to_provider_aggregate(&row)[0].provider, "unknown");
        assert_eq!(
            to_geo_aggregate(&with(row.clone(), "country", "US"))[0].provider,
            "unknown"
        );
        assert_eq!(
            to_retention_aggregate(&with(
                with(row.clone(), "cohort_date", "2026-08-10"),
                "eligible_users",
                "12"
            ))[0]
                .model,
            "union-alpha"
        );
    }
}

#[test]
fn merges_renamed_models_under_their_current_name() {
    assert_eq!(
        stat_model(Some("deepseek-v4-flash-0731"), Some("")),
        "deepseek-v4-flash"
    );
    assert_eq!(
        stat_model(Some("deepseek-v4-flash-0731-free"), Some("")),
        "deepseek-v4-flash"
    );
    assert_eq!(
        stat_model(Some("deepseek-v4-flash-dsv4-flash-final-rnaovd"), Some("")),
        "deepseek-v4-flash"
    );
    assert_eq!(
        stat_model(Some("deepseek-v4-flash-vision-exp"), Some("")),
        "deepseek-v4-flash-vision-exp"
    );
    assert_eq!(stat_model(Some("x-preview-f"), Some("")), "glm-5.3-flash");
    assert_eq!(stat_model(Some("ox-alpha"), Some("")), "glm-5.3-flash");
    assert_eq!(stat_model(Some("ox-alpha-free"), Some("")), "glm-5.3-flash");
    assert_eq!(
        stat_model(Some("big-pickle"), Some("zhipuai/ox-alpha-free")),
        "glm-5.3-flash"
    );
    assert_eq!(stat_model(Some("xiaomi/mimo-v2.5"), Some("")), "mimo-v2.5");
    let renamed = to_model_aggregate(&aggregate("x-preview-f", "unknown"));
    let model = &renamed[0];
    assert_eq!(model.provider, "zhipu");
    assert_eq!(model.model, "glm-5.3-flash");
    assert_eq!(
        to_provider_aggregate(&aggregate("ox-alpha", "unknown"))[0].provider,
        "zhipu"
    );
}

#[test]
fn renames_deepseek_flash_to_v41_without_merging_v4_or_vision_usage() {
    for model in [
        "deepseek-flash",
        "DEEPSEEK-FLASH-free:global",
        "deepseek-v4.1-flash",
    ] {
        assert_eq!(stat_model(Some(model), Some("")), "deepseek-v4.1-flash");
        assert_eq!(
            to_model_aggregate(&aggregate(model, "deepseek"))[0].model,
            "deepseek-v4.1-flash"
        );
        assert_eq!(
            to_model_aggregate(&aggregate(model, "deepseek"))[0].provider,
            "deepseek"
        );
        assert_eq!(
            to_geo_aggregate(&with(aggregate(model, "deepseek"), "country", "US"))[0].model,
            "deepseek-v4.1-flash"
        );
        assert_eq!(
            to_retention_aggregate(&with(
                aggregate(model, "deepseek"),
                "cohort_date",
                "2026-08-10"
            ))[0]
                .model,
            "deepseek-v4.1-flash"
        );
    }
    assert_eq!(
        stat_model(Some("big-pickle"), Some("deepseek/deepseek-flash")),
        "deepseek-v4.1-flash"
    );
    assert_eq!(
        stat_model(Some("deepseek-v4-flash"), Some("")),
        "deepseek-v4-flash"
    );
    assert_eq!(
        stat_model(Some("deepseek-v4-flash-vision-exp"), Some("")),
        "deepseek-v4-flash-vision-exp"
    );
    assert!(RETIRED_STAT_MODELS.contains(&"deepseek-flash"));
    assert!(!RETIRED_STAT_MODELS.contains(&"deepseek-v4.1-flash"));
}

#[test]
fn model_aggregates_prefer_provider_model_and_use_normalized_model() {
    assert!(to_model_aggregate(&aggregate("alpha-gpt-next", "openai")).is_empty());

    let flash_rows =
        to_model_aggregate(&aggregate("deepseek-v4-flash-free", "not-public-provider"));
    let flash = &flash_rows[0];
    assert_eq!(flash.period_key, "2026-05-20");
    assert_eq!(flash.provider, "deepseek");
    assert_eq!(flash.model, "deepseek-v4-flash");

    let pickled_rows = to_model_aggregate(&with(
        aggregate("big-pickle", "opencode"),
        "provider_model",
        "claude-sonnet-4-5",
    ));
    let pickled = &pickled_rows[0];
    assert_eq!(pickled.provider, "anthropic");
    assert_eq!(pickled.model, "claude-sonnet-4-5");
    assert_eq!(pickled.provider_model, "claude-sonnet-4-5");
}

#[test]
fn provider_aggregates_never_keep_opencode_as_the_provider() {
    assert_eq!(
        to_provider_aggregate(&with(
            aggregate("big-pickle", "opencode"),
            "provider_model",
            "gpt-5"
        ))[0]
            .provider,
        "openai"
    );
    assert_eq!(
        to_provider_aggregate(&aggregate("big-pickle", "opencode"))[0].provider,
        "unknown"
    );
    assert_eq!(
        to_provider_aggregate(&aggregate("muse-spark-1.2-contributor", "unknown"))[0].provider,
        "meta"
    );
}

#[test]
fn geo_aggregates_never_keep_opencode_or_big_pickle_dimensions() {
    let geo_rows = to_geo_aggregate(&with(aggregate("big-pickle", "opencode"), "country", "US"));
    let geo = &geo_rows[0];
    assert_eq!(geo.provider, "unknown");
    assert_eq!(geo.model, "unknown");
    assert_eq!(geo.country, "US");
}

#[test]
fn model_aggregates_use_iso_week_period_keys() {
    let row = with(
        with(aggregate("gpt-5.5-pro", "openai"), "grain", "week"),
        "period_key",
        "2026-W20",
    );
    assert_eq!(to_model_aggregate(&row)[0].period_key, "2026-W20");
}

#[test]
#[ignore = "porting: stats query builder not implemented"]
fn builds_bounded_r2_sql_queries_for_each_day_and_week() {
    let queries = build_stats_queries(
        "2026-08-10T00:00:00.000Z",
        "2026-08-12T12:00:00.000Z",
        Some(&source()),
    )
    .unwrap();
    assert_eq!(queries.len(), 8);
    for query in &queries {
        assert!(query.contains("WHERE lower(model) NOT IN ('alpha-gpt-next')"));
        assert!(query.contains("= 'opencode-go/union-alpha' THEN 'union-alpha'"));
        assert!(query.contains("= 'opencode/union-alpha' THEN 'union-alpha'"));
        assert!(query.contains("= 'deepseek-flash' THEN 'deepseek-v4.1-flash'"));
    }
    assert!(queries[0].contains("'week' AS grain"));
    assert!(queries[0].contains("'2026-W33' AS period_key"));
    assert!(queries[2].contains("'2026-08-10' AS period_key"));
    assert!(queries[6].contains("'2026-08-12' AS period_key"));
    assert!(queries[0].contains("FROM \"inference\".\"generation\""));
    assert!(queries[0].contains("event_type = 'generation.completed'"));
    assert!(queries[0].contains("approx_distinct(session) AS sessions"));
    assert!(!queries[0].contains("LIMIT"));
}

#[test]
#[ignore = "porting: stats query builder not implemented"]
fn aligns_periods_to_utc_calendar_boundaries() {
    let queries = build_stats_queries(
        "2026-06-17T15:56:00.000Z",
        "2026-06-19T15:56:00.000Z",
        Some(&source()),
    )
    .unwrap();
    assert_eq!(queries.len(), 8);
    assert!(queries[0].contains("'2026-W25' AS period_key"));
    assert!(queries[0].contains("started_at >= '2026-06-15T00:00:00.000Z'"));
    assert!(queries[2].contains("'2026-06-17' AS period_key"));
    assert!(queries[2].contains("started_at < '2026-06-18T00:00:00.000Z'"));
}

#[test]
#[ignore = "porting: stats query builder not implemented"]
fn uses_an_exclusive_live_and_legacy_source_handoff() {
    let queries = build_stats_queries(
        "2026-08-11T00:00:00.000Z",
        "2026-08-12T00:00:00.000Z",
        Some(&source()),
    )
    .unwrap();
    assert!(queries[0]
        .contains("(source = 'inference-legacy' AND started_at < '2026-08-11T10:57:48.186Z')"));
    assert!(
        queries[0].contains("(source = 'inference' AND started_at >= '2026-08-11T10:57:48.186Z')")
    );
}

#[test]
#[ignore = "porting: retention query builder not implemented"]
fn builds_complete_week_over_week_retention_queries() {
    let queries = build_retention_queries(
        "2026-08-10T00:00:00.000Z",
        "2026-08-31T00:00:00.000Z",
        Some(&source()),
    )
    .unwrap();
    assert_eq!(queries.len(), 2);
    for query in &queries {
        assert!(query
            .query
            .contains("= 'deepseek-flash' THEN 'deepseek-v4.1-flash'"));
    }
    assert_eq!(queries[0].cohort_dates, vec!["2026-08-10".to_string()]);
    assert_eq!(queries[1].cohort_dates, vec!["2026-08-17".to_string()]);
    assert!(queries[0].query.contains("AND product = 'go'"));
    assert!(queries[0].query.contains("LIMIT 10000"));
}

#[test]
#[ignore = "porting: retention query builder not implemented"]
fn splits_a_full_retention_window_without_dropping_or_duplicating_cohorts() {
    let queries = build_retention_queries(
        "2026-07-16T19:00:00Z",
        "2026-09-10T00:00:00Z",
        Some(&source()),
    )
    .unwrap();
    let cohorts: Vec<String> = queries
        .iter()
        .flat_map(|query| query.cohort_dates.clone())
        .collect();
    assert_eq!(
        cohorts,
        vec![
            "2026-07-13".to_string(),
            "2026-07-20".to_string(),
            "2026-07-27".to_string(),
            "2026-08-03".to_string(),
            "2026-08-10".to_string(),
            "2026-08-17".to_string(),
            "2026-08-24".to_string(),
        ]
    );
    assert!(build_retention_queries(
        "2026-08-31T00:00:00Z",
        "2026-09-10T00:00:00Z",
        Some(&source())
    )
    .unwrap()
    .is_empty());
}

#[test]
fn maps_retention_query_results() {
    let mut data = BTreeMap::new();
    data.insert("cohort_date".into(), "2026-08-10".into());
    data.insert("dataset".into(), "zen".into());
    data.insert("tier".into(), "all".into());
    data.insert("provider".into(), "deepseek".into());
    data.insert("model".into(), "deepseek-v4-flash-free".into());
    data.insert("eligible_users".into(), "125".into());
    data.insert("retained_users".into(), "74".into());

    let mapped_rows = to_retention_aggregate(&data);
    let mapped = &mapped_rows[0];
    assert_eq!(mapped.cohort_date, "2026-08-10");
    assert_eq!(mapped.dataset, "zen");
    assert_eq!(mapped.tier, "all");
    assert_eq!(mapped.provider, "deepseek");
    assert_eq!(mapped.model, "deepseek-v4-flash");
    assert_eq!(mapped.eligible_users, 125);
    assert_eq!(mapped.retained_users, 74);
}
