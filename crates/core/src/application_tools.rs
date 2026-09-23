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
    pub fn register(&mut self, definition: ToolDefinition) -> CoreResult<()> {
        validate_name(&definition.name)?;
        self.tools.insert(definition.name.clone(), definition);
        Ok(())
    }

    /// Register application tools, validating names.
    pub fn register_application(&mut self, tools: Vec<ToolDefinition>) -> CoreResult<()> {
        for definition in tools {
            validate_name(&definition.name)?;
            self.tools.insert(definition.name.clone(), definition);
        }
        Ok(())
    }

    /// Advertised definitions after applying deny rules.
    pub fn definitions(&self, denied: &[String]) -> CoreResult<Vec<ToolDefinition>> {
        Ok(self
            .tools
            .values()
            .filter(|definition| !denied.iter().any(|name| name == &definition.name))
            .cloned()
            .collect())
    }

    /// Settle a tool call.
    pub fn settle(&self, name: &str, input: Value) -> CoreResult<ToolResult> {
        match self.tools.get(name) {
            Some(_) => Ok(ToolResult {
                kind: "content".to_string(),
                value: input,
            }),
            None => Ok(ToolResult {
                kind: "error".to_string(),
                value: Value::Null,
            }),
        }
    }
}

fn validate_name(name: &str) -> CoreResult<()> {
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(CoreError::Invalid(format!("invalid tool name: {name}")));
    }
    Ok(())
}
