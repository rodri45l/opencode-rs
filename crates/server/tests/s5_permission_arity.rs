//! Port of packages/opencode/test/permission/arity.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `BashArity.prefix` returns the longest known command
//! prefix (1-3 tokens) and falls back to the first token for unknown commands.
#![allow(dead_code)]

// Fast-wave local stubs: `permission::arity` is not implemented in this crate yet.
mod arity {
    pub fn prefix(_tokens: &[&str]) -> Result<Vec<String>, &'static str> {
        Err("porting: BashArity::prefix not implemented")
    }
}

fn s(tokens: &[&str]) -> Vec<String> {
    tokens.iter().map(|token| token.to_string()).collect()
}

#[test]
#[ignore = "porting: permission-arity not implemented"]
fn arity_1_unknown_commands_default_to_first_token() {
    assert_eq!(
        arity::prefix(&["unknown", "command", "subcommand"]).unwrap(),
        s(&["unknown"])
    );
    assert_eq!(arity::prefix(&["touch", "foo.txt"]).unwrap(), s(&["touch"]));
}

#[test]
#[ignore = "porting: permission-arity not implemented"]
fn arity_2_two_token_commands() {
    assert_eq!(
        arity::prefix(&["git", "checkout", "main"]).unwrap(),
        s(&["git", "checkout"])
    );
    assert_eq!(
        arity::prefix(&["docker", "run", "nginx"]).unwrap(),
        s(&["docker", "run"])
    );
}

#[test]
#[ignore = "porting: permission-arity not implemented"]
fn arity_3_three_token_commands() {
    assert_eq!(
        arity::prefix(&["aws", "s3", "ls", "my-bucket"]).unwrap(),
        s(&["aws", "s3", "ls"])
    );
    assert_eq!(
        arity::prefix(&["npm", "run", "dev", "script"]).unwrap(),
        s(&["npm", "run", "dev"])
    );
}

#[test]
#[ignore = "porting: permission-arity not implemented"]
fn longest_match_wins_nested_prefixes() {
    assert_eq!(
        arity::prefix(&["docker", "compose", "up", "service"]).unwrap(),
        s(&["docker", "compose", "up"])
    );
    assert_eq!(
        arity::prefix(&["consul", "kv", "get", "config"]).unwrap(),
        s(&["consul", "kv", "get"])
    );
}

#[test]
#[ignore = "porting: permission-arity not implemented"]
fn exact_length_matches() {
    assert_eq!(
        arity::prefix(&["git", "checkout"]).unwrap(),
        s(&["git", "checkout"])
    );
    assert_eq!(
        arity::prefix(&["npm", "run", "dev"]).unwrap(),
        s(&["npm", "run", "dev"])
    );
}

#[test]
#[ignore = "porting: permission-arity not implemented"]
fn edge_cases() {
    assert_eq!(arity::prefix(&[]).unwrap(), s(&[]));
    assert_eq!(arity::prefix(&["single"]).unwrap(), s(&["single"]));
    assert_eq!(arity::prefix(&["git"]).unwrap(), s(&["git"]));
}
