//! Shared UI primitives (Phase 7).
//!
//! Pre-alpha scaffold. See docs/PLAN.md and docs/TEST-PORT.md.
//!
//! Only the host-neutral, non-visual behaviour of the reference `ui` package is
//! ported here: scroll key/thumb math, plural selection, and the deterministic
//! parts of the markdown pipeline. Pixel/layout rendering is human-verified.

pub mod i18n;
pub mod markdown;
pub mod scroll_view;
