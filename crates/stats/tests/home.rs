//! Port of packages/stats/core/src/domain/home.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/stats/core/src/domain/home.ts; see docs/TEST-PORT.md.

use opencode_stats::home::{build_retention_entries, RetentionMetricRow};

fn cohorts(
    model: &str,
    provider: &str,
    count: usize,
    eligible_users: i64,
    retained_users: i64,
) -> Vec<RetentionMetricRow> {
    (0..count)
        .map(|index| RetentionMetricRow {
            cohort_date: format!("2026-08-{:02}", index + 1),
            updated_at: 0,
            provider: provider.to_string(),
            model: model.to_string(),
            eligible_users,
            retained_users,
        })
        .collect()
}

#[test]
fn pools_the_latest_seven_weekly_cohorts_and_ranks_models_above_the_sample_floor() {
    let mut rows = Vec::new();
    rows.extend(cohorts("model-a", "provider-a", 8, 20, 10));
    rows.extend(cohorts("model-b", "provider-b", 8, 20, 12));
    rows.extend(cohorts("small-model", "provider-c", 8, 10, 9));

    let entries = build_retention_entries(&rows);

    let model_a = entries.iter().find(|item| item.model == "model-a").unwrap();
    assert_eq!(model_a.eligible_user_weeks, 140);
    assert_eq!(model_a.retained_user_weeks, 70);
    assert_eq!(model_a.rate, 50.0);
    assert_eq!(model_a.rank, Some(2));

    let model_b = entries.iter().find(|item| item.model == "model-b").unwrap();
    assert_eq!(model_b.eligible_user_weeks, 140);
    assert_eq!(model_b.retained_user_weeks, 84);
    assert_eq!(model_b.rate, 60.0);
    assert_eq!(model_b.rank, Some(1));

    let small = entries
        .iter()
        .find(|item| item.model == "small-model")
        .unwrap();
    assert_eq!(small.eligible_user_weeks, 70);
    assert_eq!(small.retained_user_weeks, 63);
    assert_eq!(small.rate, 90.0);
    assert_eq!(small.rank, None);
}
