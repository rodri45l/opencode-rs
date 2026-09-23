//! Port surface for the second porting wave (agent loop, tools, plugins, MCP,
//! providers, skills).
//!
//! Pure helpers whose behaviour the ported tests pin are implemented; heavier
//! runtime surfaces are typed stubs that return [`crate::tools::ToolError`] or
//! a local `NotImplemented` error until their module lands. No `panic!`/`todo!`.

pub mod mcp;
pub mod plugin;
pub mod provider;
pub mod session;
pub mod skill;
pub mod tools;
