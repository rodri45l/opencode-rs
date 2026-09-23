//! Port of packages/core/test/plugin/provider-github-copilot.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the GitHub Copilot plugin binds only to the exact
//! `@ai-sdk/github-copilot` package, selects `responses` for `gpt-5` models
//! except the `gpt-5-mini*` variants (which use `chat`), falls back to
//! `languageModel` when neither is advertised, uses the model API id, and
//! disables `gpt-5-chat-latest` for the exact `github-copilot` provider only.
//! Re-derived: the `AISDK`/`Catalog` service wiring, advertised endpoint
//! metadata and SDK factory are dropped.

use opencode_core::provider_sdk_plugins::{
    LanguageQuery, LanguageSelection, LanguageSelector, ProviderSdkPlugins, SdkCapabilities,
};

fn caps_all() -> SdkCapabilities {
    SdkCapabilities {
        responses: true,
        messages: true,
        chat: true,
        language_model: true,
    }
}

fn query<'a>(
    model_id: &'a str,
    api_id: &'a str,
    capabilities: SdkCapabilities,
) -> LanguageQuery<'a> {
    LanguageQuery {
        plugin: "github-copilot",
        provider_id: "github-copilot",
        model_id,
        api_id,
        capabilities,
        use_completion_urls: false,
    }
}

#[test]
fn binds_only_to_the_exact_copilot_package() {
    assert!(
        ProviderSdkPlugins::matches_package("github-copilot", "@ai-sdk/github-copilot").unwrap()
    );
    assert!(
        !ProviderSdkPlugins::matches_package("github-copilot", "@ai-sdk/openai-compatible")
            .unwrap()
    );
}

#[test]
fn selects_routes_by_model_id() {
    let cases = [
        ("gpt-5", LanguageSelector::Responses),
        ("gpt-5.1-codex", LanguageSelector::Responses),
        ("gpt-4o", LanguageSelector::Chat),
        ("gpt-5-mini", LanguageSelector::Chat),
        ("gpt-5-mini-2025-08-07", LanguageSelector::Chat),
    ];
    for (model, selector) in cases {
        assert_eq!(
            ProviderSdkPlugins::select_language(&query(model, model, caps_all())).unwrap(),
            Some(LanguageSelection {
                selector,
                model_id: model.to_string(),
            })
        );
    }
}

#[test]
fn falls_back_to_language_model_when_absent() {
    let capabilities = SdkCapabilities {
        language_model: true,
        ..SdkCapabilities::default()
    };
    assert_eq!(
        ProviderSdkPlugins::select_language(&query(
            "claude-sonnet-4",
            "claude-sonnet-4",
            capabilities
        ))
        .unwrap(),
        Some(LanguageSelection {
            selector: LanguageSelector::LanguageModel,
            model_id: "claude-sonnet-4".to_string(),
        })
    );
}

#[test]
fn uses_the_model_api_id_for_selection() {
    assert_eq!(
        ProviderSdkPlugins::select_language(&query("default", "gpt-5", caps_all())).unwrap(),
        Some(LanguageSelection {
            selector: LanguageSelector::Responses,
            model_id: "gpt-5".to_string(),
        })
    );
    assert_eq!(
        ProviderSdkPlugins::select_language(&query("small", "gpt-5-mini", caps_all())).unwrap(),
        Some(LanguageSelection {
            selector: LanguageSelector::Chat,
            model_id: "gpt-5-mini".to_string(),
        })
    );
    assert_eq!(
        ProviderSdkPlugins::select_language(&query("sonnet", "claude-sonnet-4", caps_all()))
            .unwrap(),
        Some(LanguageSelection {
            selector: LanguageSelector::Chat,
            model_id: "claude-sonnet-4".to_string(),
        })
    );
}

#[test]
fn disables_gpt_5_chat_latest_for_the_exact_copilot_provider() {
    assert!(ProviderSdkPlugins::disables_model(
        "github-copilot",
        "github-copilot",
        "gpt-5-chat-latest"
    )
    .unwrap());
    assert!(!ProviderSdkPlugins::disables_model(
        "github-copilot",
        "custom-copilot",
        "gpt-5-chat-latest"
    )
    .unwrap());
}
