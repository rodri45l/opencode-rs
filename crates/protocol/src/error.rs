//! The tagged error taxonomy.
//!
//! Field names and HTTP status codes are taken from `packages/protocol/src/errors.ts`.
//! Optional fields are omitted from the wire body when unset.

use serde::{Deserialize, Serialize};

/// Every error the API can return, tagged by `_tag`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[serde(tag = "_tag")]
pub enum ApiError {
    #[error("{message}")]
    #[serde(rename = "InvalidRequestError")]
    InvalidRequest {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        kind: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        field: Option<String>,
    },

    #[error("{message}")]
    #[serde(rename = "UnauthorizedError")]
    Unauthorized { message: String },

    #[error("{message}")]
    #[serde(rename = "ForbiddenError")]
    Forbidden { message: String },

    #[error("{message}")]
    #[serde(rename = "ConflictError")]
    Conflict {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        resource: Option<String>,
    },

    #[error("{message}")]
    #[serde(rename = "ServiceUnavailableError")]
    ServiceUnavailable {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        service: Option<String>,
    },

    #[error("{message}")]
    #[serde(rename = "UnknownError")]
    Unknown {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        #[serde(rename = "ref")]
        reference: Option<String>,
    },

    #[error("{message}")]
    #[serde(rename = "ProviderNotFoundError")]
    ProviderNotFound {
        #[serde(rename = "providerID")]
        provider_id: String,
        message: String,
    },

    #[error("{message}")]
    #[serde(rename = "SessionNotFoundError")]
    SessionNotFound {
        #[serde(rename = "sessionID")]
        session_id: String,
        message: String,
    },

    #[error("{message}")]
    #[serde(rename = "MessageNotFoundError")]
    MessageNotFound {
        #[serde(rename = "sessionID")]
        session_id: String,
        #[serde(rename = "messageID")]
        message_id: String,
        message: String,
    },

    #[error("{message}")]
    #[serde(rename = "PermissionNotFoundError")]
    PermissionNotFound {
        #[serde(rename = "requestID")]
        request_id: String,
        message: String,
    },

    #[error("{message}")]
    #[serde(rename = "QuestionNotFoundError")]
    QuestionNotFound {
        #[serde(rename = "requestID")]
        request_id: String,
        message: String,
    },

    #[error("{message}")]
    #[serde(rename = "PtyNotFoundError")]
    PtyNotFound {
        #[serde(rename = "ptyID")]
        pty_id: String,
        message: String,
    },

    #[error("{message}")]
    #[serde(rename = "InvalidCursorError")]
    InvalidCursor { message: String },
}

impl ApiError {
    /// The `_tag` discriminator value.
    pub const fn tag(&self) -> &'static str {
        match self {
            ApiError::InvalidRequest { .. } => "InvalidRequestError",
            ApiError::Unauthorized { .. } => "UnauthorizedError",
            ApiError::Forbidden { .. } => "ForbiddenError",
            ApiError::Conflict { .. } => "ConflictError",
            ApiError::ServiceUnavailable { .. } => "ServiceUnavailableError",
            ApiError::Unknown { .. } => "UnknownError",
            ApiError::ProviderNotFound { .. } => "ProviderNotFoundError",
            ApiError::SessionNotFound { .. } => "SessionNotFoundError",
            ApiError::MessageNotFound { .. } => "MessageNotFoundError",
            ApiError::PermissionNotFound { .. } => "PermissionNotFoundError",
            ApiError::QuestionNotFound { .. } => "QuestionNotFoundError",
            ApiError::PtyNotFound { .. } => "PtyNotFoundError",
            ApiError::InvalidCursor { .. } => "InvalidCursorError",
        }
    }

    /// The HTTP status code for this error.
    pub const fn status(&self) -> u16 {
        match self {
            ApiError::InvalidRequest { .. } | ApiError::InvalidCursor { .. } => 400,
            ApiError::Unauthorized { .. } => 401,
            ApiError::Forbidden { .. } => 403,
            ApiError::Conflict { .. } => 409,
            ApiError::ServiceUnavailable { .. } => 503,
            ApiError::Unknown { .. } => 500,
            ApiError::ProviderNotFound { .. }
            | ApiError::SessionNotFound { .. }
            | ApiError::MessageNotFound { .. }
            | ApiError::PermissionNotFound { .. }
            | ApiError::QuestionNotFound { .. }
            | ApiError::PtyNotFound { .. } => 404,
        }
    }

    /// A generic internal error.
    pub fn unknown(message: impl Into<String>) -> Self {
        ApiError::Unknown {
            message: message.into(),
            reference: None,
        }
    }
}

/// Convenience alias used across handlers.
pub type ApiResult<T> = Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_with_tag_and_omits_optional_fields() {
        let err = ApiError::SessionNotFound {
            session_id: "ses_abc".into(),
            message: "not found".into(),
        };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["_tag"], "SessionNotFoundError");
        assert_eq!(json["sessionID"], "ses_abc");
        assert_eq!(err.status(), 404);
    }

    #[test]
    fn invalid_request_omits_kind_and_field() {
        let err = ApiError::InvalidRequest {
            message: "bad".into(),
            kind: None,
            field: None,
        };
        let json = serde_json::to_value(&err).unwrap();
        assert!(json.get("kind").is_none());
        assert!(json.get("field").is_none());
        assert_eq!(err.status(), 400);
        assert_eq!(err.tag(), "InvalidRequestError");
    }

    #[test]
    fn roundtrips() {
        let err = ApiError::Conflict {
            message: "busy".into(),
            resource: Some("ses_1".into()),
        };
        let json = serde_json::to_string(&err).unwrap();
        let back: ApiError = serde_json::from_str(&json).unwrap();
        assert_eq!(back, err);
    }
}
