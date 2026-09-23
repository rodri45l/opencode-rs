//! SAP AI Core provider plugin option resolution (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/plugin/provider/sap-ai-core.ts`: a configured `serviceKey`
//! is exported to `AICORE_SERVICE_KEY` unless one is already present, the SDK
//! options keep only the deployment id and resource group, and those are omitted
//! entirely when no service key is available. The SDK factory and callable
//! language selection are dropped.

use std::collections::BTreeMap;

use crate::{CoreError, CoreResult};

/// Resolved SAP AI Core SDK configuration.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedSapAICore {
    /// The value written to `AICORE_SERVICE_KEY`, if any.
    pub service_key: Option<String>,
    /// Deployment id forwarded to the SDK, if a service key is available.
    pub deployment_id: Option<String>,
    /// Resource group forwarded to the SDK, if a service key is available.
    pub resource_group: Option<String>,
}

/// The SAP AI Core provider plugin.
#[derive(Debug, Default)]
pub struct SapAICorePlugin;

impl SapAICorePlugin {
    /// Resolve the SDK configuration from the environment and a configured key.
    pub fn resolve(
        _env: &BTreeMap<String, String>,
        _configured_service_key: Option<&str>,
    ) -> CoreResult<ResolvedSapAICore> {
        Err(CoreError::NotImplemented(
            "provider_sap_ai_core::SapAICorePlugin::resolve",
        ))
    }
}
