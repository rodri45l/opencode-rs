//! HTTP protocol surface: route groups, payloads, and the error taxonomy.
//!
//! Mirrors `packages/protocol` in the reference implementation. The error wire
//! shape is an internally tagged object with a `_tag` discriminator, e.g.
//! `{ "_tag": "SessionNotFoundError", "sessionID": "ses_...", "message": "..." }`.

pub mod codegen;
pub mod cursor;
pub mod error;
pub mod routes;

pub use codegen::{
    compile, emit_effect, emit_effect_imported, emit_promise, write, ApiSpec, CompileOptions,
    Contract, DeclaredError, Endpoint, EndpointSpec, FieldSpec, GeneratedFile, GenerationError,
    Group, GroupSpec, HttpMethod, ImportedSource, ImportedWireType, InputField, InputMode,
    InputSource, Operation, Output, OutputFs, PromiseEmitOptions, SchemaIssue, SuccessKind,
    SuccessSpec, MANIFEST_NAME,
};
pub use cursor::{
    decode_cursor, encode_cursor, AnchorDirection, InvalidCursor, ListAnchor, Order,
    SessionHistoryQuery, SessionsCursorInput, HISTORY_LIMIT_MAX,
};
pub use error::{ApiError, ApiResult, ErrorField, ErrorSpec as ErrorTaxonomySpec, ERROR_SPECS};
pub use routes::{Cursor, ImplementedRoute, Page, IMPLEMENTED_ROUTES};
