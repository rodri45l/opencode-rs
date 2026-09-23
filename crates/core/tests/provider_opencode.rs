//! Port of packages/core/test/plugin/provider-opencode.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the plugin only rewrites the `opencode` provider, uses the
//! public API key when no credential is available, disables paid models without
//! credentials while keeping free and output-only models enabled, enables every
//! model when a credential exists (`OPENCODE_API_KEY`, a configured env method,
//! or a configured `apiKey`), and prefers `gpt-5-nano` as the small model.
//! Re-derived: the `Catalog`/`Credential`/`Integration`/`PluginHost` wiring, the
//! device-code OAuth flow, and the remote provider fetch are dropped.

use opencode_core::provider_opencode::OpencodePlugin;

#[test]
fn only_rewrites_the_opencode_provider() {
    assert!(OpencodePlugin::matches_provider("opencode").unwrap());
    assert!(!OpencodePlugin::matches_provider("openai").unwrap());
}

#[test]
fn uses_the_public_key_and_disables_paid_models_without_credentials() {
    let credentials = OpencodePlugin::has_credentials(None, false, None).unwrap();
    assert!(!credentials);
    assert_eq!(
        OpencodePlugin::api_key(credentials, None)
            .unwrap()
            .as_deref(),
        Some("public")
    );
    assert!(!OpencodePlugin::model_enabled(credentials, 1.0).unwrap());
}

#[test]
fn keeps_free_and_output_only_models_enabled_without_credentials() {
    let credentials = OpencodePlugin::has_credentials(None, false, None).unwrap();
    assert!(OpencodePlugin::model_enabled(credentials, 0.0).unwrap());
    assert!(!OpencodePlugin::requires_credentials(0.0).unwrap());
}

#[test]
fn treats_any_credential_source_as_authentication() {
    assert!(OpencodePlugin::has_credentials(Some("secret"), false, None).unwrap());
    assert!(OpencodePlugin::has_credentials(None, true, None).unwrap());
    assert!(OpencodePlugin::has_credentials(None, false, Some("configured")).unwrap());
}

#[test]
fn uses_the_configured_api_key_and_enables_paid_models_when_authenticated() {
    let credentials = OpencodePlugin::has_credentials(None, false, Some("configured")).unwrap();
    assert_eq!(
        OpencodePlugin::api_key(credentials, Some("configured"))
            .unwrap()
            .as_deref(),
        Some("configured")
    );
    assert!(OpencodePlugin::api_key(credentials, None)
        .unwrap()
        .is_none());
    assert!(OpencodePlugin::model_enabled(credentials, 1.0).unwrap());
}

#[test]
fn prefers_gpt_5_nano_as_the_small_model() {
    assert_eq!(
        OpencodePlugin::small_model(&["cheap-mini", "gpt-5-nano"])
            .unwrap()
            .as_deref(),
        Some("gpt-5-nano")
    );
}
