//! Zen model/provider validation.
//!
//! Port of `packages/console/core/src/model.ts` (`ZenData.validate`, upstream
//! 18ef3cc): a `cost200K` tier without an explicit threshold defaults to
//! 200_000, and the provider `format` is preserved.

use std::collections::BTreeMap;

/// Token pricing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cost {
    pub input: i64,
    pub output: i64,
}

/// A long-context pricing tier before threshold resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cost200K {
    pub input: i64,
    pub output: i64,
    pub threshold: Option<i64>,
}

/// A long-context pricing tier with a resolved threshold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCost200K {
    pub input: i64,
    pub output: i64,
    pub threshold: i64,
}

/// A provider reference from a model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRef {
    pub id: String,
    pub model: String,
}

/// A model as configured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawZenModel {
    pub name: String,
    pub cost: Cost,
    pub cost_multiplier: i64,
    pub cost200k: Option<Cost200K>,
    pub providers: Vec<ProviderRef>,
}

/// A validated model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZenModel {
    pub name: String,
    pub cost: Cost,
    pub cost_multiplier: i64,
    pub cost200k: Option<ResolvedCost200K>,
    pub providers: Vec<ProviderRef>,
}

/// A validated provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    pub api: String,
    pub api_key: String,
    pub format: Option<String>,
}

/// The validated Zen model data set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZenData {
    pub zen_models: BTreeMap<String, ZenModel>,
    pub providers: BTreeMap<String, Provider>,
}

impl ZenData {
    /// Validate the raw model and provider maps.
    pub fn validate(
        zen_models: BTreeMap<String, RawZenModel>,
        providers: BTreeMap<String, Provider>,
    ) -> ZenData {
        let zen_models = zen_models
            .into_iter()
            .map(|(id, model)| {
                let cost200k = model.cost200k.map(|tier| ResolvedCost200K {
                    input: tier.input,
                    output: tier.output,
                    threshold: tier.threshold.unwrap_or(200_000),
                });
                (
                    id,
                    ZenModel {
                        name: model.name,
                        cost: model.cost,
                        cost_multiplier: model.cost_multiplier,
                        cost200k,
                        providers: model.providers,
                    },
                )
            })
            .collect();
        ZenData {
            zen_models,
            providers,
        }
    }
}
