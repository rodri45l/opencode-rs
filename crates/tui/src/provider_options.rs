//! Provider option ordering and custom-provider validation.
//!
//! Port of packages/tui/src/component/dialog-provider.tsx
//! `providerOptions`/`normalizeCustomProviderID` behaviour (upstream 18ef3cc).

/// The synthetic custom-provider option value.
pub const CUSTOM_PROVIDER_OPTION_VALUE: &str = "__opencode_custom_provider__";

/// A provider selection option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderOption {
    pub kind: String,
    pub title: String,
    pub value: String,
    pub provider_id: Option<String>,
    pub description: Option<String>,
    pub category: String,
}

fn priority(id: &str) -> u32 {
    match id {
        "opencode" => 0,
        "opencode-go" => 1,
        "openai" => 2,
        "github-copilot" => 3,
        "anthropic" => 4,
        "google" => 5,
        _ => 99,
    }
}

fn description(id: &str) -> Option<String> {
    match id {
        "opencode" => Some("(Recommended)".to_string()),
        "anthropic" => Some("(API key)".to_string()),
        "openai" => Some("(ChatGPT Plus/Pro or API key)".to_string()),
        "opencode-go" => Some("Low cost subscription for everyone".to_string()),
        _ => None,
    }
}

/// Build the provider option list with the synthetic Other entry appended.
pub fn provider_options(list: &[(String, String)]) -> Vec<ProviderOption> {
    let mut sorted: Vec<&(String, String)> = list.iter().collect();
    sorted.sort_by(|left, right| {
        priority(&left.0)
            .cmp(&priority(&right.0))
            .then_with(|| left.1.to_lowercase().cmp(&right.1.to_lowercase()))
            .then_with(|| left.0.cmp(&right.0))
    });

    let mut options: Vec<ProviderOption> = sorted
        .into_iter()
        .map(|(id, name)| ProviderOption {
            kind: "provider".to_string(),
            title: name.clone(),
            value: id.clone(),
            provider_id: Some(id.clone()),
            description: description(id),
            category: if priority(id) != 99 {
                "Popular".to_string()
            } else {
                "Providers".to_string()
            },
        })
        .collect();

    options.push(ProviderOption {
        kind: "custom".to_string(),
        title: "Other".to_string(),
        value: CUSTOM_PROVIDER_OPTION_VALUE.to_string(),
        provider_id: None,
        description: Some("Custom provider".to_string()),
        category: "Providers".to_string(),
    });
    options
}

/// Normalize and validate a custom provider id.
pub fn normalize_custom_provider_id(value: &str) -> Option<String> {
    let provider_id = value
        .trim()
        .strip_prefix("@ai-sdk/")
        .unwrap_or(value.trim());
    if is_valid_provider_id(provider_id) {
        Some(provider_id.to_string())
    } else {
        None
    }
}

fn is_valid_provider_id(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() || first.is_ascii_digit() => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
}
