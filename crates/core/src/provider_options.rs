//! Provider option lowering.
//!
//! Ports the observable behaviour of
//! `packages/core/src/v1/config/provider-options.ts`: each provider package maps
//! its configured provider and request options into SDK-shaped `url`, `headers`,
//! `body`, `settings`, and request body fields, falling back to raw lowering for
//! unknown packages.

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// A provider/request option lowerer for one package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lowerer {
    /// The provider package name.
    pub package: String,
}

/// Provider option lowering entry point.
#[derive(Debug, Default)]
pub struct ConfigProviderOptionsV1;

impl ConfigProviderOptionsV1 {
    /// Get the lowerer for a provider package.
    pub fn get(_package: &str) -> Lowerer {
        Lowerer {
            package: _package.to_string(),
        }
    }
}

impl Lowerer {
    /// Lower configured provider options.
    pub fn provider(&self, _options: &Value) -> CoreResult<Value> {
        Err(CoreError::NotImplemented(
            "provider_options::Lowerer::provider",
        ))
    }

    /// Lower configured request options.
    pub fn request(&self, _options: &Value) -> CoreResult<Value> {
        Err(CoreError::NotImplemented(
            "provider_options::Lowerer::request",
        ))
    }
}
