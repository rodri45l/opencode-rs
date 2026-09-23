//! Port of packages/opencode/test/lsp/lifecycle.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/lsp/lsp.ts; see docs/TEST-PORT.md.
//!
//! Ported: `LSP.Diagnostic.pretty` (severity label, 1-based line/character).
//! Dropped: the `LSP.Service` lifecycle cases (init/status/diagnostics/hasClients
//! /workspaceSymbol/definition/references, idempotent init) — they need the
//! Effect Layer, spawned LSP servers, and instance context.

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn pretty(_diagnostic: &Value) -> Result<String, NotImplemented> {
    nope("lsp lifecycle")
}

#[test]
#[ignore = "porting: lsp lifecycle not implemented"]
fn pretty_formats_error_diagnostic() {
    assert_eq!(
        pretty(&json!({
            "range": { "start": { "line": 9, "character": 4 }, "end": { "line": 9, "character": 10 } },
            "message": "Type 'string' is not assignable to type 'number'",
            "severity": 1
        }))
        .unwrap(),
        "ERROR [10:5] Type 'string' is not assignable to type 'number'"
    );
}

#[test]
#[ignore = "porting: lsp lifecycle not implemented"]
fn pretty_formats_warning_diagnostic() {
    assert_eq!(
        pretty(&json!({
            "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 5 } },
            "message": "Unused variable",
            "severity": 2
        }))
        .unwrap(),
        "WARN [1:1] Unused variable"
    );
}

#[test]
#[ignore = "porting: lsp lifecycle not implemented"]
fn pretty_defaults_to_error_when_no_severity() {
    assert_eq!(
        pretty(&json!({
            "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 1 } },
            "message": "Something wrong"
        }))
        .unwrap(),
        "ERROR [1:1] Something wrong"
    );
}
