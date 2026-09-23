//! Port of packages/opencode/test/cli/plugin-auth-picker.test.ts (upstream 18ef3cc).
//!
//! RED-first: `cli/cmd/providers`'s `resolvePluginProviders` is not implemented
//! in this crate. The reference selection/dedup/override behaviour is pinned
//! against a local typed stub.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Hook {
    auth_provider: Option<String>,
}

fn hook_with_auth(provider: &str) -> Hook {
    Hook {
        auth_provider: Some(provider.to_string()),
    }
}

fn hook_without_auth() -> Hook {
    Hook {
        auth_provider: None,
    }
}

#[derive(Debug, Clone, Default)]
struct ResolveOptions {
    hooks: Vec<Hook>,
    existing_providers: BTreeSet<String>,
    disabled: BTreeSet<String>,
    enabled: Option<BTreeSet<String>>,
    provider_names: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PluginProvider {
    id: String,
    name: String,
}

fn pp(id: &str, name: &str) -> PluginProvider {
    PluginProvider {
        id: id.to_string(),
        name: name.to_string(),
    }
}

fn resolve_plugin_providers(_options: &ResolveOptions) -> Vec<PluginProvider> {
    Vec::new()
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
#[ignore = "porting: cli plugin auth picker not implemented"]
fn returns_plugin_providers_not_in_models_dev() {
    let result = resolve_plugin_providers(&ResolveOptions {
        hooks: vec![hook_with_auth("portkey")],
        ..Default::default()
    });
    assert_eq!(result, vec![pp("portkey", "portkey")]);
}

#[test]
#[ignore = "porting: cli plugin auth picker not implemented"]
fn skips_providers_already_in_models_dev() {
    let result = resolve_plugin_providers(&ResolveOptions {
        hooks: vec![hook_with_auth("anthropic")],
        existing_providers: set(&["anthropic"]),
        ..Default::default()
    });
    assert_eq!(result, Vec::<PluginProvider>::new());
}

#[test]
#[ignore = "porting: cli plugin auth picker not implemented"]
fn deduplicates_across_plugins() {
    let result = resolve_plugin_providers(&ResolveOptions {
        hooks: vec![hook_with_auth("portkey"), hook_with_auth("portkey")],
        ..Default::default()
    });
    assert_eq!(result, vec![pp("portkey", "portkey")]);
}

#[test]
#[ignore = "porting: cli plugin auth picker not implemented"]
fn respects_disabled_providers() {
    let result = resolve_plugin_providers(&ResolveOptions {
        hooks: vec![hook_with_auth("portkey")],
        disabled: set(&["portkey"]),
        ..Default::default()
    });
    assert_eq!(result, Vec::<PluginProvider>::new());
}

#[test]
#[ignore = "porting: cli plugin auth picker not implemented"]
fn respects_enabled_providers_when_provider_is_absent() {
    let result = resolve_plugin_providers(&ResolveOptions {
        hooks: vec![hook_with_auth("portkey")],
        enabled: Some(set(&["anthropic"])),
        ..Default::default()
    });
    assert_eq!(result, Vec::<PluginProvider>::new());
}

#[test]
#[ignore = "porting: cli plugin auth picker not implemented"]
fn includes_provider_when_in_enabled_set() {
    let result = resolve_plugin_providers(&ResolveOptions {
        hooks: vec![hook_with_auth("portkey")],
        enabled: Some(set(&["portkey"])),
        ..Default::default()
    });
    assert_eq!(result, vec![pp("portkey", "portkey")]);
}

#[test]
#[ignore = "porting: cli plugin auth picker not implemented"]
fn resolves_name_from_provider_names() {
    let mut names = BTreeMap::new();
    names.insert("portkey".to_string(), "Portkey AI".to_string());
    let result = resolve_plugin_providers(&ResolveOptions {
        hooks: vec![hook_with_auth("portkey")],
        provider_names: names,
        ..Default::default()
    });
    assert_eq!(result, vec![pp("portkey", "Portkey AI")]);
}

#[test]
#[ignore = "porting: cli plugin auth picker not implemented"]
fn falls_back_to_id_when_no_name_configured() {
    let result = resolve_plugin_providers(&ResolveOptions {
        hooks: vec![hook_with_auth("portkey")],
        ..Default::default()
    });
    assert_eq!(result, vec![pp("portkey", "portkey")]);
}

#[test]
#[ignore = "porting: cli plugin auth picker not implemented"]
fn skips_hooks_without_auth() {
    let result = resolve_plugin_providers(&ResolveOptions {
        hooks: vec![
            hook_without_auth(),
            hook_with_auth("portkey"),
            hook_without_auth(),
        ],
        ..Default::default()
    });
    assert_eq!(result, vec![pp("portkey", "portkey")]);
}

#[test]
#[ignore = "porting: cli plugin auth picker not implemented"]
fn returns_empty_for_no_hooks() {
    let result = resolve_plugin_providers(&ResolveOptions::default());
    assert_eq!(result, Vec::<PluginProvider>::new());
}
