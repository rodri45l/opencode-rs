//! Port of packages/core/test/plugin/provider-dynamic.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the plugin creates an SDK from a provider factory export,
//! never overrides an SDK already supplied by an earlier plugin, injects the
//! provider id as the SDK factory name, uses the model API id for the default
//! language model, and loads npm packages through their resolved import
//! entrypoint. Re-derived: the `AISDK`/`PluginHost` service wiring, dynamic
//! `import`, and module-load error wrapping (the missing-entrypoint /
//! bad-import / missing-factory cases) are dropped.

use opencode_core::provider_dynamic::DynamicProviderPlugin;

const NOTE: &str = "porting: dynamic provider plugin not implemented";

#[test]
#[ignore = "porting: dynamic provider plugin not implemented"]
fn does_not_override_an_sdk_already_supplied_by_an_earlier_plugin() {
    assert!(!DynamicProviderPlugin::should_override_sdk(true).expect(NOTE));
    assert!(DynamicProviderPlugin::should_override_sdk(false).expect(NOTE));
}

#[test]
#[ignore = "porting: dynamic provider plugin not implemented"]
fn injects_the_provider_id_as_the_sdk_factory_name() {
    assert_eq!(
        DynamicProviderPlugin::sdk_name("custom-provider").expect(NOTE),
        "custom-provider"
    );
}

#[test]
#[ignore = "porting: dynamic provider plugin not implemented"]
fn uses_the_model_api_id_for_the_default_language_model() {
    assert_eq!(
        DynamicProviderPlugin::default_language_model_id("test-model-api").expect(NOTE),
        "test-model-api"
    );
}

#[test]
#[ignore = "porting: dynamic provider plugin not implemented"]
fn loads_npm_packages_through_their_resolved_import_entrypoint() {
    assert_eq!(
        DynamicProviderPlugin::import_source("fixture-provider", Some("/tmp/provider.mjs"))
            .expect(NOTE),
        "/tmp/provider.mjs"
    );
    assert_eq!(
        DynamicProviderPlugin::import_source("fixture-provider", None).expect(NOTE),
        "fixture-provider"
    );
}
