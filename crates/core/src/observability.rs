//! Observability resource attributes (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/observability/otlp.ts`
//! and `observability/logging.ts`: `OTEL_RESOURCE_ATTRIBUTES` is parsed as
//! comma-separated `key=value` pairs with percent-decoding, and if any entry is
//! invalid the whole user-supplied attribute set is dropped (built-ins remain).
//! Built-in attributes (`opencode.client`, `service.instance.id`,
//! `opencode.run`) win over conflicting environment values. The Effect logger
//! layer and file logger are dropped.

use std::collections::BTreeMap;

use crate::{CoreError, CoreResult};

/// Environment inputs for resource resolution.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObservabilityEnv {
    /// `OTEL_RESOURCE_ATTRIBUTES`.
    pub otel_resource_attributes: Option<String>,
    /// `OPENCODE_CLIENT`.
    pub opencode_client: Option<String>,
}

/// A resolved OTLP resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    /// The resolved attributes.
    pub attributes: BTreeMap<String, String>,
}

/// Parse and decode `OTEL_RESOURCE_ATTRIBUTES` into key/value pairs.
pub fn parse_otel_attributes(_raw: &str) -> CoreResult<BTreeMap<String, String>> {
    Err(CoreError::NotImplemented(
        "observability::parse_otel_attributes",
    ))
}

/// Resolve the OTLP resource attributes for the given environment.
pub fn resource(_env: &ObservabilityEnv) -> CoreResult<Resource> {
    Err(CoreError::NotImplemented("observability::resource"))
}
