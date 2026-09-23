//! HTTP protocol surface: route groups, payloads, and the error taxonomy.
//!
//! Mirrors `packages/protocol` in the reference implementation. The error wire
//! shape is an internally tagged object with a `_tag` discriminator, e.g.
//! `{ "_tag": "SessionNotFoundError", "sessionID": "ses_...", "message": "..." }`.

pub mod cursor;
pub mod error;
pub mod routes;

pub use cursor::{
    decode_cursor, encode_cursor, AnchorDirection, InvalidCursor, ListAnchor, Order,
    SessionHistoryQuery, SessionsCursorInput, HISTORY_LIMIT_MAX,
};
pub use error::{ApiError, ApiResult, ErrorField, ErrorSpec, ERROR_SPECS};
pub use routes::{Cursor, ImplementedRoute, Page, IMPLEMENTED_ROUTES};
