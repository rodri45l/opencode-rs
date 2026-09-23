//! Token estimation and usage normalization.
//!
//! Ports the observable behaviour of `packages/opencode/src/util/token.ts`
//! (`Token.estimate`) and `getUsage` in
//! `packages/opencode/src/session/session.ts`: cache read/write extraction,
//! reasoning split, context cost tiers, the over-200k fallback, and the
//! authoritative Copilot billed cost.

/// A provider usage report.
#[derive(Debug, Clone, Copy, Default)]
pub struct Usage {
    /// Prompt tokens.
    pub input_tokens: u64,
    /// Completion tokens.
    pub output_tokens: u64,
    /// Total tokens.
    pub total_tokens: u64,
    /// Reasoning tokens, when reported.
    pub reasoning_tokens: Option<u64>,
    /// Cache-read tokens, when reported.
    pub cache_read_input_tokens: Option<u64>,
}

/// Provider-specific cache/cost metadata.
#[derive(Debug, Clone, Copy, Default)]
pub struct Metadata {
    /// Anthropic cache-creation tokens.
    pub anthropic_cache_creation: Option<u64>,
    /// Bedrock cache-write tokens.
    pub bedrock_cache_write: Option<u64>,
    /// Vertex cache-creation tokens.
    pub vertex_cache_creation: Option<u64>,
    /// Copilot authoritative billed nano-AIU.
    pub copilot_total_nano_aiu: Option<u64>,
}

/// Per-million-token pricing.
#[derive(Debug, Clone, Copy, Default)]
pub struct Cost {
    /// Input rate.
    pub input: f64,
    /// Output rate.
    pub output: f64,
    /// Cache-read rate.
    pub cache_read: f64,
    /// Cache-write rate.
    pub cache_write: f64,
}

/// A context-size pricing tier.
#[derive(Debug, Clone, Copy, Default)]
pub struct Tier {
    /// Minimum context size for the tier.
    pub size: u64,
    /// Tier pricing.
    pub cost: Cost,
}

/// The model pricing/context data usage normalization consumes.
#[derive(Debug, Clone, Default)]
pub struct UsageModel {
    /// Context window.
    pub context: u64,
    /// Maximum output.
    pub output: u64,
    /// NPM package id.
    pub npm: &'static str,
    /// Base pricing.
    pub cost: Cost,
    /// Context tiers.
    pub tiers: Vec<Tier>,
    /// Over-200k fallback pricing.
    pub over_200k: Option<Cost>,
    /// Whether the base input rate is malformed.
    pub malformed_cost_input: bool,
}

/// Normalized usage totals and computed cost.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UsageTotals {
    /// Non-cached input tokens.
    pub input: u64,
    /// Non-reasoning output tokens.
    pub output: u64,
    /// Reasoning tokens.
    pub reasoning: u64,
    /// Cache-read tokens.
    pub cache_read: u64,
    /// Cache-write tokens.
    pub cache_write: u64,
    /// Total tokens.
    pub total: u64,
    /// Computed cost.
    pub cost: f64,
}

/// Estimate tokens as `round(chars / 4)`.
pub fn token_estimate(text: &str) -> u64 {
    let chars = text.chars().count() as f64;
    (chars / 4.0).round().max(0.0) as u64
}

/// Normalize provider usage into token totals and a computed cost.
pub fn get_usage(model: &UsageModel, usage: &Usage, metadata: &Metadata) -> UsageTotals {
    fn finite(value: f64) -> f64 {
        if value.is_finite() {
            value
        } else {
            0.0
        }
    }
    fn safe(value: f64) -> f64 {
        finite(value).max(0.0)
    }

    let input_tokens = safe(usage.input_tokens as f64);
    let output_tokens = safe(usage.output_tokens as f64);
    let reasoning_tokens = safe(usage.reasoning_tokens.unwrap_or(0) as f64);
    let cache_read = safe(usage.cache_read_input_tokens.unwrap_or(0) as f64);
    let cache_write = safe(
        metadata
            .anthropic_cache_creation
            .or(metadata.vertex_cache_creation)
            .or(metadata.bedrock_cache_write)
            .unwrap_or(0) as f64,
    );

    let adjusted_input = safe(input_tokens - cache_read - cache_write);
    let output = safe(output_tokens - reasoning_tokens);
    let total = usage.total_tokens;

    let context_tokens = input_tokens;
    let tier_cost = model
        .tiers
        .iter()
        .filter(|tier| context_tokens > tier.size as f64)
        .max_by_key(|tier| tier.size)
        .map(|tier| tier.cost);
    let fallback = if context_tokens > 200_000.0 {
        model.over_200k
    } else {
        None
    };
    let cost_info = tier_cost.or(fallback).unwrap_or(model.cost);

    let input_rate = if model.malformed_cost_input {
        0.0
    } else {
        finite(cost_info.input)
    };
    let output_rate = finite(cost_info.output);
    let cache_read_rate = finite(cost_info.cache_read);
    let cache_write_rate = finite(cost_info.cache_write);

    let copilot = metadata.copilot_total_nano_aiu;
    let cost = match copilot {
        Some(nano) if (nano as f64).is_finite() => (nano as f64) / 100_000_000_000.0,
        _ => safe(
            adjusted_input * input_rate / 1_000_000.0
                + output * output_rate / 1_000_000.0
                + cache_read * cache_read_rate / 1_000_000.0
                + cache_write * cache_write_rate / 1_000_000.0
                + reasoning_tokens * output_rate / 1_000_000.0,
        ),
    };

    UsageTotals {
        input: adjusted_input as u64,
        output: output as u64,
        reasoning: reasoning_tokens as u64,
        cache_read: cache_read as u64,
        cache_write: cache_write as u64,
        total,
        cost,
    }
}
