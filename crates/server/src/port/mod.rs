//! Port surface for the second porting wave (agent loop, tools, plugins, MCP,
//! providers, skills).
//!
//! Pure helpers whose behaviour the ported tests pin are implemented; heavier
//! runtime surfaces are typed stubs that return [`crate::tools::ToolError`] or
//! a local `NotImplemented` error until their module lands. No `panic!`/`todo!`.

pub mod glob_util;
pub mod mcp;
pub mod message_v2;
pub mod plugin;
pub mod provider;
pub mod provider_transform;
pub mod session;
pub mod session_compaction;
pub mod skill;
pub mod tool_parameters;
pub mod tools;
