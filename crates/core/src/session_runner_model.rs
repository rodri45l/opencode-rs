//! Session runner model resolution (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/session/runner/model.ts`: catalog AI-SDK models map to
//! native route ids, `apiKey` credentials never reach the provider JSON, merged
//! API settings drive bearer auth, selected Session variants overlay route
//! headers/body, an unavailable explicit variant and an unsupported API fail with
//! their exact messages, stored credentials win over configured auth, OAuth
//! metadata is not projected, and `supported` reports whether a native route
//! exists. `LLMClient.prepare`/`route.auth.apply` are replaced by reading the
//! resolved route auth token and http body directly.

use std::fmt;

use serde_json::{Map, Value};

/// A model-resolution failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// The catalog API has no native route.
    UnsupportedApi {
        /// Provider id.
        provider_id: String,
        /// Model id.
        model_id: String,
        /// API label.
        api: String,
    },
    /// The selected Session variant does not exist.
    VariantUnavailable {
        /// Provider id.
        provider_id: String,
        /// Model id.
        model_id: String,
        /// Variant id.
        variant: String,
    },
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedApi {
                provider_id,
                model_id,
                api,
            } => write!(f, "{}", unsupported_api_message(provider_id, model_id, api)),
            Self::VariantUnavailable {
                provider_id,
                model_id,
                variant,
            } => write!(
                f,
                "{}",
                variant_unavailable_message(provider_id, model_id, variant)
            ),
        }
    }
}

impl std::error::Error for ResolveError {}

/// A catalog model resolved to a native provider route.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedModel {
    /// Model id.
    pub id: String,
    /// Provider id.
    pub provider: String,
    /// Native route id.
    pub route_id: String,
    /// Base URL.
    pub base_url: String,
    /// Request headers.
    pub headers: Value,
    /// Request body.
    pub body: Value,
    /// Model limits.
    pub limits: Value,
    /// Resolved auth token.
    pub auth_token: Option<String>,
}

impl ResolvedModel {
    /// The bearer authorization header value, if any.
    pub fn bearer(&self) -> Option<String> {
        self.auth_token
            .as_ref()
            .map(|token| format!("Bearer {token}"))
    }
}

/// The exact unsupported-API message.
pub fn unsupported_api_message(provider_id: &str, model_id: &str, api: &str) -> String {
    format!("Unsupported API for {provider_id}/{model_id}: {api}")
}

/// The exact unavailable-variant message.
pub fn variant_unavailable_message(provider_id: &str, model_id: &str, variant: &str) -> String {
    format!("Variant unavailable for {provider_id}/{model_id}: {variant}")
}

fn route_for(package: &str) -> Option<&'static str> {
    match package {
        "@ai-sdk/openai" => Some("openai-responses"),
        "@ai-sdk/openai-compatible" => Some("openai-chat"),
        "@ai-sdk/anthropic" => Some("anthropic-messages"),
        _ => None,
    }
}

fn api_label(api: &Value) -> String {
    let kind = api.get("type").and_then(Value::as_str).unwrap_or("");
    match kind {
        "aisdk" => {
            let package = api.get("package").and_then(Value::as_str).unwrap_or("");
            format!("aisdk:{package}")
        }
        other => other.to_string(),
    }
}

/// Whether a catalog model has a supported native route.
pub fn supported(info: &Value) -> bool {
    let api = info
        .get("api")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let kind = api.get("type").and_then(Value::as_str).unwrap_or("");
    let package = api.get("package").and_then(Value::as_str).unwrap_or("");
    kind == "aisdk" && route_for(package).is_some()
}

/// Resolve a catalog model to a native route.
pub fn from_catalog_model(
    info: &Value,
    credential: Option<&Value>,
) -> Result<ResolvedModel, ResolveError> {
    let provider_id = info
        .get("providerID")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let model_id = info
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let api = info
        .get("api")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let kind = api.get("type").and_then(Value::as_str).unwrap_or("");
    let package = api.get("package").and_then(Value::as_str).unwrap_or("");
    let route_id = if kind == "aisdk" {
        route_for(package)
    } else {
        None
    }
    .ok_or_else(|| ResolveError::UnsupportedApi {
        provider_id: provider_id.clone(),
        model_id: model_id.clone(),
        api: api_label(&api),
    })?;

    let request = info
        .get("request")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let headers = request
        .get("headers")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let mut body = request
        .get("body")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    body.remove("apiKey");

    let mut auth_token: Option<String> = None;
    if let Some(credential) = credential {
        match credential.get("type").and_then(Value::as_str) {
            Some("key") => {
                auth_token = credential
                    .get("key")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                if let Some(metadata) = credential.get("metadata").and_then(Value::as_object) {
                    for (key, value) in metadata {
                        body.insert(key.clone(), value.clone());
                    }
                }
            }
            Some("oauth") => {
                auth_token = credential
                    .get("access")
                    .and_then(Value::as_str)
                    .map(str::to_string);
            }
            _ => {}
        }
    }
    if auth_token.is_none() {
        auth_token = api
            .get("settings")
            .and_then(|settings| settings.get("apiKey"))
            .and_then(Value::as_str)
            .map(str::to_string);
    }

    let id = api
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or(&model_id)
        .to_string();
    let base_url = api
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let limits = info.get("limit").cloned().unwrap_or(Value::Null);

    Ok(ResolvedModel {
        id,
        provider: provider_id,
        route_id: route_id.to_string(),
        base_url,
        headers,
        body: Value::Object(body),
        limits,
        auth_token,
    })
}

/// Resolve a Session's selected model, overlaying its variant.
pub fn resolve(session: &Value, catalog: &Value) -> Result<ResolvedModel, ResolveError> {
    let mut resolved = from_catalog_model(catalog, None)?;
    let provider_id = catalog
        .get("providerID")
        .and_then(Value::as_str)
        .unwrap_or("");
    let model_id = catalog.get("id").and_then(Value::as_str).unwrap_or("");
    let variant = session
        .get("model")
        .and_then(|model| model.get("variant"))
        .and_then(Value::as_str);
    let selected_variant = match variant.filter(|variant| *variant != "default") {
        Some(variant) => variant,
        None => return Ok(resolved),
    };
    let selected = catalog
        .get("variants")
        .and_then(Value::as_array)
        .and_then(|variants| {
            variants.iter().find(|candidate| {
                candidate.get("id").and_then(Value::as_str) == Some(selected_variant)
            })
        });
    let selected = match selected {
        Some(selected) => selected,
        None => {
            return Err(ResolveError::VariantUnavailable {
                provider_id: provider_id.to_string(),
                model_id: model_id.to_string(),
                variant: selected_variant.to_string(),
            })
        }
    };

    if let Some(headers) = selected.get("headers").and_then(Value::as_object) {
        let base = resolved
            .headers
            .as_object_mut()
            .expect("resolved headers object");
        for (key, value) in headers {
            base.insert(key.clone(), value.clone());
        }
    }
    if let Some(body) = selected.get("body").and_then(Value::as_object) {
        let base = resolved.body.as_object_mut().expect("resolved body object");
        for (key, value) in body {
            base.insert(key.clone(), value.clone());
        }
    }
    Ok(resolved)
}
