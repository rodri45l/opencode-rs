//! Port of packages/codemode/test/enumeration.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/codemode/src/index.ts: `Object.keys` and `for...in`
//! share one enumeration surface over plain objects, arrays, and the host tool tree,
//! and unsupported values fail with a hint at the working idioms.
//! Re-derived: the interpreter API takes only source (host tools are not injectable
//! yet), so the programs run through `CodeMode::execute`; the tool tree in each
//! program is illustrative of the upstream fixture.
//! Red-first: the restricted JavaScript interpreter is not implemented.

use opencode_codemode::{CodeMode, CodeModeError, CodeModeErrorKind, CodeModeResult};
use serde_json::{json, Value};

const NOTE: &str = "porting: code-mode interpreter not implemented";

fn run(code: &str) -> CodeModeResult {
    CodeMode::execute(code).expect(NOTE)
}

fn value(code: &str) -> Value {
    let result = run(code);
    assert!(result.ok, "expected success, got {:?}", result.error);
    result.value.expect("successful result carries a value")
}

fn error(code: &str) -> CodeModeError {
    let result = run(code);
    assert!(!result.ok, "expected failure, got {:?}", result.value);
    result.error.expect("failed result carries an error")
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn enumerates_top_level_namespaces() {
    assert_eq!(
        value(
            r#"
            const namespaces = Object.keys(tools)
            return { namespaces, count: namespaces.length }
        "#
        ),
        json!({ "namespaces": ["github", "memory", "playwright", "$codemode"], "count": 4 })
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn enumerates_tool_names_at_a_nested_namespace() {
    assert_eq!(
        value("return Object.keys(tools.github)"),
        json!(["list_issues", "get_issue"])
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn a_callable_tool_is_a_leaf_and_enumerates_as_empty() {
    assert_eq!(
        value("return Object.keys(tools.github.list_issues)"),
        json!([])
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn the_internal_discovery_namespace_enumerates_its_callable_surface() {
    assert_eq!(
        value("return Object.keys(tools.$codemode)"),
        json!(["search"])
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn an_unknown_namespace_is_an_unknown_tool_error_pointing_at_the_discovery_idioms() {
    let failure = error("return Object.keys(tools.nonexistent)");
    assert_eq!(failure.kind, CodeModeErrorKind::UnknownTool);
    assert!(failure
        .message
        .contains("Unknown tool namespace 'nonexistent'"));
    assert!(failure.message.contains("Object.keys(tools)"));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn object_values_and_entries_on_a_tool_reference_explain_the_working_idioms() {
    for method in ["values", "entries"] {
        let failure = error(&format!("return Object.{method}(tools)"));
        assert_eq!(failure.kind, CodeModeErrorKind::InvalidDataValue);
        assert!(failure
            .message
            .contains(&format!("Object.{method}(...) cannot read tool references")));
    }
    let nested = error("return Object.entries(tools.github)");
    assert!(nested.message.contains("Use Object.keys(tools) for names"));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn object_keys_over_arrays_returns_index_strings() {
    assert_eq!(
        value(r#"return Object.keys(["a", "b", "c"])"#),
        json!(["0", "1", "2"])
    );
    assert_eq!(value("return Object.keys([])"), json!([]));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn objects_keep_their_own_enumerable_keys() {
    assert_eq!(
        value("return Object.keys({ a: 1, b: 2 })"),
        json!(["a", "b"])
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn non_object_inputs_still_fail_clearly() {
    let failure = error(r#"return Object.keys("nope")"#);
    assert!(failure
        .message
        .contains("Object.keys expects a data object or array"));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn for_in_iterates_own_enumerable_keys_with_break_and_continue() {
    assert_eq!(
        value(
            r#"
            const seen = []
            for (const key in { a: 1, b: 2, c: 3, d: 4 }) {
              if (key === "b") continue
              if (key === "d") break
              seen.push(key)
            }
            return seen
        "#
        ),
        json!(["a", "c"])
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn for_in_iterates_index_strings_over_arrays() {
    assert_eq!(
        value(
            r#"
            const indexes = []
            for (const i in ["x", "y", "z"]) {
              if (i === "2") break
              indexes.push(i)
            }
            return indexes
        "#
        ),
        json!(["0", "1"])
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn for_in_supports_let_declarations_and_bare_identifiers() {
    assert_eq!(
        value("let last = \"\"; for (let key in { a: 1, b: 2 }) last = key; return last"),
        json!("b")
    );
    assert_eq!(
        value("let key = \"before\"; for (key in { only: 1 }) {}; return key"),
        json!("only")
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn for_in_enumerates_namespaces_and_tools_from_the_callable_tool_tree() {
    assert_eq!(
        value(
            r#"
            const names = []
            for (const ns in tools) {
              for (const name in tools[ns]) names.push(ns + "." + name)
            }
            return names
        "#
        ),
        json!([
            "github.list_issues",
            "github.get_issue",
            "memory.search",
            "playwright.navigate",
            "$codemode.search"
        ])
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn unsupported_values_fail_with_a_hint_at_for_of_and_object_keys() {
    for expression in [
        "\"text\"",
        "new Map([[1, 2]])",
        "new Set([1])",
        "42",
        "null",
    ] {
        let failure = error(&format!(
            "for (const key in {expression}) {{}}; return \"no\""
        ));
        assert!(failure
            .message
            .contains("for...in requires a plain object, array, or tools reference"));
        assert!(failure
            .message
            .contains("Use for...of for arrays/strings/Maps/Sets, or Object.keys(value)"));
    }
}
