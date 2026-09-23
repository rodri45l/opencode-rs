//! Port of packages/core/test/effect/observability.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `OTEL_RESOURCE_ATTRIBUTES` parses comma-separated
//! `key=value` pairs with percent-decoding; if any entry is invalid the whole
//! user-supplied set is dropped but built-ins remain; built-in attributes
//! (`opencode.client`, `service.instance.id`, `opencode.run`) win over
//! conflicting environment values. Re-derived: the Effect logger layer and the
//! file-logger tests are dropped; the pure resource resolution remains.

use opencode_core::observability::{resource, ObservabilityEnv};

const NOTE: &str = "porting: observability resource not implemented";

fn env(otel: &str) -> ObservabilityEnv {
    ObservabilityEnv {
        otel_resource_attributes: Some(otel.to_string()),
        opencode_client: None,
    }
}

#[test]
fn parses_and_decodes_otel_resource_attributes() {
    let resolved = resource(&env(
        "service.namespace=anomalyco,team=platform%2Cobservability,label=hello%3Dworld,key%2Fname=value%20here",
    ))
    .expect(NOTE);
    assert_eq!(
        resolved
            .attributes
            .get("service.namespace")
            .map(String::as_str),
        Some("anomalyco")
    );
    assert_eq!(
        resolved.attributes.get("team").map(String::as_str),
        Some("platform,observability")
    );
    assert_eq!(
        resolved.attributes.get("label").map(String::as_str),
        Some("hello=world")
    );
    assert_eq!(
        resolved.attributes.get("key/name").map(String::as_str),
        Some("value here")
    );
}

#[test]
fn drops_otel_resource_attributes_when_any_entry_is_invalid() {
    let resolved = resource(&env("service.namespace=anomalyco,broken")).expect(NOTE);
    assert!(!resolved.attributes.contains_key("service.namespace"));
    assert!(resolved.attributes.contains_key("opencode.client"));
}

#[test]
fn keeps_builtin_attributes_when_env_values_conflict() {
    let resolved = resource(&ObservabilityEnv {
        otel_resource_attributes: Some(
            "opencode.client=web,service.instance.id=override,service.namespace=anomalyco"
                .to_string(),
        ),
        opencode_client: Some("cli".to_string()),
    })
    .expect(NOTE);

    assert_eq!(
        resolved
            .attributes
            .get("opencode.client")
            .map(String::as_str),
        Some("cli")
    );
    assert_eq!(
        resolved
            .attributes
            .get("service.namespace")
            .map(String::as_str),
        Some("anomalyco")
    );
    assert_ne!(
        resolved
            .attributes
            .get("service.instance.id")
            .map(String::as_str),
        Some("override")
    );
    let run = resolved.attributes.get("opencode.run").expect(NOTE);
    assert_eq!(run.len(), 8);
    assert!(run.chars().all(|c| c.is_ascii_hexdigit()));
}
