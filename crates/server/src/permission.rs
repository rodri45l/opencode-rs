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

impl PartialEq<&str> for Action {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<str> for Action {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
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

/// The set of tools whose last matching permission rule denies the `*` pattern.
pub fn disabled(tools: &[&str], ruleset: &Ruleset) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    for tool in tools {
        let permission = tool_permission(tool);
        let last = ruleset
            .iter()
            .rev()
            .find(|rule| wildcard_match(&rule.permission, permission));
        if let Some(rule) = last {
            if rule.pattern == "*" && rule.action == Action::Deny {
                result.insert((*tool).to_string());
            }
        }
    }
    result
}

/// An ordered config value for a single permission key.
///
/// The reference `fromConfig` accepts a record whose key order is significant;
/// this ordered representation preserves that order without relying on a
/// sorted map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigValue {
    /// A bare action string, applied to the `*` pattern.
    Action(String),
    /// An ordered list of `pattern -> action` pairs.
    Rules(Vec<(String, String)>),
}

/// Error returned by the ordered permission helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionError {
    /// The module has not been implemented.
    NotImplemented(&'static str),
}

impl std::fmt::Display for PermissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "porting: {what} not implemented"),
        }
    }
}

impl std::error::Error for PermissionError {}

/// Expand a leading `~`/`$HOME` path segment to the user's home directory.
fn expand_home(pattern: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    if pattern == "~" || pattern == "$HOME" {
        return home;
    }
    if let Some(rest) = pattern.strip_prefix("~/") {
        return format!("{home}/{rest}");
    }
    if let Some(rest) = pattern.strip_prefix("$HOME/") {
        return format!("{home}/{rest}");
    }
    pattern.to_string()
}

/// Flatten an ordered config into a ruleset, preserving key/pattern order.
pub fn from_config_entries(entries: &[(&str, ConfigValue)]) -> Result<Vec<Rule>, PermissionError> {
    let mut rules = Vec::new();
    for (permission, value) in entries {
        match value {
            ConfigValue::Action(action) => {
                if let Some(action) = Action::parse(action) {
                    rules.push(Rule {
                        permission: (*permission).to_string(),
                        pattern: "*".to_string(),
                        action,
                    });
                }
            }
            ConfigValue::Rules(pairs) => {
                for (pattern, action) in pairs {
                    if let Some(action) = Action::parse(action) {
                        rules.push(Rule {
                            permission: (*permission).to_string(),
                            pattern: expand_home(pattern),
                            action,
                        });
                    }
                }
            }
        }
    }
    Ok(rules)
}

/// Evaluate across an ordered list of rulesets, last match wins.
pub fn evaluate_multi(
    permission: &str,
    pattern: &str,
    rulesets: &[&[Rule]],
) -> Result<Evaluation, PermissionError> {
    let mut action = Action::Ask;
    for ruleset in rulesets {
        for rule in *ruleset {
            if wildcard_match(&rule.permission, permission)
                && wildcard_match(&rule.pattern, pattern)
            {
                action = rule.action;
            }
        }
    }
    Ok(Evaluation { action })
}

/// Concatenate ordered rulesets, preserving order (right-hand side wins).
pub fn merge_multi(rulesets: &[&[Rule]]) -> Result<Vec<Rule>, PermissionError> {
    let mut merged = Vec::new();
    for ruleset in rulesets {
        merged.extend(ruleset.iter().cloned());
    }
    Ok(merged)
}

/// Tools whose last matching permission rule denies the `*` pattern.
pub fn disabled_multi(
    tools: &[&str],
    ruleset: &[Rule],
) -> Result<BTreeSet<String>, PermissionError> {
    let mut result = BTreeSet::new();
    for tool in tools {
        let permission = tool_permission(tool);
        let last = ruleset
            .iter()
            .rev()
            .find(|rule| wildcard_match(&rule.permission, permission));
        if let Some(rule) = last {
            if rule.pattern == "*" && rule.action == Action::Deny {
                result.insert((*tool).to_string());
            }
        }
    }
    Ok(result)
}

/// Longest known command-prefix arity for a bash-style command.
///
/// Returns the tokens that identify the "human-understandable command": the
/// longest prefix present in the arity table, otherwise the first token (or
/// empty for an empty input).
pub fn bash_arity_prefix(tokens: &[&str]) -> Vec<String> {
    if tokens.is_empty() {
        return Vec::new();
    }
    for len in (1..=tokens.len()).rev() {
        let prefix = tokens[..len].join(" ");
        if let Some(&arity) = BASH_ARITY
            .iter()
            .find(|(key, _)| *key == prefix)
            .map(|(_, arity)| arity)
        {
            return tokens[..arity.min(tokens.len())]
                .iter()
                .map(|token| (*token).to_string())
                .collect();
        }
    }
    vec![tokens[0].to_string()]
}

/// Known command-prefix arities, longest-match-wins.
#[rustfmt::skip]
const BASH_ARITY: &[(&str, usize)] = &[
    ("cat", 1), ("cd", 1), ("chmod", 1), ("chown", 1), ("cp", 1), ("echo", 1),
    ("env", 1), ("export", 1), ("grep", 1), ("kill", 1), ("killall", 1),
    ("ln", 1), ("ls", 1), ("mkdir", 1), ("mv", 1), ("ps", 1), ("pwd", 1),
    ("rm", 1), ("rmdir", 1), ("sleep", 1), ("source", 1), ("tail", 1),
    ("touch", 1), ("unset", 1), ("which", 1),
    ("aws", 3), ("az", 3),
    ("bazel", 2), ("brew", 2), ("bun", 2), ("bun run", 3), ("bun x", 3),
    ("cargo", 2), ("cargo add", 3), ("cargo run", 3),
    ("cdk", 2), ("cf", 2), ("cmake", 2), ("composer", 2),
    ("consul", 2), ("consul kv", 3),
    ("crictl", 2),
    ("deno", 2), ("deno task", 3), ("doctl", 3),
    ("docker", 2), ("docker builder", 3), ("docker compose", 3),
    ("docker container", 3), ("docker image", 3), ("docker network", 3),
    ("docker volume", 3),
    ("eksctl", 2), ("eksctl create", 3),
    ("firebase", 2), ("flyctl", 2),
    ("gcloud", 3), ("gh", 3),
    ("git", 2), ("git config", 3), ("git remote", 3), ("git stash", 3),
    ("go", 2), ("gradle", 2),
    ("helm", 2), ("heroku", 2), ("hugo", 2),
    ("ip", 2), ("ip addr", 3), ("ip link", 3), ("ip netns", 3), ("ip route", 3),
    ("kind", 2), ("kind create", 3),
    ("kubectl", 2), ("kubectl kustomize", 3), ("kubectl rollout", 3),
    ("kustomize", 2),
    ("make", 2), ("mc", 2), ("mc admin", 3), ("minikube", 2),
    ("mongosh", 2), ("mysql", 2), ("mvn", 2),
    ("ng", 2), ("npm", 2), ("npm exec", 3), ("npm init", 3), ("npm run", 3),
    ("npm view", 3),
    ("nvm", 2), ("nx", 2),
    ("openssl", 2), ("openssl req", 3), ("openssl x509", 3),
    ("pip", 2), ("pipenv", 2), ("pnpm", 2), ("pnpm dlx", 3), ("pnpm exec", 3),
    ("pnpm run", 3), ("poetry", 2), ("podman", 2), ("podman container", 3),
    ("podman image", 3), ("psql", 2), ("pulumi", 2), ("pulumi stack", 3),
    ("pyenv", 2), ("python", 2),
    ("rake", 2), ("rbenv", 2), ("redis-cli", 2), ("rustup", 2),
    ("serverless", 2), ("sfdx", 3), ("skaffold", 2), ("sls", 2), ("sst", 2),
    ("swift", 2), ("systemctl", 2),
    ("terraform", 2), ("terraform workspace", 3),
    ("tmux", 2), ("turbo", 2),
    ("ufw", 2),
    ("vault", 2), ("vault auth", 3), ("vault kv", 3), ("vercel", 2), ("volta", 2),
    ("wp", 2),
    ("yarn", 2), ("yarn dlx", 3), ("yarn run", 3),
];
