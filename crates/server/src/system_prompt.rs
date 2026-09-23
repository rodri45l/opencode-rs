//! System prompt composition.
//!
//! Ports the observable behaviour of `packages/opencode/src/session/system.ts`:
//! provider-specific prompt selection, the sorted skills block, and the MCP
//! instructions block filtered by permission.

use opencode_core::{CoreError, CoreResult};

use crate::agent::AgentInfo;
use crate::permission::Ruleset;

/// System prompt composition.
#[derive(Debug, Default)]
pub struct SystemPrompt;

impl SystemPrompt {
    /// Create the composer.
    pub fn new() -> Self {
        Self
    }

    /// Provider-specific prompts for a model's API id.
    pub fn provider(_api_id: &str) -> Vec<String> {
        vec![String::new()]
    }

    /// The skills block for an agent, sorted by name.
    pub fn skills(&self, _agent: &AgentInfo) -> CoreResult<Option<String>> {
        Err(CoreError::NotImplemented(
            "system_prompt::SystemPrompt::skills",
        ))
    }

    /// The MCP instructions block, filtered by `permission`.
    pub fn mcp(&self, _agent: &AgentInfo, _permission: &Ruleset) -> CoreResult<String> {
        Err(CoreError::NotImplemented(
            "system_prompt::SystemPrompt::mcp",
        ))
    }
}
