//! Bridge the protocol error taxonomy into axum responses.
//!
//! `ApiError` is defined in `opencode-protocol`, so it cannot implement axum's
//! `IntoResponse` directly (orphan rule). This newtype carries the conversion.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use opencode_protocol::ApiError;

/// Wrapper making [`ApiError`] renderable as an HTTP response.
#[derive(Debug)]
pub struct ApiErrorResponse(pub ApiError);

impl From<ApiError> for ApiErrorResponse {
    fn from(error: ApiError) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.0.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self.0)).into_response()
    }
}
