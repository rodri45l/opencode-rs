//! Application and location tool registration.
//!
//! Ports the observable behaviour of `packages/core/src/tool/application-tools.ts`
//! and `tool/tools.ts`: opaque application handlers are carried through
//! registration, names are validated, deny rules filter advertised definitions,
//! unknown tools settle as errors, and same-name location tools keep precedence.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::{CoreError, CoreResult};

/// A registered tool definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolDefinition {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
}

/// A tool execution outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolResult {
    /// Result kind (`content`, `error`, ...).
    pub kind: String,
    /// Result value.
    pub value: Value,
}

/// The tool registry.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    tools: BTreeMap<String, ToolDefinition>,
}

impl ToolRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a location tool.
    pub fn register(&mut self, _definition: ToolDefinition) -> CoreResult<()> {
        let _ = &self.tools;
        Err(CoreError::NotImplemented(
            "application_tools::ToolRegistry::register",
        ))
    }

    /// Register application tools, validating names.
    pub fn register_application(&mut self, _tools: Vec<ToolDefinition>) -> CoreResult<()> {
        Err(CoreError::NotImplemented(
            "application_tools::ToolRegistry::register_application",
        ))
    }

    /// Advertised definitions after applying deny rules.
    pub fn definitions(&self, _denied: &[String]) -> CoreResult<Vec<ToolDefinition>> {
        Err(CoreError::NotImplemented(
            "application_tools::ToolRegistry::definitions",
        ))
    }

    /// Settle a tool call.
    pub fn settle(&self, _name: &str, _input: Value) -> CoreResult<ToolResult> {
        Err(CoreError::NotImplemented(
            "application_tools::ToolRegistry::settle",
        ))
    }
}
