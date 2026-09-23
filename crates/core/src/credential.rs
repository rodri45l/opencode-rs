//! Stored integration credentials.
//!
//! Ports the observable behaviour of `packages/core/src/credential.ts`:
//! credentials are scoped to an integration, updating a credential keeps its
//! identity, and creating a replacement drops the previous active credential.

use std::fmt;

use crate::{CoreError, CoreResult};

/// Identifier for an integration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IntegrationId(String);

impl IntegrationId {
    /// Construct an integration id.
    pub fn make(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IntegrationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Identifier for a credential.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CredentialId(String);

impl CredentialId {
    /// Construct a credential id.
    pub fn make(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CredentialId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The secret material behind a credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialValue {
    /// A plain API key.
    Key {
        /// The secret.
        key: String,
    },
}

impl CredentialValue {
    /// Build an API-key credential.
    pub fn key(key: impl Into<String>) -> Self {
        Self::Key { key: key.into() }
    }
}

/// A stored credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialInfo {
    /// Credential id.
    pub id: CredentialId,
    /// Owning integration.
    pub integration_id: IntegrationId,
    /// Optional label.
    pub label: Option<String>,
    /// Secret material.
    pub value: CredentialValue,
}

/// Input to [`CredentialStore::create`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialCreate {
    /// Owning integration.
    pub integration_id: IntegrationId,
    /// Optional label.
    pub label: Option<String>,
    /// Secret material.
    pub value: CredentialValue,
}

/// Fields changed by [`CredentialStore::update`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CredentialPatch {
    /// New label, when set.
    pub label: Option<String>,
    /// New secret, when set.
    pub value: Option<CredentialValue>,
}

/// Store for integration credentials.
#[derive(Debug, Default)]
pub struct CredentialStore;

impl CredentialStore {
    /// Create an empty store.
    pub fn new() -> CoreResult<Self> {
        Err(CoreError::NotImplemented(
            "credential::CredentialStore::new",
        ))
    }

    /// Create (and activate) a credential.
    pub fn create(&self, _input: CredentialCreate) -> CoreResult<CredentialInfo> {
        Err(CoreError::NotImplemented(
            "credential::CredentialStore::create",
        ))
    }

    /// List the active credentials for an integration.
    pub fn list(&self, _integration: &IntegrationId) -> CoreResult<Vec<CredentialInfo>> {
        Err(CoreError::NotImplemented(
            "credential::CredentialStore::list",
        ))
    }

    /// Update a credential in place.
    pub fn update(
        &self,
        _id: &CredentialId,
        _patch: CredentialPatch,
    ) -> CoreResult<CredentialInfo> {
        Err(CoreError::NotImplemented(
            "credential::CredentialStore::update",
        ))
    }

    /// Remove a credential.
    pub fn remove(&self, _id: &CredentialId) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "credential::CredentialStore::remove",
        ))
    }
}
