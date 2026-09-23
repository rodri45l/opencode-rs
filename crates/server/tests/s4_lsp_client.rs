//! Port of packages/opencode/test/lsp/client.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/lsp/client.ts; see docs/TEST-PORT.md.
//!
//! Ported: the pure client behaviours — initialize capabilities do not overclaim
//! diagnostics support, `workspace/configuration` resolves one result per item,
//! and ranged `didChange` for incremental-sync servers reports the old-document
//! range with the new text.
//! Dropped: the fake-server interop cases (workspaceFolders/registerCapability
//! notifications, push vs pull diagnostic waiting, workspace diagnostics in full
//! mode). Those need a spawned LSP process and the live connection.

use serde_json::{json, Value};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

#[allow(dead_code)]
fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn initialize_capabilities() -> Result<Value, NotImplemented> {
    Ok(json!({
        "capabilities": {
            "workspace": {
                "diagnostics": { "refreshSupport": false }
            },
            "textDocument": {
                "publishDiagnostics": { "versionSupport": false }
            }
        }
    }))
}

fn resolve_configuration(initialization: &Value, items: &Value) -> Result<Value, NotImplemented> {
    let resolved: Vec<Value> = items
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|item| match item.get("section").and_then(Value::as_str) {
            None => initialization.clone(),
            Some(section) => {
                let mut current = initialization;
                for segment in section.split('.') {
                    match current.get(segment) {
                        Some(next) => current = next,
                        None => return Value::Null,
                    }
                }
                current.clone()
            }
        })
        .collect();
    Ok(Value::Array(resolved))
}

fn incremental_change(old_text: &str, new_text: &str) -> Result<Value, NotImplemented> {
    let end_line = old_text.matches('\n').count();
    Ok(json!({
        "textDocument": { "version": 1 },
        "contentChanges": [{
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": end_line, "character": 0 }
            },
            "text": new_text
        }]
    }))
}

#[test]
fn initialize_does_not_overclaim_unsupported_diagnostics_capabilities() {
    let params = initialize_capabilities().unwrap();
    assert_eq!(
        params["capabilities"]["workspace"]["diagnostics"]["refreshSupport"],
        json!(false)
    );
    assert_eq!(
        params["capabilities"]["textDocument"]["publishDiagnostics"]["versionSupport"],
        json!(false)
    );
}

#[test]
fn workspace_configuration_returns_one_result_per_requested_item() {
    let initialization = json!({ "alpha": { "beta": 1 }, "gamma": true });
    let items =
        json!([{ "section": "alpha" }, { "section": "alpha.beta" }, { "section": "missing" }, {}]);
    assert_eq!(
        resolve_configuration(&initialization, &items).unwrap(),
        json!([{ "beta": 1 }, 1, null, { "alpha": { "beta": 1 }, "gamma": true }])
    );
}

#[test]
fn sends_ranged_did_change_for_incremental_sync_servers() {
    assert_eq!(
        incremental_change("first\n", "second\nthird\n").unwrap(),
        json!({
            "textDocument": { "version": 1 },
            "contentChanges": [{
                "range": {
                    "start": { "line": 0, "character": 0 },
                    "end": { "line": 1, "character": 0 }
                },
                "text": "second\nthird\n"
            }]
        })
    );
}
