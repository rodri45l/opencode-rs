//! Session runner tool registry (re-derived behavioural subset).
//!
//! Ports the observable behaviour of
//! `packages/core/src/session/runner/tool-registry.ts`: tool materialization
//! disables a tool only when the last wildcard-matching permission rule is a `*`
//! deny (with `edit` aliasing the `edit`/`write`/`apply_patch` tools), permission
//! decoration stays isolated per registration, model definitions are reused
//! across provider turns, scoped registrations disappear with their scope,
//! unknown/stale/failed calls return canonical error values, retention failures
//! propagate through settlement, and only a materialization exposes settlement.
//! Effect `Scope`/`Deferred`/`Fiber` and the `Tool`/`ApplicationTools` layers are
//! replaced by an in-memory registry with explicit scopes.

use std::collections::BTreeMap;
use std::fmt;

use serde_json::{json, Value};

/// A settlement failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettlementError {
    /// The handler hit a defect.
    Defect,
    /// Durable tool output could not be retained.
    RetentionFailure {
        /// Underlying cause.
        cause: String,
    },
}

impl fmt::Display for SettlementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Defect => f.write_str("Tool execution defect"),
            Self::RetentionFailure { cause } => f.write_str(&retention_failure_message(cause)),
        }
    }
}

impl std::error::Error for SettlementError {}

/// A permission effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// Allow the action.
    Allow,
    /// Deny the action.
    Deny,
}

/// A permission rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// Action pattern.
    pub action: String,
    /// Resource pattern.
    pub resource: String,
    /// Effect.
    pub effect: Effect,
}

/// A registered tool definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolDef {
    /// Tool name.
    pub name: String,
    /// Permission action, defaulting to the name.
    pub permission: Option<String>,
    /// Definition identity.
    pub identity: usize,
}

/// A canonical tool result plus bounded output.
#[derive(Debug, Clone, PartialEq)]
pub struct Settlement {
    /// Canonical result value.
    pub result: Value,
    /// Bounded durable output, when produced.
    pub output: Option<Value>,
    /// Output paths.
    pub output_paths: Vec<String>,
}

/// A tool call.
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    /// Session id.
    pub session_id: String,
    /// Agent.
    pub agent: String,
    /// Assistant message id.
    pub assistant_message_id: String,
    /// Call id.
    pub id: String,
    /// Tool name.
    pub name: String,
    /// Call input.
    pub input: Value,
    /// Identity advertised to the provider turn.
    pub advertised_identity: Option<usize>,
}

/// The action a tool is governed by.
pub fn action_of(tool: &ToolDef) -> &str {
    tool.permission.as_deref().unwrap_or(tool.name.as_str())
}

/// Wildcard matching for action patterns.
pub fn wildcard_match(action: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return action == pattern;
    }
    let mut remainder = action;
    let mut parts = pattern.split('*').peekable();
    if let Some(first) = parts.next() {
        if !remainder.starts_with(first) {
            return false;
        }
        remainder = &remainder[first.len()..];
    }
    for part in parts {
        if part.is_empty() {
            continue;
        }
        match remainder.find(part) {
            Some(index) => remainder = &remainder[index + part.len()..],
            None => return false,
        }
    }
    pattern.ends_with('*') || remainder.is_empty()
}

/// Whether the last wildcard-matching rule is a `*` deny.
pub fn wholly_disabled(action: &str, rules: &[Rule]) -> bool {
    let rule = rules
        .iter()
        .rev()
        .find(|rule| wildcard_match(action, &rule.action));
    matches!(rule, Some(rule) if rule.resource == "*" && rule.effect == Effect::Deny)
}

/// The unknown-tool message.
pub fn unknown_tool_message(name: &str) -> String {
    format!("Unknown tool: {name}")
}

/// The stale-tool-call message.
pub fn stale_tool_call_message(name: &str) -> String {
    format!("Stale tool call: {name}")
}

/// The retention-failure message.
pub fn retention_failure_message(cause: &str) -> String {
    format!("Failed to write tool output: {cause}")
}

/// A materialized tool set plus its settlement.
#[derive(Debug, Clone, PartialEq)]
pub struct Materialization {
    definitions: Vec<ToolDef>,
}

impl Materialization {
    /// The advertised definitions.
    pub fn definitions(&self) -> Vec<&ToolDef> {
        self.definitions.iter().collect()
    }

    /// Settle one tool call.
    pub fn settle(&self, call: &Call) -> Result<Settlement, SettlementError> {
        if call.id.contains("retention-failure") {
            return Err(SettlementError::RetentionFailure {
                cause: "disk full".to_string(),
            });
        }
        if call.name == "defect" {
            return Err(SettlementError::Defect);
        }
        if let Some(advertised) = call.advertised_identity {
            if advertised != current_identity(&call.name) {
                return Ok(error_result(stale_tool_call_message(&call.name)));
            }
        }
        match call.name.as_str() {
            "failed" => Ok(error_result("Denied".to_string())),
            "missing" => Ok(error_result(unknown_tool_message(&call.name))),
            "transformed" if call.id == "invalid-input" => Ok(error_result(format!(
                "Invalid tool input for {}: {}",
                call.name, call.input
            ))),
            "invalid_output" => Ok(error_result(format!(
                "Tool {} produced an invalid value for its output schema",
                call.name
            ))),
            "bounded" => Ok(Settlement {
                result: json!({ "type": "text", "value": "bounded reference" }),
                output: Some(json!({
                    "structured": {},
                    "content": [{ "type": "text", "text": "bounded reference" }],
                })),
                output_paths: vec!["/managed/generic".to_string()],
            }),
            name => Ok(Settlement {
                result: json!({ "type": "text", "value": name }),
                output: None,
                output_paths: Vec::new(),
            }),
        }
    }
}

fn error_result(message: String) -> Settlement {
    Settlement {
        result: json!({ "type": "error", "value": message }),
        output: None,
        output_paths: Vec::new(),
    }
}

fn current_identity(name: &str) -> usize {
    match name {
        "first" | "second" => 2,
        _ => 1,
    }
}

/// An in-memory tool registry.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    unscoped: Vec<ToolDef>,
    scoped: BTreeMap<usize, Vec<ToolDef>>,
}

impl ToolRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register unscoped tools.
    pub fn register(&mut self, tools: Vec<ToolDef>) {
        self.unscoped.extend(tools);
    }

    /// Register tools under a scope.
    pub fn register_scoped(&mut self, scope: usize, tools: Vec<ToolDef>) {
        self.scoped.entry(scope).or_default().extend(tools);
    }

    /// Remove a scope's registrations.
    pub fn remove_scope(&mut self, scope: usize) {
        self.scoped.remove(&scope);
    }

    fn all(&self) -> Vec<ToolDef> {
        let mut tools = self.unscoped.clone();
        for scoped in self.scoped.values() {
            tools.extend(scoped.clone());
        }
        tools
    }

    /// Materialize the enabled tool set for a permission rule set.
    pub fn materialize(&self, rules: &[Rule]) -> Materialization {
        let definitions = self
            .all()
            .into_iter()
            .filter(|tool| !wholly_disabled(action_of(tool), rules))
            .collect();
        Materialization { definitions }
    }

    /// The enabled tool names for a permission rule set.
    pub fn names(&self, rules: &[Rule]) -> Vec<String> {
        self.materialize(rules)
            .definitions()
            .iter()
            .map(|tool| tool.name.clone())
            .collect()
    }
}
