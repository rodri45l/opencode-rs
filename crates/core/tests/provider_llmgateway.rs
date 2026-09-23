//! Port of packages/core/test/plugin/provider-llmgateway.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the llmgateway plugin is registered, applies the legacy
//! referer headers (`HTTP-Referer`, `X-Title`, `X-Source`) plus configured
//! headers only to an enabled `llmgateway` provider, and leaves a disabled
//! provider untouched. Re-derived: the `Catalog`/`Integration` service wiring is
//! dropped.

use std::collections::BTreeMap;

use opencode_core::provider_header_plugins::ProviderHeaderPlugins;
use opencode_core::provider_plugins::ProviderRequest;
use serde_json::json;

const NOTE: &str = "porting: llmgateway provider plugin not implemented";

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
#[ignore = "porting: llmgateway provider plugin not implemented"]
fn is_registered_so_legacy_referer_headers_can_be_applied() {
    assert!(ProviderHeaderPlugins::registered_ids()
        .expect(NOTE)
        .contains(&"llmgateway"));
}

#[test]
#[ignore = "porting: llmgateway provider plugin not implemented"]
fn applies_legacy_referer_headers_only_to_an_enabled_llmgateway() {
    let mut gateway = request(&[("Existing", "value")]);
    ProviderHeaderPlugins::apply_llmgateway("llmgateway", true, &mut gateway).expect(NOTE);
    assert_eq!(
        gateway.headers.get("Existing").map(String::as_str),
        Some("value")
    );
    assert_eq!(
        gateway.headers.get("HTTP-Referer").map(String::as_str),
        Some("https://opencode.ai/")
    );
    assert_eq!(
        gateway.headers.get("X-Title").map(String::as_str),
        Some("opencode")
    );
    assert_eq!(
        gateway.headers.get("X-Source").map(String::as_str),
        Some("opencode")
    );

    let mut openrouter = request(&[]);
    ProviderHeaderPlugins::apply_llmgateway("openrouter", true, &mut openrouter).expect(NOTE);
    assert!(openrouter.headers.is_empty());
}

#[test]
#[ignore = "porting: llmgateway provider plugin not implemented"]
fn does_not_apply_legacy_headers_to_a_disabled_llmgateway_provider() {
    let mut gateway = request(&[]);
    ProviderHeaderPlugins::apply_llmgateway("llmgateway", false, &mut gateway).expect(NOTE);

    assert!(gateway.headers.is_empty());
}
