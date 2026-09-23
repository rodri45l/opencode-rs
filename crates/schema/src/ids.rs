//! Prefixed identifiers.
//!
//! The reference server identifies resources by opaque strings carrying a
//! stable prefix (`evt_`, `ses`, `msg`, ...). The prefixes are enforced at the
//! API layer, so the Rust types validate them on construction.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error returned when a string does not match the expected id shape.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("{kind} must start with {prefix:?}: {value:?}")]
pub struct IdError {
    pub kind: &'static str,
    pub prefix: &'static str,
    pub value: String,
}

macro_rules! prefixed_id {
    ($(#[$meta:meta])* $name:ident, $prefix:literal, $kind:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// The required prefix for this id kind.
            pub const PREFIX: &'static str = $prefix;

            /// Parse and validate an existing id.
            pub fn parse(value: impl Into<String>) -> Result<Self, IdError> {
                let value = value.into();
                if value.starts_with(Self::PREFIX) {
                    Ok(Self(value))
                } else {
                    Err(IdError { kind: $kind, prefix: Self::PREFIX, value })
                }
            }

            /// Generate a new, prefixed, time-ordered id.
            pub fn generate() -> Self {
                Self(format!("{}{}", Self::PREFIX, uuid::Uuid::now_v7().simple()))
            }

            /// Generate a new id with the canonical `prefix_` separator.
            ///
            /// The reference constructors always insert an underscore between
            /// the prefix and the sortable body (e.g. `que_`, `pty_`).
            pub fn create() -> Self {
                let separator = if Self::PREFIX.ends_with('_') { "" } else { "_" };
                Self(format!(
                    "{}{}{}",
                    Self::PREFIX,
                    separator,
                    uuid::Uuid::now_v7().simple()
                ))
            }

            /// Borrow the underlying string.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

prefixed_id!(
    /// Identifier for an event (`evt_`).
    EventId,
    "evt_",
    "event id"
);
prefixed_id!(
    /// Identifier for a session (`ses`).
    SessionId,
    "ses",
    "session id"
);
prefixed_id!(
    /// Identifier for a message (`msg_`).
    MessageId,
    "msg_",
    "message id"
);
prefixed_id!(
    /// Identifier for a workspace (`wrk`).
    WorkspaceId,
    "wrk",
    "workspace id"
);
prefixed_id!(
    /// Identifier for a permission request (`per`).
    PermissionId,
    "per",
    "permission id"
);
prefixed_id!(
    /// Identifier for a message part (`prt`).
    PartId,
    "prt",
    "part id"
);
prefixed_id!(
    /// Identifier for a pseudo-terminal (`pty`).
    PtyId,
    "pty",
    "pty id"
);
prefixed_id!(
    /// Identifier for a question request (`que`).
    QuestionId,
    "que",
    "question id"
);
