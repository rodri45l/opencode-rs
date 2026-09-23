//! Core services: event bus and (later) session engine, storage, tools.
//!
//! Phase 1 only needs an in-process event bus. Storage and the agent loop land
//! in later phases; the trait boundaries live here so the server crate never
//! depends on a concrete implementation.
//!
//! The modules below are the public surface for the red-first test port (see
//! `docs/TEST-PORT.md`). Their bodies are stubs returning
//! [`CoreError::NotImplemented`] until the corresponding module lands.

pub mod agent;
pub mod application_tools;
pub mod background_job;
pub mod bus;
pub mod catalog;
pub mod command;
pub mod config;
pub mod config_skill;
pub mod copilot_convert;
pub mod credential;
pub mod error;
pub mod event;
pub mod file_mutation;
pub mod fs_ignore;
pub mod fs_util;
pub mod global;
pub mod instruction_context;
pub mod keyed_mutex;
pub mod location;
pub mod location_filesystem;
pub mod location_mutation;
pub mod model;
pub mod models_dev;
pub mod npm_config;
pub mod patch;
pub mod path;
pub mod policy;
pub mod provider_options;
pub mod provider_plugins;
pub mod pty;
pub mod reference;
pub mod reference_guidance;
pub mod repository;
pub mod session_compaction;
pub mod shell;
pub mod system_context;
pub mod tool_output_store;
pub mod which;

pub use bus::{EventBus, EventSubscription, InMemoryEventBus};
pub use error::{CoreError, CoreResult};
pub use path::AbsolutePath;
