//! Port of packages/opencode/test/permission/next.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned (pure surface only): `fromConfig` ordering + `~`/`$HOME`
//! expansion, `evaluate` last-matching-rule semantics with wildcard
//! permissions/patterns, `merge` concatenation, and `disabled` classification.
//!
//! The reference file's `ask`/`reply`/instance-lifecycle suites require the
//! live Effect service and are tracked as skipped (see PORT-STATUS.s5.json).
#![allow(dead_code)]

// Fast-wave local stubs: `permission::next` is not implemented in this crate yet.
mod permission {
    use std::collections::BTreeSet;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Rule {
        pub permission: String,
        pub pattern: String,
        pub action: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ConfigValue {
        Action(String),
        Rules(Vec<(String, String)>),
    }

    pub type Ruleset<'a> = &'a [Rule];

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PermissionError {
        NotImplemented(&'static str),
    }

    impl std::fmt::Display for PermissionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                PermissionError::NotImplemented(what) => {
                    write!(f, "porting: {what} not implemented")
                }
            }
        }
    }

    pub fn from_config(_entries: &[(&str, ConfigValue)]) -> Result<Vec<Rule>, PermissionError> {
        Err(PermissionError::NotImplemented("permission::fromConfig"))
    }

    pub fn evaluate(
        _permission: &str,
        _pattern: &str,
        _rulesets: &[Ruleset<'_>],
    ) -> Result<Rule, PermissionError> {
        Err(PermissionError::NotImplemented("permission::evaluate"))
    }

    pub fn merge(_rulesets: &[Ruleset<'_>]) -> Result<Vec<Rule>, PermissionError> {
        Err(PermissionError::NotImplemented("permission::merge"))
    }

    pub fn disabled(
        _tools: &[&str],
        _ruleset: Ruleset<'_>,
    ) -> Result<BTreeSet<String>, PermissionError> {
        Err(PermissionError::NotImplemented("permission::disabled"))
    }
}

use permission::{ConfigValue, Rule};

fn rule(permission: &str, pattern: &str, action: &str) -> Rule {
    Rule {
        permission: permission.to_string(),
        pattern: pattern.to_string(),
        action: action.to_string(),
    }
}

fn action(value: &str) -> ConfigValue {
    ConfigValue::Action(value.to_string())
}

fn rules(pairs: &[(&str, &str)]) -> ConfigValue {
    ConfigValue::Rules(
        pairs
            .iter()
            .map(|(pattern, action)| (pattern.to_string(), action.to_string()))
            .collect(),
    )
}

fn home() -> String {
    std::env::var("HOME").expect("HOME")
}

// fromConfig tests

#[test]
#[ignore = "porting: permission-next not implemented"]
fn from_config_string_value_becomes_wildcard_rule() {
    let result = permission::from_config(&[("bash", action("allow"))]).unwrap();
    assert_eq!(result, vec![rule("bash", "*", "allow")]);
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn from_config_object_value_converts_to_rules_array() {
    let result =
        permission::from_config(&[("bash", rules(&[("*", "allow"), ("rm", "deny")]))]).unwrap();
    assert_eq!(
        result,
        vec![rule("bash", "*", "allow"), rule("bash", "rm", "deny")]
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn from_config_mixed_string_and_object_values() {
    let result = permission::from_config(&[
        ("bash", rules(&[("*", "allow"), ("rm", "deny")])),
        ("edit", action("allow")),
        ("webfetch", action("ask")),
    ])
    .unwrap();
    assert_eq!(
        result,
        vec![
            rule("bash", "*", "allow"),
            rule("bash", "rm", "deny"),
            rule("edit", "*", "allow"),
            rule("webfetch", "*", "ask"),
        ]
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn from_config_empty_object() {
    let result = permission::from_config(&[]).unwrap();
    assert!(result.is_empty());
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn from_config_expands_tilde_to_home_directory() {
    let result =
        permission::from_config(&[("external_directory", rules(&[("~/projects/*", "allow")]))])
            .unwrap();
    assert_eq!(
        result,
        vec![rule(
            "external_directory",
            &format!("{}/projects/*", home()),
            "allow"
        )]
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn from_config_expands_dollar_home_to_home_directory() {
    let result = permission::from_config(&[(
        "external_directory",
        rules(&[("$HOME/projects/*", "allow")]),
    )])
    .unwrap();
    assert_eq!(
        result,
        vec![rule(
            "external_directory",
            &format!("{}/projects/*", home()),
            "allow"
        )]
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn from_config_expands_dollar_home_without_trailing_slash() {
    let result =
        permission::from_config(&[("external_directory", rules(&[("$HOME", "allow")]))]).unwrap();
    assert_eq!(result, vec![rule("external_directory", &home(), "allow")]);
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn from_config_does_not_expand_tilde_in_middle_of_path() {
    let result =
        permission::from_config(&[("external_directory", rules(&[("/some/~/path", "allow")]))])
            .unwrap();
    assert_eq!(
        result,
        vec![rule("external_directory", "/some/~/path", "allow")]
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn from_config_preserves_top_level_config_key_order() {
    let wildcard_first =
        permission::from_config(&[("*", action("deny")), ("bash", action("allow"))]).unwrap();
    let specific_first =
        permission::from_config(&[("bash", action("allow")), ("*", action("deny"))]).unwrap();

    assert_eq!(
        wildcard_first
            .iter()
            .map(|r| r.permission.as_str())
            .collect::<Vec<_>>(),
        vec!["*", "bash"]
    );
    assert_eq!(
        specific_first
            .iter()
            .map(|r| r.permission.as_str())
            .collect::<Vec<_>>(),
        vec!["bash", "*"]
    );

    assert_eq!(
        permission::evaluate("bash", "ls", &[wildcard_first.as_slice()])
            .unwrap()
            .action,
        "allow"
    );
    assert_eq!(
        permission::evaluate("bash", "ls", &[specific_first.as_slice()])
            .unwrap()
            .action,
        "deny"
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn from_config_wildcard_acts_as_fallback_when_it_appears_before_specifics() {
    let ruleset =
        permission::from_config(&[("*", action("ask")), ("bash", action("allow"))]).unwrap();
    assert_eq!(
        permission::evaluate("edit", "foo.ts", &[ruleset.as_slice()])
            .unwrap()
            .action,
        "ask"
    );
    assert_eq!(
        permission::evaluate("bash", "ls", &[ruleset.as_slice()])
            .unwrap()
            .action,
        "allow"
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn from_config_top_level_ordering_is_not_sorted_by_wildcard_specificity() {
    let ruleset = permission::from_config(&[
        ("bash", action("allow")),
        ("*", action("ask")),
        ("edit", action("deny")),
        ("mcp_*", action("allow")),
    ])
    .unwrap();
    assert_eq!(
        ruleset
            .iter()
            .map(|r| r.permission.as_str())
            .collect::<Vec<_>>(),
        vec!["bash", "*", "edit", "mcp_*"]
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn from_config_sub_pattern_insertion_order_inside_a_tool_key_is_preserved() {
    let ruleset =
        permission::from_config(&[("bash", rules(&[("*", "deny"), ("git *", "allow")]))]).unwrap();
    assert_eq!(
        ruleset
            .iter()
            .map(|r| r.pattern.as_str())
            .collect::<Vec<_>>(),
        vec!["*", "git *"]
    );
    assert_eq!(
        permission::evaluate("bash", "rm foo", &[ruleset.as_slice()])
            .unwrap()
            .action,
        "deny"
    );
    assert_eq!(
        permission::evaluate("bash", "git status", &[ruleset.as_slice()])
            .unwrap()
            .action,
        "allow"
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn from_config_documented_fallback_first_example() {
    let ruleset = permission::from_config(&[
        ("*", action("ask")),
        ("bash", action("allow")),
        ("edit", action("deny")),
    ])
    .unwrap();
    assert_eq!(
        permission::evaluate("bash", "ls", &[ruleset.as_slice()])
            .unwrap()
            .action,
        "allow"
    );
    assert_eq!(
        permission::evaluate("edit", "foo.ts", &[ruleset.as_slice()])
            .unwrap()
            .action,
        "deny"
    );
    assert_eq!(
        permission::evaluate("read", "foo.ts", &[ruleset.as_slice()])
            .unwrap()
            .action,
        "ask"
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn from_config_expands_exact_tilde_to_home_directory() {
    let result =
        permission::from_config(&[("external_directory", rules(&[("~", "allow")]))]).unwrap();
    assert_eq!(result, vec![rule("external_directory", &home(), "allow")]);
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_matches_expanded_tilde_pattern() {
    let ruleset =
        permission::from_config(&[("external_directory", rules(&[("~/projects/*", "allow")]))])
            .unwrap();
    let result = permission::evaluate(
        "external_directory",
        &format!("{}/projects/file.txt", home()),
        &[ruleset.as_slice()],
    )
    .unwrap();
    assert_eq!(result.action, "allow");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_matches_expanded_dollar_home_pattern() {
    let ruleset = permission::from_config(&[(
        "external_directory",
        rules(&[("$HOME/projects/*", "allow")]),
    )])
    .unwrap();
    let result = permission::evaluate(
        "external_directory",
        &format!("{}/projects/file.txt", home()),
        &[ruleset.as_slice()],
    )
    .unwrap();
    assert_eq!(result.action, "allow");
}

// merge tests

#[test]
#[ignore = "porting: permission-next not implemented"]
fn merge_simple_concatenation() {
    let a = vec![rule("bash", "*", "allow")];
    let b = vec![rule("bash", "*", "deny")];
    assert_eq!(
        permission::merge(&[a.as_slice(), b.as_slice()]).unwrap(),
        vec![rule("bash", "*", "allow"), rule("bash", "*", "deny")]
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn merge_adds_new_permission() {
    let a = vec![rule("bash", "*", "allow")];
    let b = vec![rule("edit", "*", "deny")];
    assert_eq!(
        permission::merge(&[a.as_slice(), b.as_slice()]).unwrap(),
        vec![rule("bash", "*", "allow"), rule("edit", "*", "deny")]
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn merge_concatenates_rules_for_same_permission() {
    let a = vec![rule("bash", "foo", "ask")];
    let b = vec![rule("bash", "*", "deny")];
    assert_eq!(
        permission::merge(&[a.as_slice(), b.as_slice()]).unwrap(),
        vec![rule("bash", "foo", "ask"), rule("bash", "*", "deny")]
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn merge_multiple_rulesets() {
    let a = vec![rule("bash", "*", "allow")];
    let b = vec![rule("bash", "rm", "ask")];
    let c = vec![rule("edit", "*", "allow")];
    assert_eq!(
        permission::merge(&[a.as_slice(), b.as_slice(), c.as_slice()]).unwrap(),
        vec![
            rule("bash", "*", "allow"),
            rule("bash", "rm", "ask"),
            rule("edit", "*", "allow"),
        ]
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn merge_empty_ruleset_does_nothing() {
    let a = vec![rule("bash", "*", "allow")];
    let empty: &[Rule] = &[];
    assert_eq!(
        permission::merge(&[a.as_slice(), empty]).unwrap(),
        vec![rule("bash", "*", "allow")]
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn merge_preserves_rule_order() {
    let a = vec![
        rule("edit", "src/*", "allow"),
        rule("edit", "src/secret/*", "deny"),
    ];
    let b = vec![rule("edit", "src/secret/ok.ts", "allow")];
    assert_eq!(
        permission::merge(&[a.as_slice(), b.as_slice()]).unwrap(),
        vec![
            rule("edit", "src/*", "allow"),
            rule("edit", "src/secret/*", "deny"),
            rule("edit", "src/secret/ok.ts", "allow"),
        ]
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn merge_config_permission_overrides_default_ask() {
    let defaults = vec![rule("*", "*", "ask")];
    let config = vec![rule("bash", "*", "allow")];
    let merged = permission::merge(&[defaults.as_slice(), config.as_slice()]).unwrap();

    assert_eq!(
        permission::evaluate("bash", "ls", &[merged.as_slice()])
            .unwrap()
            .action,
        "allow"
    );
    assert_eq!(
        permission::evaluate("edit", "foo.ts", &[merged.as_slice()])
            .unwrap()
            .action,
        "ask"
    );
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn merge_config_ask_overrides_default_allow() {
    let defaults = vec![rule("bash", "*", "allow")];
    let config = vec![rule("bash", "*", "ask")];
    let merged = permission::merge(&[defaults.as_slice(), config.as_slice()]).unwrap();

    assert_eq!(
        permission::evaluate("bash", "ls", &[merged.as_slice()])
            .unwrap()
            .action,
        "ask"
    );
}

// evaluate tests

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_exact_pattern_match() {
    let result = permission::evaluate("bash", "rm", &[&[rule("bash", "rm", "deny")]]).unwrap();
    assert_eq!(result.action, "deny");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_wildcard_pattern_match() {
    let result = permission::evaluate("bash", "rm", &[&[rule("bash", "*", "allow")]]).unwrap();
    assert_eq!(result.action, "allow");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_last_matching_rule_wins() {
    let result = permission::evaluate(
        "bash",
        "rm",
        &[&[rule("bash", "*", "allow"), rule("bash", "rm", "deny")]],
    )
    .unwrap();
    assert_eq!(result.action, "deny");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_last_matching_rule_wins_wildcard_after_specific() {
    let result = permission::evaluate(
        "bash",
        "rm",
        &[&[rule("bash", "rm", "deny"), rule("bash", "*", "allow")]],
    )
    .unwrap();
    assert_eq!(result.action, "allow");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_glob_pattern_match() {
    let result =
        permission::evaluate("edit", "src/foo.ts", &[&[rule("edit", "src/*", "allow")]]).unwrap();
    assert_eq!(result.action, "allow");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_last_matching_glob_wins() {
    let result = permission::evaluate(
        "edit",
        "src/components/Button.tsx",
        &[&[
            rule("edit", "src/*", "deny"),
            rule("edit", "src/components/*", "allow"),
        ]],
    )
    .unwrap();
    assert_eq!(result.action, "allow");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_order_matters_for_specificity() {
    let result = permission::evaluate(
        "edit",
        "src/components/Button.tsx",
        &[&[
            rule("edit", "src/components/*", "allow"),
            rule("edit", "src/*", "deny"),
        ]],
    )
    .unwrap();
    assert_eq!(result.action, "deny");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_unknown_permission_returns_ask() {
    let result =
        permission::evaluate("unknown_tool", "anything", &[&[rule("bash", "*", "allow")]]).unwrap();
    assert_eq!(result.action, "ask");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_empty_ruleset_returns_ask() {
    let empty: &[permission::Ruleset<'_>] = &[];
    let result = permission::evaluate("bash", "rm", empty).unwrap();
    assert_eq!(result.action, "ask");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_no_matching_pattern_returns_ask() {
    let result =
        permission::evaluate("edit", "etc/passwd", &[&[rule("edit", "src/*", "allow")]]).unwrap();
    assert_eq!(result.action, "ask");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_empty_rules_array_returns_ask() {
    let empty_rules: &[Rule] = &[];
    let result = permission::evaluate("bash", "rm", &[empty_rules]).unwrap();
    assert_eq!(result.action, "ask");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_multiple_matching_patterns_last_wins() {
    let result = permission::evaluate(
        "edit",
        "src/secret.ts",
        &[&[
            rule("edit", "*", "ask"),
            rule("edit", "src/*", "allow"),
            rule("edit", "src/secret.ts", "deny"),
        ]],
    )
    .unwrap();
    assert_eq!(result.action, "deny");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_non_matching_patterns_are_skipped() {
    let result = permission::evaluate(
        "edit",
        "src/foo.ts",
        &[&[
            rule("edit", "*", "ask"),
            rule("edit", "test/*", "deny"),
            rule("edit", "src/*", "allow"),
        ]],
    )
    .unwrap();
    assert_eq!(result.action, "allow");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_exact_match_at_end_wins_over_earlier_wildcard() {
    let result = permission::evaluate(
        "bash",
        "/bin/rm",
        &[&[rule("bash", "*", "allow"), rule("bash", "/bin/rm", "deny")]],
    )
    .unwrap();
    assert_eq!(result.action, "deny");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_wildcard_at_end_overrides_earlier_exact_match() {
    let result = permission::evaluate(
        "bash",
        "/bin/rm",
        &[&[rule("bash", "/bin/rm", "deny"), rule("bash", "*", "allow")]],
    )
    .unwrap();
    assert_eq!(result.action, "allow");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_wildcard_permission_matches_any_permission() {
    let result = permission::evaluate("bash", "rm", &[&[rule("*", "*", "deny")]]).unwrap();
    assert_eq!(result.action, "deny");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_wildcard_permission_with_specific_pattern() {
    let result = permission::evaluate("bash", "rm", &[&[rule("*", "rm", "deny")]]).unwrap();
    assert_eq!(result.action, "deny");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_glob_permission_pattern() {
    let result = permission::evaluate(
        "mcp_server_tool",
        "anything",
        &[&[rule("mcp_*", "*", "allow")]],
    )
    .unwrap();
    assert_eq!(result.action, "allow");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_specific_permission_and_wildcard_permission_combined() {
    let result = permission::evaluate(
        "bash",
        "rm",
        &[&[rule("*", "*", "deny"), rule("bash", "*", "allow")]],
    )
    .unwrap();
    assert_eq!(result.action, "allow");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_wildcard_permission_does_not_match_when_specific_exists() {
    let result = permission::evaluate(
        "edit",
        "src/foo.ts",
        &[&[rule("*", "*", "deny"), rule("edit", "src/*", "allow")]],
    )
    .unwrap();
    assert_eq!(result.action, "allow");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_multiple_matching_permission_patterns_combine_rules() {
    let result = permission::evaluate(
        "mcp_dangerous",
        "anything",
        &[&[
            rule("*", "*", "ask"),
            rule("mcp_*", "*", "allow"),
            rule("mcp_dangerous", "*", "deny"),
        ]],
    )
    .unwrap();
    assert_eq!(result.action, "deny");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_wildcard_permission_fallback_for_unknown_tool() {
    let result = permission::evaluate(
        "unknown_tool",
        "anything",
        &[&[rule("*", "*", "ask"), rule("bash", "*", "allow")]],
    )
    .unwrap();
    assert_eq!(result.action, "ask");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_later_wildcard_permission_can_override_earlier_specific_permission() {
    let result = permission::evaluate(
        "bash",
        "rm",
        &[&[rule("bash", "*", "allow"), rule("*", "*", "deny")]],
    )
    .unwrap();
    assert_eq!(result.action, "deny");
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn evaluate_merges_multiple_rulesets() {
    let config = vec![rule("bash", "*", "allow")];
    let approved = vec![rule("bash", "rm", "deny")];
    let result =
        permission::evaluate("bash", "rm", &[config.as_slice(), approved.as_slice()]).unwrap();
    assert_eq!(result.action, "deny");
}

// disabled tests

#[test]
#[ignore = "porting: permission-next not implemented"]
fn disabled_returns_empty_set_when_all_tools_allowed() {
    let result =
        permission::disabled(&["bash", "edit", "read"], &[rule("*", "*", "allow")]).unwrap();
    assert!(result.is_empty());
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn disabled_disables_tool_when_denied() {
    let result = permission::disabled(
        &["bash", "edit", "read"],
        &[rule("*", "*", "allow"), rule("bash", "*", "deny")],
    )
    .unwrap();
    assert!(result.contains("bash"));
    assert!(!result.contains("edit"));
    assert!(!result.contains("read"));
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn disabled_disables_edit_write_apply_patch_when_edit_denied() {
    let result = permission::disabled(
        &["edit", "write", "apply_patch", "bash"],
        &[rule("*", "*", "allow"), rule("edit", "*", "deny")],
    )
    .unwrap();
    assert!(result.contains("edit"));
    assert!(result.contains("write"));
    assert!(result.contains("apply_patch"));
    assert!(!result.contains("bash"));
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn disabled_does_not_disable_when_partially_denied() {
    let result = permission::disabled(
        &["bash"],
        &[rule("bash", "*", "allow"), rule("bash", "rm *", "deny")],
    )
    .unwrap();
    assert!(!result.contains("bash"));
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn disabled_does_not_disable_when_action_is_ask() {
    let result = permission::disabled(&["bash", "edit"], &[rule("*", "*", "ask")]).unwrap();
    assert!(result.is_empty());
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn disabled_does_not_disable_when_specific_allow_after_wildcard_deny() {
    let result = permission::disabled(
        &["bash"],
        &[rule("bash", "*", "deny"), rule("bash", "echo *", "allow")],
    )
    .unwrap();
    assert!(!result.contains("bash"));
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn disabled_does_not_disable_when_wildcard_allow_after_deny() {
    let result = permission::disabled(
        &["bash"],
        &[rule("bash", "rm *", "deny"), rule("bash", "*", "allow")],
    )
    .unwrap();
    assert!(!result.contains("bash"));
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn disabled_disables_multiple_tools() {
    let result = permission::disabled(
        &["bash", "edit", "webfetch"],
        &[
            rule("bash", "*", "deny"),
            rule("edit", "*", "deny"),
            rule("webfetch", "*", "deny"),
        ],
    )
    .unwrap();
    assert!(result.contains("bash"));
    assert!(result.contains("edit"));
    assert!(result.contains("webfetch"));
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn disabled_wildcard_permission_denies_all_tools() {
    let result =
        permission::disabled(&["bash", "edit", "read"], &[rule("*", "*", "deny")]).unwrap();
    assert!(result.contains("bash"));
    assert!(result.contains("edit"));
    assert!(result.contains("read"));
}

#[test]
#[ignore = "porting: permission-next not implemented"]
fn disabled_specific_allow_overrides_wildcard_deny() {
    let result = permission::disabled(
        &["bash", "edit", "read"],
        &[rule("*", "*", "deny"), rule("bash", "*", "allow")],
    )
    .unwrap();
    assert!(!result.contains("bash"));
    assert!(result.contains("edit"));
    assert!(result.contains("read"));
}
