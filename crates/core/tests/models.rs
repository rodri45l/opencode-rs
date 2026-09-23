//! Port of packages/core/test/models.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `get()` serves the provider catalog from the on-disk cache,
//! an absent cache yields an empty catalog while fetching is disabled, and the
//! decoded catalog is cached across calls until an explicit refresh. Dropped
//! (re-derived): the HTTP fetch/refresh cases and the `Flag` env toggling, which
//! depend on the platform `HttpClient` and a bundler fixture.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use opencode_core::models_dev::ModelsDev;
use serde_json::{json, Value};

const NOTE: &str = "porting: models-dev not implemented";

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-rs-models-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn fixture() -> Value {
    json!({
        "acme": {
            "id": "acme",
            "name": "Acme",
            "env": ["ACME_API_KEY"],
            "models": {
                "acme-1": {
                    "id": "acme-1",
                    "name": "Acme One",
                    "release_date": "2026-01-01",
                    "attachment": false,
                    "reasoning": false,
                    "temperature": true,
                    "tool_call": true,
                    "limit": { "context": 128000, "output": 8192 },
                },
            },
        },
    })
}

fn fixture_two() -> Value {
    json!({
        "beta": {
            "id": "beta",
            "name": "Beta",
            "env": ["BETA_API_KEY"],
            "models": {
                "beta-1": {
                    "id": "beta-1",
                    "name": "Beta One",
                    "release_date": "2026-02-01",
                    "attachment": false,
                    "reasoning": true,
                    "temperature": false,
                    "tool_call": false,
                    "limit": { "context": 64000, "output": 4096 },
                },
            },
        },
    })
}

#[test]
fn get_returns_providers_from_disk_when_cache_file_exists() {
    let dir = scratch();
    let cache_file = dir.join("models.json");
    std::fs::write(&cache_file, serde_json::to_vec(&fixture()).unwrap()).unwrap();

    let models = ModelsDev::with_cache_path(&cache_file);
    assert_eq!(models.get().expect(NOTE), fixture());
}

#[test]
fn get_returns_empty_catalog_when_disk_is_empty() {
    let dir = scratch();
    let models = ModelsDev::with_cache_path(dir.join("models.json"));

    assert_eq!(models.get().expect(NOTE), json!({}));
}

#[test]
fn get_caches_across_calls_until_invalidated() {
    let dir = scratch();
    let cache_file = dir.join("models.json");
    std::fs::write(&cache_file, serde_json::to_vec(&fixture()).unwrap()).unwrap();

    let models = ModelsDev::with_cache_path(&cache_file);
    let first = models.get().expect(NOTE);

    // Mutate the disk between calls: the decoded cache must mask the change.
    std::fs::write(&cache_file, serde_json::to_vec(&fixture_two()).unwrap()).unwrap();
    let second = models.get().expect(NOTE);

    assert_eq!(first, fixture());
    assert_eq!(second, fixture());
}
