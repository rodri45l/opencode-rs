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
pub mod bus;
pub mod catalog;
pub mod command;
pub mod config;
pub mod credential;
pub mod error;
pub mod event;
pub mod file_mutation;
pub mod fs_util;
pub mod global;
pub mod location;
pub mod model;
pub mod models_dev;
pub mod patch;
pub mod path;
pub mod policy;
pub mod shell;

pub use bus::{EventBus, EventSubscription, InMemoryEventBus};
pub use error::{CoreError, CoreResult};
pub use path::AbsolutePath;
