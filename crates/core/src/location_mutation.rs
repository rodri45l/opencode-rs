//! Location-scoped mutation path resolution.
//!
//! Ports the observable behaviour of `packages/core/src/location-mutation.ts`:
//! resolve relative, prospective, absolute and external targets into canonical
//! paths and resources, reject lexical escapes, and require external-directory
//! authorization for explicit external absolute targets.

use serde_json::Value;

use crate::path::AbsolutePath;
use crate::{CoreError, CoreResult};

/// An external directory that requires authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalDirectory {
    /// Stable existing directory.
    pub directory: AbsolutePath,
    /// Resource glob under that directory.
    pub resource: String,
}

/// A resolved mutation target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationTarget {
    /// Canonical target path.
    pub canonical: AbsolutePath,
    /// Resource name relative to the location (or the path for externals).
    pub resource: String,
    /// External-directory authorization, when required.
    pub external_directory: Option<ExternalDirectory>,
}

/// The input accepted by [`LocationMutation::resolve`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveInput {
    /// The requested path.
    pub path: String,
    /// Optional mutation kind.
    pub kind: Option<String>,
}

impl ResolveInput {
    /// Decode a resolve input, ignoring unknown fields.
    pub fn decode(_value: &Value) -> CoreResult<Self> {
        Err(CoreError::NotImplemented(
            "location_mutation::ResolveInput::decode",
        ))
    }
}

/// Location-scoped path resolution.
#[derive(Debug, Default)]
pub struct LocationMutation;

impl LocationMutation {
    /// Resolve `input` against `directory`.
    pub fn resolve(_directory: &AbsolutePath, _input: &ResolveInput) -> CoreResult<MutationTarget> {
        Err(CoreError::NotImplemented(
            "location_mutation::LocationMutation::resolve",
        ))
    }
}
