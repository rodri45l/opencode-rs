//! Agent definitions and subagent session permissions.
//!
//! Ports the observable behaviour of `packages/opencode/src/agent/agent.ts`
//! and `agent/subagent-permissions.ts`: a catalog of built-in agents (plus
//! config-defined agents), and the helper that derives a subagent's session
//! permission from its parent session so parent denials remain hard ceilings.

use serde_json::Value;

use crate::permission::{from_config, Ruleset};

/// Whether an agent is selectable directly or only as a subagent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMode {
    /// Selectable as the top-level agent.
    Primary,
    /// Invocable only by another agent.
    Subagent,
    /// Selectable directly and as a subagent.
    All,
}

/// A materialized agent definition.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentInfo {
    /// Agent name.
    pub name: String,
    /// Primary/subagent mode.
    pub mode: AgentMode,
    /// Human description, when configured.
    pub description: Option<String>,
    /// Permission rules.
    pub permission: Ruleset,
    /// Provider/model options.
    pub options: Value,
}

/// A catalog of built-in and config-defined agents.
#[derive(Debug, Clone, Default)]
pub struct AgentCatalog {
    config: Value,
}

impl AgentCatalog {
    /// Create a catalog with only the built-in agents.
    pub fn new() -> Self {
        Self {
            config: Value::Null,
        }
    }

    /// Create a catalog that also exposes config-defined agents.
    pub fn with_config(config: Value) -> Self {
        Self { config }
    }

    /// Look up an agent by name.
    pub fn get(&self, name: &str) -> Option<AgentInfo> {
        if let Some(agent) = self.config_agent(name) {
            return Some(agent);
        }
        builtin_agent(name)
    }

    /// All known agents (built-ins followed by config-defined).
    pub fn list(&self) -> Vec<AgentInfo> {
        let mut agents: Vec<AgentInfo> = ["build", "plan", "general", "explore"]
            .iter()
            .filter_map(|name| builtin_agent(name))
            .collect();
        if let Some(map) = self.config.get("agent").and_then(Value::as_object) {
            for (name, _) in map {
                if let Some(agent) = self.config_agent(name) {
                    agents.push(agent);
                }
            }
        }
        agents
    }

    fn config_agent(&self, name: &str) -> Option<AgentInfo> {
        let agent = self.config.get("agent")?.get(name)?;
        let mode = match agent.get("mode").and_then(Value::as_str) {
            Some("subagent") => AgentMode::Subagent,
            Some("all") => AgentMode::All,
            _ => AgentMode::Primary,
        };
        Some(AgentInfo {
            name: name.to_string(),
            mode,
            description: agent
                .get("description")
                .and_then(Value::as_str)
                .map(str::to_string),
            permission: agent.get("permission").map(from_config).unwrap_or_default(),
            options: agent.get("options").cloned().unwrap_or(Value::Null),
        })
    }
}

fn builtin_agent(name: &str) -> Option<AgentInfo> {
    let (mode, description, permission) = match name {
        "build" => (
            AgentMode::Primary,
            "Build agent",
            serde_json::json!({ "*": "allow" }),
        ),
        "plan" => (
            AgentMode::Primary,
            "Plan agent",
            serde_json::json!({ "*": "allow", "edit": "deny" }),
        ),
        "general" => (
            AgentMode::Subagent,
            "General subagent",
            serde_json::json!({ "*": "allow" }),
        ),
        "explore" => (
            AgentMode::Subagent,
            "Read-only explorer",
            serde_json::json!({ "*": "deny", "read": "allow" }),
        ),
        _ => return None,
    };
    Some(AgentInfo {
        name: name.to_string(),
        mode,
        description: Some(description.to_string()),
        permission: from_config(&permission),
        options: Value::Null,
    })
}

/// Derive the session permission handed to a subagent.
///
/// The parent session permission is returned as-is so that, when merged after
/// the subagent's own permission, parent denials act as hard runtime ceilings.
pub fn derive_subagent_session_permission(
    parent_session_permission: &Ruleset,
    _subagent: &AgentInfo,
) -> Ruleset {
    parent_session_permission.clone()
}
