//! Port of packages/core/test/plugin/provider-nvidia.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the NVIDIA plugin is registered, applies its tracking
//! headers (`HTTP-Referer`, `X-Title`, `X-BILLING-INVOKE-ORIGIN`) only to the
//! exact `nvidia` provider id, merges with existing headers, and preserves an
//! explicitly configured billing origin. Re-derived: the `Catalog`/`PluginHost`
//! wiring is dropped.

use std::collections::BTreeMap;

use opencode_core::provider_header_plugins::ProviderHeaderPlugins;
use opencode_core::provider_plugins::ProviderRequest;
use serde_json::json;

const NOTE: &str = "porting: nvidia provider plugin not implemented";

fn request(headers: &[(&str, &str)]) -> ProviderRequest {
    ProviderRequest {
        headers: headers
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect::<BTreeMap<_, _>>(),
        body: json!({}),
    }
}

#[test]
#[ignore = "porting: nvidia provider plugin not implemented"]
fn is_registered_so_legacy_referer_headers_can_be_applied() {
    assert!(ProviderHeaderPlugins::registered_ids()
        .expect(NOTE)
        .contains(&"nvidia"));
}

#[test]
#[ignore = "porting: nvidia provider plugin not implemented"]
fn applies_nvidia_tracking_headers_only_to_nvidia() {
    let mut nvidia = request(&[("Existing", "value")]);
    ProviderHeaderPlugins::apply_nvidia("nvidia", &mut nvidia).expect(NOTE);
    assert_eq!(
        nvidia.headers.get("Existing").map(String::as_str),
        Some("value")
    );
    assert_eq!(
        nvidia.headers.get("HTTP-Referer").map(String::as_str),
        Some("https://opencode.ai/")
    );
    assert_eq!(
        nvidia.headers.get("X-Title").map(String::as_str),
        Some("opencode")
    );
    assert_eq!(
        nvidia
            .headers
            .get("X-BILLING-INVOKE-ORIGIN")
            .map(String::as_str),
        Some("OpenCode")
    );

    let mut openrouter = request(&[]);
    ProviderHeaderPlugins::apply_nvidia("openrouter", &mut openrouter).expect(NOTE);
    assert!(openrouter.headers.is_empty());
}

#[test]
#[ignore = "porting: nvidia provider plugin not implemented"]
fn adds_the_default_billing_origin_for_custom_nvidia_endpoints() {
    let mut nvidia = request(&[]);
    ProviderHeaderPlugins::apply_nvidia("nvidia", &mut nvidia).expect(NOTE);

    assert_eq!(
        nvidia.headers.get("HTTP-Referer").map(String::as_str),
        Some("https://opencode.ai/")
    );
    assert_eq!(
        nvidia.headers.get("X-Title").map(String::as_str),
        Some("opencode")
    );
    assert_eq!(
        nvidia
            .headers
            .get("X-BILLING-INVOKE-ORIGIN")
            .map(String::as_str),
        Some("OpenCode")
    );
}

#[test]
#[ignore = "porting: nvidia provider plugin not implemented"]
fn preserves_an_explicit_nvidia_billing_origin_header() {
    let mut nvidia = request(&[("X-BILLING-INVOKE-ORIGIN", "CustomOrigin")]);
    ProviderHeaderPlugins::apply_nvidia("nvidia", &mut nvidia).expect(NOTE);

    assert_eq!(
        nvidia
            .headers
            .get("X-BILLING-INVOKE-ORIGIN")
            .map(String::as_str),
        Some("CustomOrigin")
    );
}
