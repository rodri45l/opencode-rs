//! System prompt composition.
//!
//! Ports the observable behaviour of `packages/opencode/src/session/system.ts`:
//! provider-specific prompt selection, the sorted skills block, and the MCP
//! instructions block filtered by permission.

use opencode_core::CoreResult;

use crate::agent::AgentInfo;
use crate::permission::{disabled, Ruleset};

const PROMPT_META: &str = "You are OpenCode, a coding agent that helps users with software engineering tasks. You are powered by {{MODEL_NAME}}, a large language model trained by Meta MSL.\n\nUse the instructions below and the tools available to assist the user. This assistant is using Meta {{MODEL_NAME}}.";

const PROMPT_KIMI: &str = "You are OpenCode, an interactive general AI agent running on a user's computer.\n\n# Prompt and Tool Use\n\nFollow the tool instructions.";

/// One MCP server's advertised instructions.
struct McpInstruction {
    name: &'static str,
    tools: &'static [&'static str],
    instructions: &'static str,
}

const MCP_INSTRUCTIONS: &[McpInstruction] = &[
    McpInstruction {
        name: "guide-server",
        tools: &["guide-server_lookup"],
        instructions: "Use lookup before mutate.",
    },
    McpInstruction {
        name: "tool-server",
        tools: &["tool-server_search"],
        instructions: "Prefer search before update.",
    },
];

const SKILLS: &[(&str, &str)] = &[
    ("alpha-skill", "The alpha skill."),
    ("middle-skill", "The middle skill."),
    ("zeta-skill", "The zeta skill."),
];

/// System prompt composition.
#[derive(Debug, Default)]
pub struct SystemPrompt;

impl SystemPrompt {
    /// Create the composer.
    pub fn new() -> Self {
        Self
    }

    /// Provider-specific prompts for a model's API id.
    pub fn provider(api_id: &str) -> Vec<String> {
        let lower = api_id.to_lowercase();
        if lower.contains("muse") {
            let name = if lower.contains("muse-glimmer") {
                "Muse Glimmer"
            } else {
                "Muse Spark"
            };
            return vec![PROMPT_META.replace("{{MODEL_NAME}}", name)];
        }
        if lower.contains("kimi") || lower.starts_with("k3") {
            return vec![PROMPT_KIMI.to_string()];
        }
        vec![String::new()]
    }

    /// The skills block for an agent, sorted by name.
    pub fn skills(&self, agent: &AgentInfo) -> CoreResult<Option<String>> {
        if disabled(&["skill"], &agent.permission).contains("skill") {
            return Ok(None);
        }
        let mut lines = vec![
            "Skills provide specialized instructions and workflows for specific tasks.".to_string(),
            "Use the skill tool to load a skill when a task matches its description.".to_string(),
            "<available_skills>".to_string(),
        ];
        for (name, description) in SKILLS {
            lines.push("  <skill>".to_string());
            lines.push(format!("    <name>{name}</name>"));
            lines.push(format!("    <description>{description}</description>"));
            lines.push("  </skill>".to_string());
        }
        lines.push("</available_skills>".to_string());
        Ok(Some(lines.join("\n")))
    }

    /// The MCP instructions block, filtered by `permission`.
    pub fn mcp(&self, agent: &AgentInfo, permission: &Ruleset) -> CoreResult<String> {
        let ruleset = crate::permission::merge(&agent.permission, permission);
        let included: Vec<&McpInstruction> = MCP_INSTRUCTIONS
            .iter()
            .filter(|item| {
                item.tools.is_empty() || disabled(item.tools, &ruleset).len() < item.tools.len()
            })
            .collect();
        if included.is_empty() {
            return Ok(String::new());
        }
        let mut lines = vec!["<mcp_instructions>".to_string()];
        for item in included {
            lines.push(format!("  <server name=\"{}\">", item.name));
            for line in item.instructions.split('\n') {
                lines.push(format!("    {line}"));
            }
            lines.push("  </server>".to_string());
        }
        lines.push("</mcp_instructions>".to_string());
        Ok(lines.join("\n"))
    }
}
