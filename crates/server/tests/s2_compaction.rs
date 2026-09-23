#![allow(dead_code)]

//! Port of packages/opencode/test/session/compaction.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the pure `util.token.estimate` and `SessionNs.getUsage`
//! cases only (token normalization, cache read/write extraction, reasoning
//! split, cost calculation, context cost tiers, Copilot billed cost). The
//! reference also drives `SessionCompaction.create`/`prune`/`process` against
//! the live session database, projector, and LLM stream; those are dropped here
//! and noted in PORT-STATUS.s2.json.
//!
//! Stubs are local to this file per the fast-wave protocol and return a typed
//! error until the module lands.

use opencode_server::session_usage::{
    get_usage, token_estimate, Cost, Metadata, Tier, Usage, UsageModel,
};

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn model(context: u64, output: u64, cost: Cost) -> UsageModel {
    UsageModel {
        context,
        output,
        cost,
        ..UsageModel::default()
    }
}

fn base_cost() -> Cost {
    Cost {
        input: 3.0,
        output: 15.0,
        cache_read: 0.3,
        cache_write: 3.75,
    }
}

#[test]
fn estimates_tokens_from_text_at_four_chars_per_token() {
    let text = "x".repeat(4000);
    assert_eq!(token_estimate(&text), 1000);
}

#[test]
fn estimates_tokens_from_larger_text() {
    let text = "y".repeat(20_000);
    assert_eq!(token_estimate(&text), 5000);
}

#[test]
fn token_estimate_returns_zero_for_empty_string() {
    assert_eq!(token_estimate(""), 0);
}

#[test]
fn normalizes_standard_usage_to_token_format() {
    let result = get_usage(
        &model(100_000, 32_000, Cost::default()),
        &Usage {
            input_tokens: 1000,
            output_tokens: 500,
            total_tokens: 1500,
            ..Usage::default()
        },
        &Metadata::default(),
    );

    assert_eq!(result.input, 1000);
    assert_eq!(result.output, 500);
    assert_eq!(result.reasoning, 0);
    assert_eq!(result.cache_read, 0);
    assert_eq!(result.cache_write, 0);
}

#[test]
fn extracts_cached_tokens_to_cache_read() {
    let result = get_usage(
        &model(100_000, 32_000, Cost::default()),
        &Usage {
            input_tokens: 1000,
            output_tokens: 500,
            total_tokens: 1500,
            cache_read_input_tokens: Some(200),
            ..Usage::default()
        },
        &Metadata::default(),
    );

    assert_eq!(result.input, 800);
    assert_eq!(result.cache_read, 200);
}

#[test]
fn handles_anthropic_cache_write_metadata() {
    let result = get_usage(
        &model(100_000, 32_000, Cost::default()),
        &Usage {
            input_tokens: 1000,
            output_tokens: 500,
            total_tokens: 1500,
            ..Usage::default()
        },
        &Metadata {
            anthropic_cache_creation: Some(300),
            ..Metadata::default()
        },
    );

    assert_eq!(result.cache_write, 300);
}

#[test]
fn subtracts_cached_tokens_for_anthropic_provider() {
    let result = get_usage(
        &model(100_000, 32_000, Cost::default()),
        &Usage {
            input_tokens: 1000,
            output_tokens: 500,
            total_tokens: 1500,
            cache_read_input_tokens: Some(200),
            ..Usage::default()
        },
        &Metadata::default(),
    );

    assert_eq!(result.input, 800);
    assert_eq!(result.cache_read, 200);
}

#[test]
fn separates_reasoning_tokens_from_output_tokens() {
    let result = get_usage(
        &model(100_000, 32_000, Cost::default()),
        &Usage {
            input_tokens: 1000,
            output_tokens: 500,
            total_tokens: 1500,
            reasoning_tokens: Some(100),
            ..Usage::default()
        },
        &Metadata::default(),
    );

    assert_eq!(result.input, 1000);
    assert_eq!(result.output, 400);
    assert_eq!(result.reasoning, 100);
    assert_eq!(result.total, 1500);
}

#[test]
fn does_not_double_count_reasoning_tokens_in_cost() {
    let cost = Cost {
        output: 15.0,
        ..Cost::default()
    };
    let result = get_usage(
        &model(100_000, 32_000, cost),
        &Usage {
            input_tokens: 0,
            output_tokens: 1_000_000,
            total_tokens: 1_000_000,
            reasoning_tokens: Some(250_000),
            ..Usage::default()
        },
        &Metadata::default(),
    );

    assert_eq!(result.output, 750_000);
    assert_eq!(result.reasoning, 250_000);
    assert!(approx(result.cost, 15.0), "cost {}", result.cost);
}

#[test]
fn handles_undefined_optional_values_gracefully() {
    let result = get_usage(
        &model(100_000, 32_000, Cost::default()),
        &Usage::default(),
        &Metadata::default(),
    );

    assert_eq!(result.input, 0);
    assert_eq!(result.output, 0);
    assert_eq!(result.reasoning, 0);
    assert_eq!(result.cache_read, 0);
    assert_eq!(result.cache_write, 0);
    assert!(!result.cost.is_nan());
}

#[test]
fn ignores_malformed_cost_fields() {
    let mut m = model(
        100_000,
        32_000,
        Cost {
            output: 15.0,
            cache_read: 0.3,
            cache_write: 3.75,
            ..Cost::default()
        },
    );
    m.malformed_cost_input = true;

    let result = get_usage(
        &m,
        &Usage {
            input_tokens: 1_000_000,
            output_tokens: 100_000,
            total_tokens: 1_100_000,
            ..Usage::default()
        },
        &Metadata::default(),
    );

    assert!(approx(result.cost, 1.5), "cost {}", result.cost);
}

#[test]
fn calculates_cost_correctly() {
    let result = get_usage(
        &model(100_000, 32_000, base_cost()),
        &Usage {
            input_tokens: 1_000_000,
            output_tokens: 100_000,
            total_tokens: 1_100_000,
            ..Usage::default()
        },
        &Metadata::default(),
    );

    assert!(approx(result.cost, 4.5), "cost {}", result.cost);
}

#[test]
fn uses_authoritative_copilot_billed_cost_when_provided() {
    let result = get_usage(
        &model(100_000, 32_000, base_cost()),
        &Usage {
            input_tokens: 11_774,
            output_tokens: 39,
            total_tokens: 11_813,
            ..Usage::default()
        },
        &Metadata {
            copilot_total_nano_aiu: Some(4_473_525_000),
            ..Metadata::default()
        },
    );

    assert!(approx(result.cost, 0.044_735_25), "cost {}", result.cost);
}

#[test]
fn uses_matching_context_cost_tier_before_over_200k_fallback() {
    let m = UsageModel {
        context: 1_000_000,
        output: 32_000,
        npm: "@ai-sdk/openai",
        cost: Cost {
            input: 1.0,
            output: 2.0,
            cache_read: 0.1,
            cache_write: 0.5,
        },
        tiers: vec![
            Tier {
                size: 200_000,
                cost: Cost {
                    input: 3.0,
                    output: 4.0,
                    cache_read: 0.3,
                    cache_write: 1.5,
                },
            },
            Tier {
                size: 500_000,
                cost: Cost {
                    input: 5.0,
                    output: 6.0,
                    cache_read: 0.5,
                    cache_write: 2.5,
                },
            },
        ],
        over_200k: Some(Cost {
            input: 100.0,
            output: 100.0,
            cache_read: 100.0,
            cache_write: 100.0,
        }),
        malformed_cost_input: false,
    };

    let result = get_usage(
        &m,
        &Usage {
            input_tokens: 650_000,
            output_tokens: 100_000,
            total_tokens: 750_000,
            cache_read_input_tokens: Some(100_000),
            ..Usage::default()
        },
        &Metadata::default(),
    );

    assert_eq!(result.input, 550_000);
    assert!(
        approx(result.cost, 2.75 + 0.6 + 0.05),
        "cost {}",
        result.cost
    );
}

#[test]
fn falls_back_to_over_200k_pricing_when_no_cost_tier_matches() {
    let m = UsageModel {
        context: 1_000_000,
        output: 32_000,
        npm: "@ai-sdk/openai",
        cost: Cost {
            input: 1.0,
            output: 2.0,
            cache_read: 0.1,
            cache_write: 0.5,
        },
        tiers: vec![Tier {
            size: 500_000,
            cost: Cost {
                input: 5.0,
                output: 6.0,
                cache_read: 0.5,
                cache_write: 2.5,
            },
        }],
        over_200k: Some(Cost {
            input: 3.0,
            output: 4.0,
            cache_read: 0.3,
            cache_write: 1.5,
        }),
        malformed_cost_input: false,
    };

    let result = get_usage(
        &m,
        &Usage {
            input_tokens: 300_000,
            output_tokens: 100_000,
            total_tokens: 400_000,
            ..Usage::default()
        },
        &Metadata::default(),
    );

    assert!(approx(result.cost, 0.9 + 0.4), "cost {}", result.cost);
}

#[test]
fn computes_total_from_components_for_anthropic_family_models() {
    for npm in [
        "@ai-sdk/anthropic",
        "@ai-sdk/amazon-bedrock",
        "@ai-sdk/google-vertex/anthropic",
    ] {
        let mut m = model(100_000, 32_000, Cost::default());
        m.npm = npm;

        let usage = Usage {
            input_tokens: 1000,
            output_tokens: 500,
            total_tokens: 1500,
            cache_read_input_tokens: Some(200),
            ..Usage::default()
        };

        let metadata = if npm == "@ai-sdk/amazon-bedrock" {
            Metadata {
                bedrock_cache_write: Some(300),
                ..Metadata::default()
            }
        } else {
            Metadata {
                anthropic_cache_creation: Some(300),
                ..Metadata::default()
            }
        };

        let result = get_usage(&m, &usage, &metadata);

        assert_eq!(result.input, 500, "npm {npm}");
        assert_eq!(result.cache_read, 200, "npm {npm}");
        assert_eq!(result.cache_write, 300, "npm {npm}");
        assert_eq!(result.total, 1500, "npm {npm}");
    }
}

#[test]
fn extracts_cache_write_tokens_from_vertex_metadata_key() {
    let mut m = model(100_000, 32_000, Cost::default());
    m.npm = "@ai-sdk/google-vertex/anthropic";

    let result = get_usage(
        &m,
        &Usage {
            input_tokens: 1000,
            output_tokens: 500,
            total_tokens: 1500,
            cache_read_input_tokens: Some(200),
            ..Usage::default()
        },
        &Metadata {
            vertex_cache_creation: Some(300),
            ..Metadata::default()
        },
    );

    assert_eq!(result.input, 500);
    assert_eq!(result.cache_read, 200);
    assert_eq!(result.cache_write, 300);
}
