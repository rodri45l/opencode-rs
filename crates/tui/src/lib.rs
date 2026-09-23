//! Terminal user interface (Phase 7).
//!
//! Pre-alpha scaffold. See docs/PLAN.md and docs/TEST-PORT.md.
//!
//! Only the host-neutral, non-visual behaviour of the reference `tui` package
//! is ported here: keybinding-independent formatting, config/keymap mappings,
//! state reducers, history/scroll models, and message/part projections. TUI
//! rendering is human-verified.

pub mod apply_patch_file;
pub mod clipboard;
pub mod dialog_session_list;
pub mod dialog_workspace_create;
pub mod diff_viewer_file_tree;
pub mod editor;
pub mod error;
pub mod filetype;
pub mod format;
pub mod local;
pub mod local_attachment;
pub mod markdown_code_state;
pub mod markdown_inline_code_kind;
pub mod markdown_stream;
pub mod markdown_worker_protocol;
pub mod markdown_worker_transport;
pub mod message_file;
pub mod message_part_text;
pub mod model;
pub mod model_options;
pub mod notifications;
pub mod part_default_open;
pub mod persistence;
pub mod plugin_runtime;
pub mod presentation;
pub mod prompt_display;
pub mod prompt_history;
pub mod prompt_input_machine;
pub mod prompt_jsonl;
pub mod prompt_part;
pub mod prompt_submit;
pub mod prompt_traits;
pub mod provider_options;
pub mod renderer;
pub mod revert_diff;
pub mod runtime;
pub mod session_diff;
pub mod session_review_file_preview_v2_virtualize;
pub mod session_util;
pub mod sync;
pub mod sync_hydration;
pub mod sync_undefined_messages;
pub mod theme;
pub mod thinking;
pub mod tool_display;
pub mod transcript;
