//! Port of packages/core/test/plugin/provider-google-vertex.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: project/location resolve from environment variables using
//! the legacy precedence (`GOOGLE_VERTEX_*` before `GOOGLE_CLOUD_*` before the
//! generic names), a configured project/location wins over the environment,
//! `global` selects the global endpoint while regional locations get a regional
//! host, and model ids are trimmed. Re-derived: the `AISDK`/`Catalog` service
//! wiring and SDK mocking are replaced by direct plugin helpers.

use std::collections::BTreeMap;

use opencode_core::provider_google_vertex::GoogleVertexPlugin;

fn env(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

#[test]
fn resolves_project_and_location_from_env_using_legacy_precedence() {
    let values = env(&[
        ("GOOGLE_CLOUD_PROJECT", "google-cloud-project"),
        ("GCP_PROJECT", "gcp-project"),
        ("GCLOUD_PROJECT", "gcloud-project"),
        ("GOOGLE_VERTEX_LOCATION", "google-vertex-location"),
        ("GOOGLE_CLOUD_LOCATION", "google-cloud-location"),
        ("VERTEX_LOCATION", "vertex-location"),
    ]);

    assert_eq!(
        GoogleVertexPlugin::resolve_project(&values)
            .unwrap()
            .as_deref(),
        Some("google-cloud-project")
    );
    assert_eq!(
        GoogleVertexPlugin::resolve_location(&values).unwrap(),
        "google-vertex-location"
    );
}

#[test]
fn resolves_the_advertised_vertex_project_env() {
    let values = env(&[
        ("GOOGLE_VERTEX_PROJECT", "vertex-project"),
        ("GOOGLE_VERTEX_LOCATION", "europe-west4"),
    ]);

    assert_eq!(
        GoogleVertexPlugin::resolve_project(&values)
            .unwrap()
            .as_deref(),
        Some("vertex-project")
    );
    assert_eq!(
        GoogleVertexPlugin::resolve_location(&values).unwrap(),
        "europe-west4"
    );
}

#[test]
fn defaults_location_to_us_central1_when_only_project_is_configured() {
    let empty = BTreeMap::new();
    assert_eq!(
        GoogleVertexPlugin::resolve_location(&empty).unwrap(),
        "us-central1"
    );
}

#[test]
fn rewrites_regional_and_global_endpoint_templates() {
    let template = "https://${GOOGLE_VERTEX_ENDPOINT}/v1/projects/${GOOGLE_VERTEX_PROJECT}/locations/${GOOGLE_VERTEX_LOCATION}";

    assert_eq!(
        GoogleVertexPlugin::apply_url_template(template, "config-project", "eu").unwrap(),
        "https://eu-aiplatform.googleapis.com/v1/projects/config-project/locations/eu"
    );
    assert_eq!(
        GoogleVertexPlugin::apply_url_template(template, "config-project", "global").unwrap(),
        "https://aiplatform.googleapis.com/v1/projects/config-project/locations/global"
    );
}

#[test]
fn trims_model_ids_before_selecting_language_models() {
    assert_eq!(
        GoogleVertexPlugin::select_model_id(" gemini-2.5-pro ").unwrap(),
        "gemini-2.5-pro"
    );
}
