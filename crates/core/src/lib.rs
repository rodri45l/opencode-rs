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
pub mod groq;
pub mod guidance;
pub mod instruction_context;
pub mod keyed_mutex;
pub mod location;
pub mod location_filesystem;
pub mod location_layer;
pub mod location_mutation;
pub mod mistral;
pub mod model;
pub mod models_dev;
pub mod models_dev_plugin;
pub mod move_session;
pub mod npm;
pub mod npm_config;
pub mod patch;
pub mod path;
pub mod permission;
pub mod plugin_skill;
pub mod policy;
pub mod preload;
pub mod project_directories;
pub mod provider;
pub mod provider_azure;
pub mod provider_azure_cognitive_services;
pub mod provider_gitlab;
pub mod provider_google_vertex;
pub mod provider_header_plugins;
pub mod provider_options;
pub mod provider_plugins;
pub mod provider_sap_ai_core;
pub mod provider_sdk_plugins;
pub mod provider_snowflake_cortex;
pub mod pty;
pub mod question;
pub mod reference;
pub mod reference_guidance;
pub mod repository;
pub mod repository_cache;
pub mod ripgrep;
pub mod session_compaction;
pub mod session_history;
pub mod session_runner_tool_events;
pub mod shared_schema;
pub mod shell;
pub mod skill_discovery;
pub mod snapshot;
pub mod state;
pub mod system_context;
pub mod tool_output_store;
pub mod tool_webfetch;
pub mod tool_websearch;
pub mod variant;
pub mod which;
pub mod xai_responses;

pub use bus::{EventBus, EventSubscription, InMemoryEventBus};
pub use error::{CoreError, CoreResult};
pub use path::AbsolutePath;
