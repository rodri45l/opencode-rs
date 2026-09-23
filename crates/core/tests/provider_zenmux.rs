//! Port of packages/core/test/plugin/provider-zenmux.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Zenmux plugin is registered, applies its exact legacy
//! headers, merges with existing headers, lets configured headers override the
//! defaults, and is guarded to the exact `zenmux` provider id. Re-derived as a
//! pure header mapping; the `Catalog`/`PluginHost` wiring is dropped.

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
    assert!(ProviderPlugins::ids().contains(&"zenmux"));
}

#[test]
fn applies_the_exact_legacy_zenmux_headers() {
    let mut zenmux = request(&[]);
    ProviderPlugins::apply_zenmux("zenmux", &mut zenmux).unwrap();

    assert_eq!(
        zenmux.headers.get("HTTP-Referer").map(String::as_str),
        Some("https://opencode.ai/")
    );
    assert_eq!(
        zenmux.headers.get("X-Title").map(String::as_str),
        Some("opencode")
    );
    let mut keys: Vec<&String> = zenmux.headers.keys().collect();
    keys.sort();
    assert_eq!(keys, vec!["HTTP-Referer", "X-Title"]);
}

#[test]
fn merges_legacy_zenmux_headers_with_existing_headers() {
    let mut zenmux = request(&[("Existing", "value")]);
    ProviderPlugins::apply_zenmux("zenmux", &mut zenmux).unwrap();

    assert_eq!(
        zenmux.headers.get("Existing").map(String::as_str),
        Some("value")
    );
    assert_eq!(
        zenmux.headers.get("HTTP-Referer").map(String::as_str),
        Some("https://opencode.ai/")
    );
    assert_eq!(
        zenmux.headers.get("X-Title").map(String::as_str),
        Some("opencode")
    );
}

#[test]
fn lets_configured_zenmux_legacy_headers_override_defaults() {
    let mut zenmux = request(&[
        ("HTTP-Referer", "https://example.com/"),
        ("X-Title", "custom-title"),
    ]);
    ProviderPlugins::apply_zenmux("zenmux", &mut zenmux).unwrap();

    assert_eq!(
        zenmux.headers.get("HTTP-Referer").map(String::as_str),
        Some("https://example.com/")
    );
    assert_eq!(
        zenmux.headers.get("X-Title").map(String::as_str),
        Some("custom-title")
    );
}

#[test]
fn guards_legacy_zenmux_headers_to_the_exact_provider_id() {
    let mut openrouter = request(&[
        ("HTTP-Referer", "https://example.com/"),
        ("X-Title", "custom-title"),
    ]);
    ProviderPlugins::apply_zenmux("openrouter", &mut openrouter).unwrap();

    assert_eq!(
        openrouter.headers.get("HTTP-Referer").map(String::as_str),
        Some("https://example.com/")
    );
    assert_eq!(
        openrouter.headers.get("X-Title").map(String::as_str),
        Some("custom-title")
    );
}
