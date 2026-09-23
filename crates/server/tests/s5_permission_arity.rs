//! Port of packages/opencode/test/permission/arity.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `BashArity.prefix` returns the longest known command
//! prefix (1-3 tokens) and falls back to the first token for unknown commands.
#![allow(dead_code)]

use opencode_server::permission::bash_arity_prefix as prefix;

fn s(tokens: &[&str]) -> Vec<String> {
    tokens.iter().map(|token| token.to_string()).collect()
}

#[test]
fn arity_1_unknown_commands_default_to_first_token() {
    assert_eq!(
        prefix(&["unknown", "command", "subcommand"]),
        s(&["unknown"])
    );
    assert_eq!(prefix(&["touch", "foo.txt"]), s(&["touch"]));
}

#[test]
fn arity_2_two_token_commands() {
    assert_eq!(
        prefix(&["git", "checkout", "main"]),
        s(&["git", "checkout"])
    );
    assert_eq!(prefix(&["docker", "run", "nginx"]), s(&["docker", "run"]));
}

#[test]
fn arity_3_three_token_commands() {
    assert_eq!(
        prefix(&["aws", "s3", "ls", "my-bucket"]),
        s(&["aws", "s3", "ls"])
    );
    assert_eq!(
        prefix(&["npm", "run", "dev", "script"]),
        s(&["npm", "run", "dev"])
    );
}

#[test]
fn longest_match_wins_nested_prefixes() {
    assert_eq!(
        prefix(&["docker", "compose", "up", "service"]),
        s(&["docker", "compose", "up"])
    );
    assert_eq!(
        prefix(&["consul", "kv", "get", "config"]),
        s(&["consul", "kv", "get"])
    );
}

#[test]
fn exact_length_matches() {
    assert_eq!(prefix(&["git", "checkout"]), s(&["git", "checkout"]));
    assert_eq!(prefix(&["npm", "run", "dev"]), s(&["npm", "run", "dev"]));
}

#[test]
fn edge_cases() {
    assert_eq!(prefix(&[]), s(&[]));
    assert_eq!(prefix(&["single"]), s(&["single"]));
    assert_eq!(prefix(&["git"]), s(&["git"]));
}
