//! Google Vertex provider plugin.
//!
//! Ports the observable behaviour of
//! `packages/core/src/plugin/provider/google-vertex.ts`: project and location
//! resolve from environment variables using the legacy precedence, a configured
//! project/location wins over the environment, `global` selects the global
//! endpoint, and model ids are trimmed before selecting a language model.

use std::collections::BTreeMap;

use crate::CoreResult;

/// The Google Vertex provider plugin.
#[derive(Debug, Default)]
pub struct GoogleVertexPlugin;

impl GoogleVertexPlugin {
    /// Resolve the project id: `GOOGLE_VERTEX_PROJECT`, then the legacy names.
    pub fn resolve_project(env: &BTreeMap<String, String>) -> CoreResult<Option<String>> {
        Ok(first(
            env,
            &[
                "GOOGLE_VERTEX_PROJECT",
                "GOOGLE_CLOUD_PROJECT",
                "GCP_PROJECT",
                "GCLOUD_PROJECT",
            ],
        ))
    }

    /// Resolve the location: `GOOGLE_VERTEX_LOCATION`, then legacy names, else
    /// `us-central1`.
    pub fn resolve_location(env: &BTreeMap<String, String>) -> CoreResult<String> {
        Ok(first(
            env,
            &[
                "GOOGLE_VERTEX_LOCATION",
                "GOOGLE_CLOUD_LOCATION",
                "VERTEX_LOCATION",
            ],
        )
        .unwrap_or_else(|| "us-central1".to_string()))
    }

    /// Rewrite an endpoint template using the resolved project/location.
    pub fn apply_url_template(template: &str, project: &str, location: &str) -> CoreResult<String> {
        let endpoint = if location == "global" {
            "aiplatform.googleapis.com".to_string()
        } else {
            format!("{location}-aiplatform.googleapis.com")
        };
        Ok(template
            .replace("${GOOGLE_VERTEX_PROJECT}", project)
            .replace("${GOOGLE_VERTEX_LOCATION}", location)
            .replace("${GOOGLE_VERTEX_ENDPOINT}", &endpoint))
    }

    /// Normalize a model id (trim surrounding whitespace).
    pub fn select_model_id(id: &str) -> CoreResult<String> {
        Ok(id.trim().to_string())
    }
}

/// The Google Vertex Anthropic provider plugin.
///
/// Ports the observable behaviour of
/// `packages/core/src/plugin/provider/google-vertex.ts` (Anthropic branch):
/// project resolves from `GOOGLE_CLOUD_PROJECT`/`GCP_PROJECT`/`GCLOUD_PROJECT`,
/// location from `GOOGLE_CLOUD_LOCATION`/`VERTEX_LOCATION` (ignoring
/// `GOOGLE_VERTEX_LOCATION`) defaulting to `global`, and the publisher base URL
/// uses regional `*.rep.googleapis.com` hosts for multi-region locations.
#[derive(Debug, Default)]
pub struct GoogleVertexAnthropicPlugin;

impl GoogleVertexAnthropicPlugin {
    /// Resolve the project id for the Anthropic publisher.
    pub fn resolve_project(env: &BTreeMap<String, String>) -> CoreResult<Option<String>> {
        Ok(first(
            env,
            &["GOOGLE_CLOUD_PROJECT", "GCP_PROJECT", "GCLOUD_PROJECT"],
        ))
    }

    /// Resolve the location for the Anthropic publisher, defaulting to `global`.
    pub fn resolve_location(env: &BTreeMap<String, String>) -> CoreResult<String> {
        Ok(first(env, &["GOOGLE_CLOUD_LOCATION", "VERTEX_LOCATION"])
            .unwrap_or_else(|| "global".to_string()))
    }

    /// Build the Anthropic publisher base URL, respecting a configured base URL.
    pub fn base_url(project: &str, location: &str, configured: Option<&str>) -> CoreResult<String> {
        if let Some(configured) = configured.filter(|value| !value.is_empty()) {
            return Ok(configured.to_string());
        }
        if location == "eu" || location == "us" {
            return Ok(format!(
                "https://aiplatform.{location}.rep.googleapis.com/v1/projects/{project}/locations/{location}/publishers/anthropic/models"
            ));
        }
        let host = if location == "global" {
            "aiplatform.googleapis.com".to_string()
        } else {
            format!("{location}-aiplatform.googleapis.com")
        };
        Ok(format!(
            "https://{host}/v1/projects/{project}/locations/{location}/publishers/anthropic/models"
        ))
    }
}

fn first(env: &BTreeMap<String, String>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(value) = env.get(*key).filter(|value| !value.is_empty()) {
            return Some(value.clone());
        }
    }
    None
}
