//! Port of packages/codemode/test/parity.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/codemode/src/index.ts and src/tool-runtime.ts: the
//! "ordinary defensive JavaScript just works" parity rules (string/array property reads
//! are undefined, object spread of null is a no-op, `typeof` of an undeclared identifier
//! is "undefined", non-finite values normalize to null at the boundary) and Error
//! `instanceof` semantics.
//! Re-derived: the result boundary is `CodeModeResult`; a bare `undefined` normalizes to
//! `null`. Non-finite numbers cannot be represented by serde_json, so the shared
//! `ToolRuntime::copyOut` boundary is checked on representable nested data.
//! Red-first: the restricted JavaScript interpreter is not implemented.

use opencode_codemode::{CodeMode, CodeModeError, CodeModeErrorKind, CodeModeResult, ToolRuntime};
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
fn unknown_property_on_a_string_is_undefined() {
    assert_eq!(
        value("const s = \"hi\"; return s.login === undefined"),
        json!(true)
    );
    assert_eq!(value("const s = \"hi\"; return s.login"), json!(null));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn optional_chaining_and_fallback_on_a_string_does_not_throw() {
    assert_eq!(
        value("const s = \"hi\"; return s?.login ?? \"fallback\""),
        json!("fallback")
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn the_real_mcp_pattern_result_is_a_json_string_defensive_read_falls_through() {
    assert_eq!(
        value("const me = { result: '{\"login\":\"x\"}' }; return me.result?.login ?? me.result"),
        json!("{\"login\":\"x\"}")
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn unknown_property_on_a_number_is_undefined() {
    assert_eq!(value("return (5).foo ?? \"n\""), json!("n"));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn supported_string_methods_still_work() {
    assert_eq!(value("return \"AB\".toLowerCase()"), json!("ab"));
    assert_eq!(value("return \"hello\".length"), json!(5));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn unknown_property_on_an_array_is_undefined() {
    assert_eq!(value("return [1,2,3].foo === undefined"), json!(true));
    assert_eq!(value("return [1,2,3].foo"), json!(null));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn optional_chaining_on_an_array_does_not_throw() {
    assert_eq!(value("return [1,2,3]?.foo ?? \"fb\""), json!("fb"));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn unknown_property_reads_stay_undefined_for_unimplemented_methods() {
    assert_eq!(value("return [1,2,3].toSpliced === undefined"), json!(true));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn supported_array_methods_and_indexing_still_work() {
    assert_eq!(value("return [1,2,3].map(x => x + 1)"), json!([2, 3, 4]));
    assert_eq!(value("return [1,2,3][9] === undefined"), json!(true));
    assert_eq!(value("return [1,2,3][9]"), json!(null));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn spreading_null_is_a_no_op() {
    assert_eq!(
        value("const o = null; return { ...o, a: 1 }"),
        json!({"a": 1})
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn spreading_an_absent_argument_merges_cleanly() {
    assert_eq!(
        value("function f(opts){ return { ...opts, a: 1 } } return f(undefined)"),
        json!({"a": 1})
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn spreading_a_real_object_still_works() {
    assert_eq!(
        value("const o = { a: 1 }; return { ...o, b: 2 }"),
        json!({"a": 1, "b": 2})
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn spreading_an_array_into_an_object_still_errors() {
    let err = error("return { ...[1,2], a: 1 }");
    assert_eq!(err.kind, CodeModeErrorKind::InvalidDataValue);
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn feature_detection_guard_does_not_throw() {
    assert_eq!(
        value("return typeof foo === \"undefined\" ? \"safe\" : \"no\""),
        json!("safe")
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn typeof_of_a_declared_binding_is_unaffected() {
    assert_eq!(value("const x = 5; return typeof x"), json!("number"));
    assert_eq!(value("const s = \"a\"; return typeof s"), json!("string"));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn referencing_an_undeclared_identifier_outside_typeof_still_throws() {
    let err = error("return foo + 1");
    assert!(err.message.contains("foo"));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn guards_run_instead_of_the_program_crashing_on_a_transient_nan() {
    assert_eq!(value("return parseInt(\"abc\") || 0"), json!(0));
    assert_eq!(
        value("const x = Number(\"abc\"); return Number.isNaN(x) ? 0 : x"),
        json!(0)
    );
    assert_eq!(
        value("const o = {}; o.count = (o.count || 0) + 1; return o.count"),
        json!(1)
    );
    assert_eq!(
        value("const a = []; return a.length ? a.reduce((s,x)=>s+x,0)/a.length : 0"),
        json!(0)
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn a_non_finite_value_becomes_null_when_it_leaves_the_sandbox() {
    assert_eq!(value("return 5/0"), json!(null));
    assert_eq!(value("return 0/0"), json!(null));
    assert_eq!(value("return Math.max()"), json!(null));
    assert_eq!(
        value("return { a: Number(\"x\"), b: 2, c: [1/0] }"),
        json!({"a": null, "b": 2, "c": [null]})
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn nan_and_infinity_are_usable_identifiers_and_inspectable_in_sandbox() {
    assert_eq!(value("return Number.isNaN(NaN)"), json!(true));
    assert_eq!(value("return Infinity > 1e9"), json!(true));
    assert_eq!(value("return Number.isFinite(1/0)"), json!(false));
    assert_eq!(
        value("return [3,1,2].reduce((a,b)=>Math.max(a,b), -Infinity)"),
        json!(3)
    );
    assert_eq!(
        value("return JSON.stringify({ x: Number(\"z\") })"),
        json!("{\"x\":null}")
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn copy_out_normalizes_the_shared_return_and_tool_arg_boundary() {
    // Non-finite JS numbers cannot be represented by serde_json; this re-derives the
    // boundary on representable nested data (the interpreter cases above pin NaN/Inf).
    assert_eq!(ToolRuntime::copy_out(&json!(42)), json!(42));
    assert_eq!(
        ToolRuntime::copy_out(&json!({"a": 1, "b": [2, 3]})),
        json!({"a": 1, "b": [2, 3]})
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn new_error_carries_name_message_and_is_instanceof_error() {
    assert_eq!(
        value("const e = new Error(\"boom\"); return [e instanceof Error, e.name, e.message]"),
        json!([true, "Error", "boom"])
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn error_without_new_behaves_like_new_error() {
    assert_eq!(
        value("const e = Error(\"plain\"); return [e instanceof Error, e.name, e.message]"),
        json!([true, "Error", "plain"])
    );
    assert_eq!(
        value("const e = new Error(); return [e.name, e.message, e instanceof Error]"),
        json!(["Error", "", true])
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn specific_error_types_are_instanceof_themselves_and_error_not_each_other() {
    assert_eq!(
        value("const e = new TypeError(\"t\"); return [e instanceof TypeError, e instanceof Error, e instanceof RangeError]"),
        json!([true, true, false])
    );
    assert_eq!(
        value("return new Error(\"e\") instanceof TypeError"),
        json!(false)
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn thrown_errors_keep_instanceof_through_try_catch() {
    assert_eq!(
        value(
            "try { throw new Error(\"x\") } catch (e) { return [e instanceof Error, e.message] }"
        ),
        json!([true, "x"])
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn interpreter_runtime_failures_are_caught_as_error_values() {
    assert_eq!(
        value("try { JSON.parse(\"nope\") } catch (e) { return e instanceof Error }"),
        json!(true)
    );
    assert_eq!(
        value("try { undeclared() } catch (e) { return e instanceof Error }"),
        json!(true)
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn caught_failures_carry_the_constructor_name_the_real_js_failure_would_have() {
    assert_eq!(
        value(
            r#"
            try { JSON.parse("{oops") } catch (e) {
              return [e.name, e instanceof SyntaxError, e instanceof Error, e instanceof TypeError, e.message.includes("JSON")]
            }
        "#
        ),
        json!(["SyntaxError", true, true, false, true])
    );
    assert_eq!(
        value("try { undeclared() } catch (e) { return [e.name, e instanceof ReferenceError] }"),
        json!(["ReferenceError", true])
    );
    assert_eq!(
        value("try { const c = 1; c = 2 } catch (e) { return [e.name, e instanceof TypeError] }"),
        json!(["TypeError", true])
    );
    assert_eq!(
        value("try { \"a\".normalize(\"NOPE\") } catch (e) { return [e.name, e instanceof RangeError] }"),
        json!(["RangeError", true])
    );
    assert_eq!(
        value("try { \"a\".match(\"(\") } catch (e) { return [e.name, e instanceof SyntaxError] }"),
        json!(["SyntaxError", true])
    );
    assert_eq!(
        value("try { new RegExp(\"(\") } catch (e) { return [e.name, e instanceof SyntaxError] }"),
        json!(["SyntaxError", true])
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn diagnostics_without_a_specific_real_js_analogue_are_named_plain_error() {
    assert_eq!(
        value("try { JSON.parse(5) } catch (e) { return [e.name, e instanceof Error] }"),
        json!(["Error", true])
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn promise_allsettled_rejection_reasons_are_error_values() {
    assert_eq!(
        value(
            r#"
            const settled = await Promise.allSettled([Promise.reject(new Error("b"))])
            return [settled[0].reason instanceof Error, settled[0].reason.message]
        "#
        ),
        json!([true, "b"])
    );
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn non_error_thrown_values_are_not_instanceof_error() {
    assert_eq!(
        value("try { throw \"raw\" } catch (e) { return e instanceof Error }"),
        json!(false)
    );
    assert_eq!(
        value("try { throw { message: \"shaped\" } } catch (e) { return e instanceof Error }"),
        json!(false)
    );
}
