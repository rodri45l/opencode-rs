//! Port of packages/opencode/test/provider/model-status.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: catalog status never accepts `active`, normalized status
//! does, public provider schemas accept `active`, and catalog models omit
//! status entirely.

use opencode_server::port::provider::{
    CatalogModelStatus, ConfigProviderModel, ModelStatus, ModelsDevModel, ProviderModel,
};
use serde_json::json;

#[test]
fn keeps_catalog_status_separate_from_normalized_provider_status() {
    assert_eq!(
        CatalogModelStatus::parse("deprecated"),
        Ok(CatalogModelStatus::Deprecated)
    );
    assert!(CatalogModelStatus::parse("active").is_err());
    assert_eq!(ModelStatus::parse("active"), Ok(ModelStatus::Active));
}

#[test]
fn accepts_active_status_across_public_provider_schemas() {
    let config: ConfigProviderModel =
        serde_json::from_value(json!({ "status": "active" })).expect("config provider decodes");
    assert_eq!(config.status, ModelStatus::Active);

    let models_dev: ModelsDevModel = serde_json::from_value(json!({
        "id": "test-model",
        "name": "Test Model",
        "release_date": "2026-01-01",
        "attachment": false,
        "reasoning": false,
        "temperature": true,
        "tool_call": true,
        "limit": { "context": 128000, "output": 8192 },
    }))
    .expect("models.dev entry decodes");
    assert_eq!(models_dev.status, None);

    let provider: ProviderModel = serde_json::from_value(json!({
        "id": "test-model",
        "providerID": "test-provider",
        "name": "Test Model",
        "status": "active",
        "limit": { "context": 128000, "output": 8192 },
    }))
    .expect("provider model decodes");
    assert_eq!(provider.status, "active");
}
