//! Model/provider normalization shared by the stats inference pipeline.
//!
//! Derived from the observable behaviour pinned by
//! `packages/stats/core/src/domain/model-normalization.ts` (upstream 18ef3cc).

/// Ordered author rules: the first match on the normalized model id wins.
pub const MODEL_AUTHOR_RULES: &[(&str, &str)] = &[
    ("claude", "anthropic"),
    ("gemini", "google"),
    ("deepseek", "deepseek"),
    ("glm", "zhipu"),
    ("gpt", "openai"),
    ("grok", "xai"),
    ("hy3", "tencent"),
    ("kimi", "moonshot"),
    ("mimo", "xiaomi"),
    ("minimax", "minimax"),
    ("muse-spark", "meta"),
    ("nemotron", "nvidia"),
    ("qwen", "qwen"),
];

/// Models excluded from public statistics entirely.
pub const EXCLUDED_MODELS: &[&str] = &["alpha-gpt-next"];

/// Models whose route provider is never exposed.
pub const STEALTH_MODELS: &[&str] = &["omen-alpha", "union-alpha"];

/// Models treated as the free tier regardless of the reported tier.
pub const FREE_MODELS: &[&str] = &["gpt-5-nano", "grok-code", "big-pickle"];

/// Model ids longer than this collapse to `unknown`.
pub const MODEL_NAME_MAX_LENGTH: usize = 256;

/// Current-name aliases for renamed or namespaced model ids.
pub const MODEL_NAME_ALIASES: &[(&str, &str)] = &[
    ("deepseek-flash", "deepseek-v4.1-flash"),
    ("deepseek-v4-flash-0731", "deepseek-v4-flash"),
    (
        "deepseek-v4-flash-dsv4-flash-final-rnaovd",
        "deepseek-v4-flash",
    ),
    ("opencode-go/union-alpha", "union-alpha"),
    ("opencode/union-alpha", "union-alpha"),
    ("ox-alpha", "glm-5.3-flash"),
    ("x-preview-f", "glm-5.3-flash"),
    ("xiaomi/mimo-v2.5", "mimo-v2.5"),
];

/// Retired model ids, used to decide whether a provider may be surfaced.
pub const RETIRED_STAT_MODELS: &[&str] = &[
    "big-pickle",
    "deepseek-flash",
    "deepseek-v4-flash-0731",
    "deepseek-v4-flash-dsv4-flash-final-rnaovd",
    "opencode-go/union-alpha",
    "opencode/union-alpha",
    "ox-alpha",
    "x-preview-f",
    "xiaomi/mimo-v2.5",
];

/// Retired providers never surfaced as the public provider.
pub const RETIRED_STAT_PROVIDERS: &[&str] = &["opencode"];

fn strip_trailing_variants(mut value: &str) -> &str {
    loop {
        let stripped = value
            .strip_suffix("-free")
            .or_else(|| value.strip_suffix(":free"))
            .or_else(|| value.strip_suffix(":global"));
        match stripped {
            Some(next) => value = next,
            None => return value,
        }
    }
}

/// Lower-case a model id and drop router/provider suffix variants.
pub fn normalize_inference_model(value: Option<&str>) -> String {
    let raw = value.unwrap_or("").to_lowercase();
    let stripped = strip_trailing_variants(&raw).to_string();
    if stripped.is_empty() {
        "unknown".to_string()
    } else {
        stripped
    }
}

/// Map a model id to its public author. `None` means the model is excluded.
pub fn model_author(value: Option<&str>) -> Option<&'static str> {
    let model = normalize_inference_model(value);
    if EXCLUDED_MODELS.contains(&model.as_str()) {
        return None;
    }
    Some(
        MODEL_AUTHOR_RULES
            .iter()
            .find(|(needle, _)| model.contains(*needle))
            .map(|(_, author)| *author)
            .unwrap_or("unknown"),
    )
}

fn alias_for(value: &str) -> Option<&'static str> {
    MODEL_NAME_ALIASES
        .iter()
        .find(|(from, _)| *from == value)
        .map(|(_, to)| *to)
}

/// Normalize a model id, resolving `big-pickle` through `provider_model`.
pub fn stat_model(model: Option<&str>, provider_model: Option<&str>) -> String {
    let normalized = normalize_inference_model(model);
    let resolved = if normalized == "big-pickle" {
        let last = provider_model.and_then(|value| value.rsplit('/').next());
        normalize_inference_model(last)
    } else {
        normalized
    };
    let value = alias_for(resolved.to_lowercase().as_str()).unwrap_or(resolved.as_str());
    if value.chars().count() > MODEL_NAME_MAX_LENGTH {
        "unknown".to_string()
    } else {
        value.to_string()
    }
}

/// Resolve the public provider for a model, preferring the provider-model author.
pub fn stat_provider(
    model: Option<&str>,
    provider_model: Option<&str>,
    provider: Option<&str>,
) -> Option<String> {
    let normalized = stat_model(model, provider_model);
    if STEALTH_MODELS.contains(&normalized.as_str()) {
        return Some("unknown".to_string());
    }

    let model_author_value = model_author(Some(normalized.as_str()))?;

    if let Some(provider_author) = model_author(provider_model) {
        if provider_author != "unknown" {
            return Some(provider_author.to_string());
        }
    }
    if model_author_value != "unknown" {
        return Some(model_author_value.to_string());
    }
    if let Some(provider) = provider {
        if !RETIRED_STAT_PROVIDERS.contains(&provider.to_lowercase().as_str()) {
            return Some(provider.to_string());
        }
    }
    Some(model_author_value.to_string())
}
