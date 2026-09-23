//! Permission rulesets and evaluation.
//!
//! Ports the observable behaviour of `packages/opencode/src/permission`:
//! a ruleset is an ordered list of `permission`/`pattern`/`action` rules,
//! evaluated last-match-wins with `*` wildcards in either the action or the
//! resource segment. `from_config` flattens the nested config form, `merge`
//! concatenates rulesets (so the right-hand side wins), and `disabled` lists
//! the tools whose permission resolves to `deny`.

use serde_json::Value;
use std::collections::BTreeSet;

/// The effect a matching rule has on a permission check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// The action is allowed.
    Allow,
    /// The action is denied.
    Deny,
    /// The user is prompted.
    Ask,
}

impl Action {
    /// The wire spelling.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
            Self::Ask => "ask",
        }
    }

    /// Parse the wire spelling.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "allow" => Some(Self::Allow),
            "deny" => Some(Self::Deny),
            "ask" => Some(Self::Ask),
            _ => None,
        }
    }
}

/// A single permission rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// The action governed by the rule, e.g. `edit`.
    pub permission: String,
    /// The resource pattern, e.g. `*` or `src/*`.
    pub pattern: String,
    /// The effect when the rule matches.
    pub action: Action,
}

/// An ordered list of permission rules.
pub type Ruleset = Vec<Rule>;

/// The outcome of evaluating a permission against a ruleset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Evaluation {
    /// The resolved action.
    pub action: Action,
}

/// The permission a tool is evaluated under (`write`/`apply_patch` route
/// through `edit`).
fn tool_permission(tool: &str) -> &str {
    match tool {
        "write" | "apply_patch" => "edit",
        other => other,
    }
}

/// Match `text` against a `*`-wildcard `pattern`.
fn wildcard_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }
    let mut rest = text;
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if index == 0 {
            if let Some(tail) = rest.strip_prefix(part) {
                rest = tail;
            } else {
                return false;
            }
        } else if index == parts.len() - 1 {
            return rest.ends_with(part);
        } else if let Some(position) = rest.find(part) {
            rest = &rest[position + part.len()..];
        } else {
            return false;
        }
    }
    true
}

/// Flatten the nested config form into an ordered ruleset.
pub fn from_config(config: &Value) -> Ruleset {
    let mut rules = Ruleset::new();
    let Some(map) = config.as_object() else {
        return rules;
    };
    for (permission, value) in map {
        match value {
            Value::String(action) => {
                if let Some(action) = Action::parse(action) {
                    rules.push(Rule {
                        permission: permission.clone(),
                        pattern: "*".to_string(),
                        action,
                    });
                }
            }
            Value::Object(patterns) => {
                for (pattern, action) in patterns {
                    if let Some(action) = action.as_str().and_then(Action::parse) {
                        rules.push(Rule {
                            permission: permission.clone(),
                            pattern: pattern.clone(),
                            action,
                        });
                    }
                }
            }
            _ => {}
        }
    }
    rules
}

/// Concatenate two rulesets; the right-hand side takes precedence.
pub fn merge(left: &Ruleset, right: &Ruleset) -> Ruleset {
    let mut merged = left.clone();
    merged.extend(right.iter().cloned());
    merged
}

/// Evaluate `permission`/`pattern`, last matching rule wins.
pub fn evaluate(permission: &str, pattern: &str, ruleset: &Ruleset) -> Evaluation {
    let mut action = Action::Ask;
    for rule in ruleset {
        if wildcard_match(&rule.permission, permission) && wildcard_match(&rule.pattern, pattern) {
            action = rule.action;
        }
    }
    Evaluation { action }
}

/// The set of tools whose permission resolves to `deny` with pattern `*`.
pub fn disabled(tools: &[&str], ruleset: &Ruleset) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    for tool in tools {
        if evaluate(tool_permission(tool), "*", ruleset).action == Action::Deny {
            result.insert((*tool).to_string());
        }
    }
    result
}
