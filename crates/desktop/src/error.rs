//! Typed errors shared by the desktop helpers.

/// An unimplemented or failed operation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct NotImplemented(pub String);

/// An attachment-picker failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct AttachmentError(pub String);
