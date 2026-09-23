//! Port of packages/core/test/plugin/provider-kilo.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Kilo plugin is registered, applies the exact legacy
//! referer header casing only to the `kilo` provider id, merges with existing
//! headers, and is guarded by the provider id rather than endpoint matching.
//! Re-derived as a pure header mapping; the `Catalog`/`PluginHost` wiring is dropped.

use std::collections::BTreeMap;

use opencode_core::provider_plugins::{ProviderPlugins, ProviderRequest};
use serde_json::json;

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
fn is_registered_so_legacy_referer_headers_can_be_applied() {
    assert!(ProviderPlugins::ids().contains(&"kilo"));
}

#[test]
fn applies_legacy_referer_headers_only_to_kilo() {
    let mut kilo = request(&[("Existing", "value")]);
    ProviderPlugins::apply_kilo("kilo", &mut kilo).unwrap();
    assert_eq!(
        kilo.headers.get("Existing").map(String::as_str),
        Some("value")
    );
    assert_eq!(
        kilo.headers.get("HTTP-Referer").map(String::as_str),
        Some("https://opencode.ai/")
    );
    assert_eq!(
        kilo.headers.get("X-Title").map(String::as_str),
        Some("opencode")
    );

    let mut openrouter = request(&[]);
    ProviderPlugins::apply_kilo("openrouter", &mut openrouter).unwrap();
    assert!(openrouter.headers.is_empty());
}

#[test]
fn uses_the_exact_legacy_kilo_header_casing_and_set() {
    let mut kilo = request(&[]);
    ProviderPlugins::apply_kilo("kilo", &mut kilo).unwrap();

    let keys: Vec<&String> = kilo.headers.keys().collect();
    assert_eq!(keys, vec!["HTTP-Referer", "X-Title"]);
    assert!(!kilo.headers.contains_key("http-referer"));
    assert!(!kilo.headers.contains_key("x-title"));
    assert!(!kilo.headers.contains_key("X-Source"));
}

#[test]
fn uses_the_legacy_provider_id_guard_instead_of_endpoint_matching() {
    let mut custom = request(&[]);
    ProviderPlugins::apply_kilo("custom-kilo", &mut custom).unwrap();
    assert!(custom.headers.is_empty());
}
