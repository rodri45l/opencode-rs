//! Port of packages/core/test/plugin/provider-sap-ai-core.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a configured `serviceKey` is written to `AICORE_SERVICE_KEY`
//! unless one is already present, the SDK keeps only `deploymentId` and
//! `resourceGroup`, and both are omitted when no service key is available.
//! Re-derived: the `AISDK`/`Npm` service wiring, the callable language selector
//! and the SDK factory are dropped.

use std::collections::BTreeMap;

use opencode_core::provider_sap_ai_core::{ResolvedSapAICore, SapAICorePlugin};

const NOTE: &str = "porting: sap-ai-core provider plugin not implemented";

fn env(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

#[test]
fn copies_a_configured_service_key_into_the_environment() {
    let resolved = SapAICorePlugin::resolve(
        &env(&[
            ("AICORE_DEPLOYMENT_ID", "deployment"),
            ("AICORE_RESOURCE_GROUP", "resource-group"),
        ]),
        Some("service-key"),
    )
    .expect(NOTE);

    assert_eq!(
        resolved,
        ResolvedSapAICore {
            service_key: Some("service-key".to_string()),
            deployment_id: Some("deployment".to_string()),
            resource_group: Some("resource-group".to_string()),
        }
    );
}

#[test]
fn preserves_an_existing_environment_service_key() {
    let resolved = SapAICorePlugin::resolve(
        &env(&[
            ("AICORE_SERVICE_KEY", "env-service-key"),
            ("AICORE_DEPLOYMENT_ID", "deployment"),
            ("AICORE_RESOURCE_GROUP", "resource-group"),
        ]),
        Some("option-service-key"),
    )
    .expect(NOTE);

    assert_eq!(resolved.service_key.as_deref(), Some("env-service-key"));
    assert_eq!(resolved.deployment_id.as_deref(), Some("deployment"));
    assert_eq!(resolved.resource_group.as_deref(), Some("resource-group"));
}

#[test]
fn omits_deployment_and_resource_group_without_a_service_key() {
    let resolved = SapAICorePlugin::resolve(
        &env(&[
            ("AICORE_DEPLOYMENT_ID", "deployment"),
            ("AICORE_RESOURCE_GROUP", "resource-group"),
        ]),
        None,
    )
    .expect(NOTE);

    assert_eq!(
        resolved,
        ResolvedSapAICore {
            service_key: None,
            deployment_id: None,
            resource_group: None,
        }
    );
}
