//! Port of packages/opencode/test/config/lsp.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the `ConfigLSPV1.Info` refinement; see docs/TEST-PORT.md.
//!
//! Re-derived against `opencode_server::lsp_config::validate_lsp_config`, which
//! takes the decoded JSON value rather than an Effect schema.

use opencode_server::lsp_config::validate_lsp_config;
use serde_json::json;

const EXPECTED: &str = "For custom LSP servers, 'extensions' array is required.";

#[test]
fn accepts_true_and_false_top_level_toggle() {
    assert!(validate_lsp_config(&json!(true)).is_ok());
    assert!(validate_lsp_config(&json!(false)).is_ok());
}

#[test]
fn accepts_builtin_server_without_extensions() {
    let input = json!({ "typescript": { "command": ["typescript-language-server", "--stdio"] } });
    assert!(validate_lsp_config(&input).is_ok());
}

#[test]
fn accepts_custom_server_with_extensions() {
    let input = json!({ "my-lsp": { "command": ["my-lsp-bin"], "extensions": [".ml"] } });
    assert!(validate_lsp_config(&input).is_ok());
}

#[test]
fn accepts_disabled_custom_server_without_extensions() {
    let input = json!({ "my-lsp": { "disabled": true } });
    assert!(validate_lsp_config(&input).is_ok());
}

#[test]
fn accepts_mix_of_builtin_and_custom_with_extensions() {
    let input = json!({
        "typescript": { "command": ["typescript-language-server", "--stdio"] },
        "my-lsp": { "command": ["my-lsp-bin"], "extensions": [".ml"] },
    });
    assert!(validate_lsp_config(&input).is_ok());
}

#[test]
fn accepts_custom_server_with_empty_extensions_array() {
    let input = json!({ "my-lsp": { "command": ["my-lsp-bin"], "extensions": [] } });
    assert!(validate_lsp_config(&input).is_ok());
}

#[test]
fn rejects_custom_server_without_extensions() {
    let input = json!({ "my-lsp": { "command": ["my-lsp-bin"] } });
    assert_eq!(validate_lsp_config(&input).unwrap_err(), EXPECTED);
}

#[test]
fn rejects_custom_server_without_extensions_mixed_with_valid_builtin() {
    let input = json!({
        "typescript": { "command": ["typescript-language-server", "--stdio"] },
        "my-lsp": { "command": ["my-lsp-bin"] },
    });
    assert_eq!(validate_lsp_config(&input).unwrap_err(), EXPECTED);
}
