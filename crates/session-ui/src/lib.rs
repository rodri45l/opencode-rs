//! Session UI components (Phase 7).
//!
//! Pre-alpha scaffold. See docs/PLAN.md and docs/TEST-PORT.md.
//!
//! Only the host-neutral, non-visual behaviour of the reference `session-ui`
//! package is ported here: diff projection, markdown streaming/worker state,
//! message/part projections, and the prompt-input interaction machine/store.
//! Component rendering is human-verified.

pub mod apply_patch_file;
pub mod markdown_code_state;
pub mod markdown_inline_code_kind;
pub mod markdown_stream;
pub mod markdown_worker_protocol;
pub mod markdown_worker_queue;
pub mod markdown_worker_transport;
pub mod message_file;
pub mod message_part_text;
pub mod part_default_open;
pub mod prompt_input_machine;
pub mod prompt_input_store;
pub mod prompt_types;
pub mod session_diff;
pub mod session_review_file_preview_v2_virtualize;
