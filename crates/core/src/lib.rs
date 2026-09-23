//! Core services: event bus and (later) session engine, storage, tools.
//!
//! Phase 1 only needs an in-process event bus. Storage and the agent loop land
//! in later phases; the trait boundaries live here so the server crate never
//! depends on a concrete implementation.

pub mod bus;

pub use bus::{EventBus, EventSubscription, InMemoryEventBus};
