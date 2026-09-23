//! Go (lite) usage breakdown.
//!
//! Port of `packages/console/app/src/lib/lite-usage.ts` (upstream 18ef3cc): the
//! model quota limit is the window limit divided by the multiplier; usage is
//! grouped into rounded contribution percentages that sum to the usage percent.

/// A usage source row.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageSource {
    pub model: String,
    pub name: String,
    pub cost: f64,
    pub quota_cost: f64,
    pub multiplier: Option<f64>,
    pub estimated: bool,
}

/// A grouped usage row with its contribution.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageRow {
    pub model: String,
    pub name: String,
    pub cost: f64,
    pub quota_cost: f64,
    pub multiplier: Option<f64>,
    pub estimated: bool,
    pub contribution_percent: f64,
}

/// The usage breakdown.
#[derive(Debug, Clone, PartialEq)]
pub struct Breakdown {
    pub usage: f64,
    pub limit: f64,
    pub usage_percent: f64,
    pub rows: Vec<UsageRow>,
}

/// The quota limit for a given multiplier, or `None` when unknown.
pub fn get_model_quota_limit(limit: f64, multiplier: Option<f64>) -> Option<f64> {
    match multiplier {
        Some(multiplier) if multiplier > 0.0 => Some(limit / multiplier),
        _ => None,
    }
}

/// The rounded usage percent (`Math.round(amount / limit * 1000) / 10`).
pub fn get_usage_percent(amount: f64, limit: f64) -> f64 {
    if limit == 0.0 {
        return 0.0;
    }
    (amount / limit * 1000.0).round() / 10.0
}

/// Build the lite usage breakdown.
pub fn build_lite_usage_breakdown(usage: f64, limit: f64, sources: &[UsageSource]) -> Breakdown {
    let mut groups: Vec<UsageSource> = Vec::new();
    for item in sources {
        let key = format!(
            "[{},{}]",
            serde_json::to_string(&item.model).unwrap_or_default(),
            match item.multiplier {
                Some(value) => serde_json::to_string(&value).unwrap_or_default(),
                None => "null".to_string(),
            }
        );
        match groups.iter_mut().find(|group| {
            let group_key = format!(
                "[{},{}]",
                serde_json::to_string(&group.model).unwrap_or_default(),
                match group.multiplier {
                    Some(value) => serde_json::to_string(&value).unwrap_or_default(),
                    None => "null".to_string(),
                }
            );
            group_key == key
        }) {
            Some(group) => {
                group.cost += item.cost;
                group.quota_cost += item.quota_cost;
                group.estimated = group.estimated || item.estimated;
            }
            None => groups.push(item.clone()),
        }
    }

    let mut rows: Vec<UsageRow> = groups
        .into_iter()
        .filter(|item| item.cost != 0.0 || item.quota_cost != 0.0)
        .map(|item| UsageRow {
            model: item.model,
            name: item.name,
            cost: item.cost,
            quota_cost: item.quota_cost,
            multiplier: item.multiplier,
            estimated: item.estimated,
            contribution_percent: 0.0,
        })
        .collect();
    rows.sort_by(|a, b| {
        b.quota_cost
            .partial_cmp(&a.quota_cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let usage_percent = get_usage_percent(usage, limit);
    let target = (usage_percent * 10.0).round().max(0.0) as i64;
    let total_quota: f64 = rows.iter().map(|row| row.quota_cost.max(0.0)).sum();
    let mut units: Vec<(f64, i64)> = rows
        .iter()
        .map(|row| {
            let exact = if total_quota == 0.0 {
                0.0
            } else {
                (row.quota_cost.max(0.0) / total_quota) * target as f64
            };
            (exact, exact.floor() as i64)
        })
        .collect();
    let assigned: i64 = units.iter().map(|(_, value)| *value).sum();
    let remaining = target - assigned;
    let mut ranked: Vec<usize> = (0..units.len()).collect();
    ranked.sort_by(|&a, &b| {
        let fa = units[a].0 - units[a].1 as f64;
        let fb = units[b].0 - units[b].1 as f64;
        fb.partial_cmp(&fa).unwrap_or(std::cmp::Ordering::Equal)
    });
    if !ranked.is_empty() {
        for index in 0..remaining.max(0) as usize {
            units[ranked[index % ranked.len()]].1 += 1;
        }
    }
    for (row, (_, value)) in rows.iter_mut().zip(units.iter()) {
        row.contribution_percent = *value as f64 / 10.0;
    }

    Breakdown {
        usage,
        limit,
        usage_percent,
        rows,
    }
}
