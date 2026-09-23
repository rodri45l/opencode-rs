//! Azure Cognitive Services provider plugin (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/plugin/provider/azure-cognitive-services.ts`: the SDK
//! package is `@ai-sdk/openai-compatible`, the resource name from
//! `AZURE_COGNITIVE_SERVICES_RESOURCE_NAME` maps to the
//! `https://{resource}.cognitiveservices.azure.com/openai` API URL, and the
//! language selector order matches the Azure plugin. Language selection is
//! resolved by [`crate::provider_sdk_plugins`].

use std::collections::BTreeMap;

use crate::{CoreError, CoreResult};

/// The Azure Cognitive Services provider plugin.
#[derive(Debug, Default)]
pub struct AzureCognitiveServicesPlugin;

impl AzureCognitiveServicesPlugin {
    /// The API URL derived from the resource env var, if present.
    pub fn base_url(_env: &BTreeMap<String, String>) -> CoreResult<Option<String>> {
        Err(CoreError::NotImplemented(
            "provider_azure_cognitive_services::AzureCognitiveServicesPlugin::base_url",
        ))
    }
}
