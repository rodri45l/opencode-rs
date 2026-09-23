//! Port of packages/console/app/test/liteUsage.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/console/app/src/lib/lite-usage.ts: the model
//! quota limit is the window limit divided by the multiplier; usage is grouped
//! into rounded contribution percentages that sum to the usage percent; rows are
//! ordered by contribution; same-model same-rate sources merge; unknown rates do
//! not merge with recorded rates; and the input sources are never mutated.

use opencode_console::lite_usage::{
    build_lite_usage_breakdown, get_model_quota_limit, UsageSource,
};

fn src(
    model: &str,
    name: &str,
    cost: f64,
    quota_cost: f64,
    multiplier: Option<f64>,
    estimated: bool,
) -> UsageSource {
    UsageSource {
        model: model.to_string(),
        name: name.to_string(),
        cost,
        quota_cost,
        multiplier,
        estimated,
    }
}

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn derives_the_model_quota_from_the_window_limit_and_multiplier() {
    assert_eq!(get_model_quota_limit(30.0, Some(1.0)), Some(30.0));
    assert_eq!(get_model_quota_limit(30.0, Some(2.0)), Some(15.0));
    assert_eq!(get_model_quota_limit(30.0, Some(4.0)), Some(7.5));
}

#[test]
fn groups_model_quota_usage_into_the_percentage_of_the_limit() {
    let result = build_lite_usage_breakdown(
        416.0,
        1_200.0,
        &[
            src("glm", "GLM", 200.0, 300.0, Some(1.5), false),
            src("kimi", "Kimi", 116.0, 116.0, Some(1.0), false),
        ],
    );

    assert_eq!(result.usage_percent, 34.7);
    let row = &result.rows[0];
    assert_eq!(row.name, "GLM");
    assert_eq!(row.multiplier, Some(1.5));
    assert_eq!(row.contribution_percent, 25.0);
    let total: f64 = result.rows.iter().map(|row| row.contribution_percent).sum();
    assert!(approx(total, result.usage_percent));
}

#[test]
fn distributes_credits_across_the_model_contributions() {
    let result = build_lite_usage_breakdown(
        366.0,
        1_200.0,
        &[
            src("glm", "GLM", 200.0, 300.0, Some(1.5), false),
            src("kimi", "Kimi", 116.0, 116.0, Some(1.0), true),
        ],
    );

    assert_eq!(result.rows.len(), 2);
    assert!(result
        .rows
        .iter()
        .all(|row| row.contribution_percent >= 0.0));
    let total: f64 = result.rows.iter().map(|row| row.contribution_percent).sum();
    assert!(approx(total, result.usage_percent));
}

#[test]
fn does_not_synthesize_a_row_when_request_history_is_unavailable() {
    let result = build_lite_usage_breakdown(120.0, 1_200.0, &[]);
    assert!(result.rows.is_empty());
}

#[test]
fn allocates_rounded_percentages_without_making_positive_rows_negative() {
    let sources: Vec<UsageSource> = (0..20)
        .map(|index| {
            src(
                &format!("model-{index}"),
                &format!("Model {index}"),
                4.0,
                4.0,
                Some(1.0),
                false,
            )
        })
        .collect();
    let result = build_lite_usage_breakdown(80.0, 10_000.0, &sources);

    assert!(result
        .rows
        .iter()
        .all(|row| row.contribution_percent >= 0.0));
    let total: f64 = result.rows.iter().map(|row| row.contribution_percent).sum();
    assert!(approx(total, result.usage_percent));
}

#[test]
fn keeps_multiplier_changes_for_the_same_model_as_separate_rows() {
    let result = build_lite_usage_breakdown(
        500.0,
        1_000.0,
        &[
            src("glm", "GLM", 100.0, 100.0, Some(1.0), false),
            src("glm", "GLM", 200.0, 400.0, Some(2.0), false),
        ],
    );

    assert_eq!(
        result
            .rows
            .iter()
            .map(|row| row.multiplier)
            .collect::<Vec<_>>(),
        vec![Some(2.0), Some(1.0)]
    );
    assert_eq!(
        result
            .rows
            .iter()
            .map(|row| row.contribution_percent)
            .collect::<Vec<_>>(),
        vec![40.0, 10.0]
    );
}

#[test]
fn merges_same_rate_usage_regardless_of_estimated_order() {
    for estimated in [false, true] {
        let sources = vec![
            src(
                "deepseek-v4-flash",
                "DeepSeek V4 Flash",
                200.0,
                400.0,
                Some(2.0),
                estimated,
            ),
            src("other", "Other", 500.0, 500.0, Some(1.0), false),
            src(
                "deepseek-v4-flash",
                "DeepSeek V4 Flash",
                100.0,
                199.0,
                Some(2.0),
                !estimated,
            ),
        ];
        let original = sources.clone();
        let result = build_lite_usage_breakdown(1_050.0, 6_000.0, &sources);

        assert_eq!(result.rows.len(), 2);
        let row = &result.rows[0];
        assert_eq!(row.model, "deepseek-v4-flash");
        assert_eq!(row.cost, 300.0);
        assert_eq!(row.quota_cost, 599.0);
        assert_eq!(row.multiplier, Some(2.0));
        assert!(row.estimated);
        assert_eq!(
            get_model_quota_limit(result.limit, row.multiplier),
            Some(3_000.0)
        );
        assert_eq!(result.usage, 1_050.0);
        assert_eq!(result.usage_percent, 17.5);
        let total: f64 = result.rows.iter().map(|row| row.contribution_percent).sum();
        assert!(approx(total, result.usage_percent));
        assert_eq!(sources, original);
    }
}

#[test]
fn keeps_distinct_model_ids_with_the_same_display_name_separate() {
    let result = build_lite_usage_breakdown(
        300.0,
        1_000.0,
        &[
            src("first", "Model", 100.0, 100.0, Some(1.0), false),
            src("second", "Model", 200.0, 200.0, Some(1.0), false),
        ],
    );

    assert_eq!(
        result
            .rows
            .iter()
            .map(|row| row.model.as_str())
            .collect::<Vec<_>>(),
        vec!["second", "first"]
    );
}

#[test]
fn does_not_merge_unknown_rates_with_recorded_rates() {
    let result = build_lite_usage_breakdown(
        300.0,
        1_000.0,
        &[
            src("glm", "GLM", 100.0, 100.0, None, true),
            src("glm", "GLM", 200.0, 200.0, Some(1.0), false),
        ],
    );

    assert_eq!(
        result
            .rows
            .iter()
            .map(|row| row.multiplier)
            .collect::<Vec<_>>(),
        vec![Some(1.0), None]
    );
    assert_eq!(
        get_model_quota_limit(result.limit, result.rows[1].multiplier),
        None
    );
}
