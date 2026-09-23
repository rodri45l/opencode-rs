//! Google Vertex provider plugin.
//!
//! Ports the observable behaviour of
//! `packages/core/src/plugin/provider/google-vertex.ts`: project and location
//! resolve from environment variables using the legacy precedence, a configured
//! project/location wins over the environment, `global` selects the global
//! endpoint, and model ids are trimmed before selecting a language model.

use std::collections::BTreeMap;

use crate::{CoreError, CoreResult};

/// The Google Vertex provider plugin.
#[derive(Debug, Default)]
pub struct GoogleVertexPlugin;

impl GoogleVertexPlugin {
    /// Resolve the project id: `GOOGLE_VERTEX_PROJECT`, then the legacy names.
    pub fn resolve_project(_env: &BTreeMap<String, String>) -> CoreResult<Option<String>> {
        Err(CoreError::NotImplemented(
            "provider_google_vertex::GoogleVertexPlugin::resolve_project",
        ))
    }

    /// Resolve the location: `GOOGLE_VERTEX_LOCATION`, then legacy names, else
    /// `us-central1`.
    pub fn resolve_location(_env: &BTreeMap<String, String>) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "provider_google_vertex::GoogleVertexPlugin::resolve_location",
        ))
    }

    /// Rewrite an endpoint template using the resolved project/location.
    pub fn apply_url_template(
        _template: &str,
        _project: &str,
        _location: &str,
    ) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "provider_google_vertex::GoogleVertexPlugin::apply_url_template",
        ))
    }

    /// Normalize a model id (trim surrounding whitespace).
    pub fn select_model_id(_id: &str) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "provider_google_vertex::GoogleVertexPlugin::select_model_id",
        ))
    }
}
