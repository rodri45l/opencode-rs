//! Port of packages/core/test/plugin/provider-google-vertex-anthropic.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the Anthropic Google Vertex plugin resolves project from
//! `GOOGLE_CLOUD_PROJECT`/`GCP_PROJECT`/`GCLOUD_PROJECT` and location from
//! `GOOGLE_CLOUD_LOCATION`/`VERTEX_LOCATION` (ignoring `GOOGLE_VERTEX_LOCATION`),
//! defaulting to `global`; the publisher base URL uses regional
//! `*.rep.googleapis.com` hosts for multi-region locations and honours a
//! configured base URL; model ids are trimmed. Re-derived: the `AISDK`/`Catalog`
//! service wiring and SDK factory are dropped.

use std::collections::BTreeMap;

use opencode_core::provider_google_vertex::{GoogleVertexAnthropicPlugin, GoogleVertexPlugin};

fn env(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

#[test]
fn resolves_legacy_project_and_location_env() {
    let values = env(&[
        ("GOOGLE_CLOUD_PROJECT", "cloud-project"),
        ("GCP_PROJECT", "gcp-project"),
        ("GCLOUD_PROJECT", "gcloud-project"),
        ("GOOGLE_CLOUD_LOCATION", "cloud-location"),
        ("VERTEX_LOCATION", "vertex-location"),
        ("GOOGLE_VERTEX_LOCATION", "google-vertex-location"),
    ]);

    assert_eq!(
        GoogleVertexAnthropicPlugin::resolve_project(&values)
            .unwrap()
            .as_deref(),
        Some("cloud-project")
    );
    assert_eq!(
        GoogleVertexAnthropicPlugin::resolve_location(&values).unwrap(),
        "cloud-location"
    );
}

#[test]
fn defaults_location_to_global_and_ignores_vertex_location() {
    let values = env(&[
        ("GCP_PROJECT", "gcp-project"),
        ("GCLOUD_PROJECT", "gcloud-project"),
        ("GOOGLE_VERTEX_LOCATION", "ignored-location"),
    ]);

    assert_eq!(
        GoogleVertexAnthropicPlugin::resolve_project(&values)
            .unwrap()
            .as_deref(),
        Some("gcp-project")
    );
    assert_eq!(
        GoogleVertexAnthropicPlugin::resolve_location(&values).unwrap(),
        "global"
    );
    assert_eq!(
        GoogleVertexAnthropicPlugin::base_url("gcp-project", "global", None).unwrap(),
        "https://aiplatform.googleapis.com/v1/projects/gcp-project/locations/global/publishers/anthropic/models"
    );
}

#[test]
fn prefers_google_cloud_location_over_vertex_location() {
    let values = env(&[
        ("GOOGLE_CLOUD_PROJECT", "project"),
        ("GOOGLE_CLOUD_LOCATION", "cloud-location"),
        ("VERTEX_LOCATION", "vertex-location"),
    ]);

    assert_eq!(
        GoogleVertexAnthropicPlugin::resolve_location(&values).unwrap(),
        "cloud-location"
    );
    assert_eq!(
        GoogleVertexAnthropicPlugin::base_url("project", "cloud-location", None).unwrap(),
        "https://cloud-location-aiplatform.googleapis.com/v1/projects/project/locations/cloud-location/publishers/anthropic/models"
    );
}

#[test]
fn builds_multi_region_endpoints() {
    assert_eq!(
        GoogleVertexAnthropicPlugin::base_url("project", "eu", None).unwrap(),
        "https://aiplatform.eu.rep.googleapis.com/v1/projects/project/locations/eu/publishers/anthropic/models"
    );
    assert_eq!(
        GoogleVertexAnthropicPlugin::base_url("project", "us", None).unwrap(),
        "https://aiplatform.us.rep.googleapis.com/v1/projects/project/locations/us/publishers/anthropic/models"
    );
}

#[test]
fn keeps_a_configured_base_url() {
    assert_eq!(
        GoogleVertexAnthropicPlugin::base_url("project", "eu", Some("https://proxy.example/v1"))
            .unwrap(),
        "https://proxy.example/v1"
    );
}

#[test]
fn trims_model_ids_before_selecting_language_models() {
    assert_eq!(
        GoogleVertexPlugin::select_model_id(" claude-sonnet-4-5 ").unwrap(),
        "claude-sonnet-4-5"
    );
}
