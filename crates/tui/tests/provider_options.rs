//! Port of packages/tui/test/cli/cmd/tui/provider-options.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/component/dialog-provider.tsx; see docs/TEST-PORT.md.

use opencode_tui::provider_options::{
    normalize_custom_provider_id, provider_options, CUSTOM_PROVIDER_OPTION_VALUE,
};
use std::collections::HashSet;

fn providers(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(id, name)| (id.to_string(), name.to_string()))
        .collect()
}

#[test]
fn includes_a_synthetic_other_option_for_custom_providers() {
    let options = provider_options(&providers(&[("openai", "OpenAI")]));
    let other = options.last().expect("other option");
    assert_eq!(other.title, "Other");
    assert_eq!(other.description.as_deref(), Some("Custom provider"));
    assert_eq!(other.category, "Providers");
}

#[test]
fn does_not_use_other_as_the_generic_provider_category() {
    let options = provider_options(&providers(&[("mistral", "Mistral")]));
    assert_eq!(options[0].category, "Providers");
}

#[test]
fn keeps_popular_providers_first_and_sorts_the_rest_alphabetically() {
    let options = provider_options(&providers(&[
        ("openai", "OpenAI"),
        ("custom-z", "Zebra Provider"),
        ("anthropic", "Anthropic"),
        ("mistral", "Mistral"),
        ("aws", "AWS Bedrock"),
    ]));

    assert_eq!(
        options
            .iter()
            .map(|option| option.value.clone())
            .collect::<Vec<_>>(),
        vec![
            "openai",
            "anthropic",
            "aws",
            "mistral",
            "custom-z",
            CUSTOM_PROVIDER_OPTION_VALUE,
        ]
    );
}

#[test]
fn does_not_collide_with_a_configured_provider_named_other() {
    let values: Vec<String> = provider_options(&providers(&[("other", "Other Provider")]))
        .into_iter()
        .map(|option| option.value)
        .collect();
    assert_eq!(values.iter().collect::<HashSet<_>>().len(), values.len());
}

#[test]
fn normalizes_and_validates_custom_provider_ids() {
    assert_eq!(
        normalize_custom_provider_id("  custom-provider  ").as_deref(),
        Some("custom-provider")
    );
    assert_eq!(
        normalize_custom_provider_id("custom_provider").as_deref(),
        Some("custom_provider")
    );
    assert_eq!(
        normalize_custom_provider_id("@ai-sdk/custom-provider").as_deref(),
        Some("custom-provider")
    );
    assert_eq!(normalize_custom_provider_id("-custom-provider"), None);
    assert_eq!(normalize_custom_provider_id("Custom Provider"), None);
}
