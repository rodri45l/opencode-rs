//! Port of packages/codemode/test/stdlib.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/codemode/src/index.ts. Re-derived behavioural
//! subset: the standard-library value types are not ported, so these are
//! red-first `#[ignore]` tests over the typed [`CodeMode`] surface.

use opencode_codemode::CodeMode;

fn value(code: &str) -> serde_json::Value {
    let result = CodeMode::execute(code).expect("code-mode execution");
    assert!(result.ok, "expected success");
    result.value.unwrap_or(serde_json::Value::Null)
}

#[test]
#[ignore = "porting: code-mode stdlib value types not implemented"]
fn date_epoch_construction_and_iso_rendering() {
    assert_eq!(
        value(r#"return new Date(0).toISOString()"#),
        serde_json::json!("1970-01-01T00:00:00.000Z")
    );
}

#[test]
#[ignore = "porting: code-mode stdlib value types not implemented"]
fn regex_global_exec_advances_last_index_across_calls() {
    assert_eq!(
        value(
            r#"
            const r = /\d+/g
            const first = r.exec("a1b22c")
            const second = r.exec("a1b22c")
            return [first[0], second[0]]
          "#,
        ),
        serde_json::json!(["1", "22"])
    );
}

#[test]
#[ignore = "porting: code-mode stdlib value types not implemented"]
fn map_get_set_has_size_with_chaining() {
    assert_eq!(
        value(
            r#"
            const m = new Map()
            m.set("a", 1).set("b", 2)
            return { a: m.get("a"), b: m.get("b"), has: m.has("a"), miss: m.get("zz") === undefined, size: m.size }
          "#,
        ),
        serde_json::json!({ "a": 1, "b": 2, "has": true, "miss": true, "size": 2 })
    );
}

#[test]
#[ignore = "porting: code-mode stdlib value types not implemented"]
fn dates_serialize_to_iso_strings_at_the_boundary() {
    assert_eq!(
        value(r#"return { when: new Date(0), tags: [new Date(1000)] }"#),
        serde_json::json!({
            "when": "1970-01-01T00:00:00.000Z",
            "tags": ["1970-01-01T00:00:01.000Z"],
        })
    );
}

#[test]
#[ignore = "porting: code-mode stdlib value types not implemented"]
fn maps_serialize_to_empty_objects_at_the_boundary() {
    assert_eq!(
        value(r#"return new Map([["a", 1]])"#),
        serde_json::json!({})
    );
}
