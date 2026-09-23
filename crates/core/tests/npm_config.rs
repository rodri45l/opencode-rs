//! Port of packages/core/test/npm-config.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a project `.npmrc` is read into a normalized map (scoped
//! registry keys preserved, booleans camelCased, list options flattened) and the
//! registry resolves without a trailing slash.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use opencode_core::npm_config::NpmConfig;
use serde_json::json;

const NOTE: &str = "porting: npm config not implemented";

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch(contents: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-rs-npmconfig-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(".npmrc"), contents).unwrap();
    dir
}

#[test]
#[ignore = "porting: npm config not implemented"]
fn reads_registry_from_project_npmrc() {
    let dir = scratch("registry=https://registry.example.test/\n");
    let config = NpmConfig::load(&dir.to_string_lossy()).expect(NOTE);
    assert_eq!(config["registry"], json!("https://registry.example.test/"));
}

#[test]
#[ignore = "porting: npm config not implemented"]
fn reads_scoped_registries_from_project_npmrc() {
    let dir = scratch("@acme:registry=https://npm.acme.test/\n");
    let config = NpmConfig::load(&dir.to_string_lossy()).expect(NOTE);
    assert_eq!(config["@acme:registry"], json!("https://npm.acme.test/"));
}

#[test]
#[ignore = "porting: npm config not implemented"]
fn flattens_boolean_and_list_options() {
    let dir = scratch("ignore-scripts=true\nomit[]=dev\nomit[]=optional\n");
    let config = NpmConfig::load(&dir.to_string_lossy()).expect(NOTE);
    assert_eq!(config["ignoreScripts"], json!(true));
    assert_eq!(config["omit"], json!(["dev", "optional"]));
}

#[test]
#[ignore = "porting: npm config not implemented"]
fn normalizes_configured_registry_without_trailing_slash() {
    let dir = scratch("registry=https://registry.example.test/\n");
    assert_eq!(
        NpmConfig::registry(&dir.to_string_lossy()).expect(NOTE),
        "https://registry.example.test"
    );
}

#[test]
#[ignore = "porting: npm config not implemented"]
fn leaves_configured_registry_without_trailing_slash_unchanged() {
    let dir = scratch("registry=https://registry.example.test\n");
    assert_eq!(
        NpmConfig::registry(&dir.to_string_lossy()).expect(NOTE),
        "https://registry.example.test"
    );
}
