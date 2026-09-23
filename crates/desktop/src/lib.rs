//! Desktop shell (Phase 8).
//!
//! Pure desktop helper logic ported from `packages/desktop` (upstream 18ef3cc).

pub mod attachment_picker;
pub mod connections;
pub mod draft_store;
pub mod error;
pub mod external_url;
pub mod initialization;
pub mod initialization_forward;
pub mod install_state;
pub mod shell_env;
pub mod store_cleanup;
pub mod updater_controller;
pub mod updater_subscriptions;
pub mod window_registry;
pub mod wsl_servers;

pub use error::{AttachmentError, NotImplemented};
