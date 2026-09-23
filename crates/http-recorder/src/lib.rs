//! HTTP record/replay cassettes for provider and integration tests.
//!
//! Pre-alpha scaffold. See docs/PLAN.md and docs/TEST-PORT.md.

pub mod cassette;
pub mod redaction;

pub use cassette::{cassette_path, has_cassette_sync, list_cassettes, CassetteError};
pub use redaction::{
    redact_headers, redact_url, redacted_error_request, secret_findings, Json, Redactor,
    RedactorOptions, RequestSnapshot, ResponseSnapshot, SecretFinding, UrlRedactor,
    DEFAULT_REDACT_HEADERS, DEFAULT_REDACT_QUERY, REDACTED,
};
