//! Port of packages/opencode/test/lsp/index.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/lsp/lsp.ts and src/lsp/server.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure builtin-LSP selection decision — files outside the instance
//! never spawn; `.ts` spawns TypeScript only when `lsp` is enabled; a config
//! object keeps builtin LSPs; `.py` spawns Pyright, or Ty when the experimental
//! flag is on; and `hasClients` mirrors that decision.
//! Dropped: cases that spawn/spy real servers or await the `lsp.updated` event
//! through the Effect/EventV2Bridge runtime, and the disableLspDownload pass-through
//! assertion.

use serde_json::{json, Value};

use opencode_server::lsp_index::{builtin_server, has_clients};

#[test]
fn does_not_spawn_builtin_lsp_for_files_outside_instance() {
    assert_eq!(
        builtin_server("/workspace/../outside.ts", false, &json!(true), false).unwrap(),
        None
    );
}

#[test]
fn does_not_spawn_builtin_lsp_for_files_inside_instance_when_lsp_is_unset() {
    assert_eq!(
        builtin_server("/workspace/src/inside.ts", true, &Value::Null, false).unwrap(),
        None
    );
}

#[test]
fn would_spawn_builtin_lsp_for_ts_when_lsp_is_true() {
    assert_eq!(
        builtin_server("/workspace/src/inside.ts", true, &json!(true), false).unwrap(),
        Some("typescript".to_string())
    );
}

#[test]
fn keeps_builtin_lsps_when_config_object_is_provided() {
    assert_eq!(
        builtin_server(
            "/workspace/src/inside.ts",
            true,
            &json!({ "eslint": { "disabled": true } }),
            false
        )
        .unwrap(),
        Some("typescript".to_string())
    );
}

#[test]
fn uses_pyright_instead_of_ty_by_default() {
    assert_eq!(
        builtin_server("/workspace/src/inside.py", true, &json!(true), false).unwrap(),
        Some("pyright".to_string())
    );
}

#[test]
fn uses_ty_instead_of_pyright_when_experimental_lsp_ty_is_enabled() {
    assert_eq!(
        builtin_server("/workspace/src/inside.py", true, &json!(true), true).unwrap(),
        Some("ty".to_string())
    );
}

#[test]
fn has_clients_matches_the_builtin_selection() {
    assert!(!has_clients("/workspace/src/inside.ts", true, &Value::Null).unwrap());
    assert!(has_clients("/workspace/src/inside.ts", true, &json!(true)).unwrap());
    assert!(has_clients(
        "/workspace/src/inside.ts",
        true,
        &json!({ "eslint": { "disabled": true } })
    )
    .unwrap());
    assert!(!has_clients("/workspace/../outside.ts", false, &json!(true)).unwrap());
}
