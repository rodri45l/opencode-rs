//! Port of packages/app/src/hooks/provider-catalog.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeMap;

use opencode_app::provider_catalog::{
    resolve_default_model, resolve_default_model_legacy, select_provider_catalog, Catalog, ModelRef,
};

fn empty_catalog() -> Catalog {
    Catalog {
        all: BTreeMap::new(),
        connected: Vec::new(),
        default: BTreeMap::new(),
    }
}

fn catalog(id: &str) -> Catalog {
    let mut all = BTreeMap::new();
    all.insert(id.to_string(), id.to_string());
    let mut default = BTreeMap::new();
    default.insert(id.to_string(), format!("{id}-model"));
    Catalog {
        all,
        connected: vec![id.to_string()],
        default,
    }
}

#[test]
fn selects_the_ready_catalog_for_an_explicit_directory() {
    let directory = catalog("directory");
    assert_eq!(
        select_provider_catalog(true, Some("/repo"), true, Some(directory.clone()), None),
        directory
    );
}

#[test]
fn returns_an_empty_catalog_while_an_explicit_directory_is_unresolved() {
    assert_eq!(
        select_provider_catalog(true, None, false, None, None),
        empty_catalog()
    );
    assert_eq!(
        select_provider_catalog(true, Some("/repo"), false, Some(catalog("directory")), None),
        empty_catalog()
    );
}

#[test]
fn uses_the_route_catalog_when_it_is_ready() {
    let directory = catalog("directory");
    assert_eq!(
        select_provider_catalog(
            false,
            Some("/repo"),
            true,
            Some(directory.clone()),
            Some(catalog("global"))
        ),
        directory
    );
}

#[test]
fn falls_back_to_the_global_catalog_for_route_consumers() {
    let global = catalog("global");
    assert_eq!(
        select_provider_catalog(false, None, false, None, Some(global.clone())),
        global
    );
    assert_eq!(
        select_provider_catalog(
            false,
            Some("/repo"),
            false,
            Some(catalog("directory")),
            Some(global.clone())
        ),
        global
    );
}

#[test]
fn uses_the_current_server_default_model() {
    assert_eq!(
        resolve_default_model(
            Some(ModelRef {
                provider_id: "openai".into(),
                model_id: "gpt-5".into()
            }),
            Some("anthropic/claude")
        ),
        Some(ModelRef {
            provider_id: "openai".into(),
            model_id: "gpt-5".into()
        })
    );
}

#[test]
fn does_not_use_legacy_config_when_the_current_server_has_no_default() {
    assert_eq!(resolve_default_model(None, Some("anthropic/claude")), None);
}

#[test]
fn uses_config_for_legacy_servers() {
    // Distinguish "current absent" (None) from "legacy" (Some(None)).
    assert_eq!(
        resolve_default_model_legacy(Some(None), Some("anthropic/claude")),
        Some(ModelRef {
            provider_id: "anthropic".into(),
            model_id: "claude".into()
        })
    );
}
