//! Port of packages/codemode/test/promise.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/codemode/src/index.ts. Re-derived behavioural
//! subset: first-class promise values are not ported, so these are red-first
//! `#[ignore]` tests over the typed [`CodeMode`] surface.

use opencode_codemode::CodeMode;

#[test]
#[ignore = "porting: code-mode promise values not implemented"]
fn an_un_awaited_tool_call_starts_eagerly_in_call_order() {
    let result = CodeMode::execute(
        r#"
        const a = tools.host.sleepy({ id: 1, ms: 40 })
        const b = tools.host.sleepy({ id: 2, ms: 40 })
        const rb = await b
        const ra = await a
        return [ra, rb]
      "#,
    )
    .expect("code-mode execution");

    assert!(result.ok);
    assert_eq!(result.value, Some(serde_json::json!([1, 2])));
}

#[test]
#[ignore = "porting: code-mode promise values not implemented"]
fn awaiting_the_same_promise_twice_settles_once() {
    let result = CodeMode::execute(
        r#"
        const p = tools.host.sleepy({ id: 7 })
        const x = await p
        const y = await p
        return [x, y]
      "#,
    )
    .expect("code-mode execution");

    assert!(result.ok);
    assert_eq!(result.value, Some(serde_json::json!([7, 7])));
    assert_eq!(result.tool_calls.len(), 1);
}

#[test]
#[ignore = "porting: code-mode promise values not implemented"]
fn promise_all_mixes_promises_and_plain_values() {
    let result = CodeMode::execute(
        r#"return await Promise.all([tools.host.sleepy({ id: 1 }), "plain", tools.host.sleepy({ id: 2 }), 42])"#,
    )
    .expect("code-mode execution");

    assert!(result.ok);
    assert_eq!(result.value, Some(serde_json::json!([1, "plain", 2, 42])));
}

#[test]
#[ignore = "porting: code-mode promise values not implemented"]
fn promise_race_first_settlement_wins_and_losers_are_interrupted() {
    let result = CodeMode::execute(
        r#"
        const fast = tools.host.sleepy({ id: 1, ms: 10 })
        const slow = tools.host.sleepy({ id: 2, ms: 5000 })
        return await Promise.race([fast, slow])
      "#,
    )
    .expect("code-mode execution");

    assert!(result.ok);
    assert_eq!(result.value, Some(serde_json::json!(1)));
}

#[test]
#[ignore = "porting: code-mode promise values not implemented"]
fn returning_an_un_awaited_promise_inside_data_is_a_diagnostic() {
    let result = CodeMode::execute("return { result: tools.host.sleepy({ id: 1 }) }")
        .expect("code-mode execution");
    let error = result.error.expect("failure carries an error");
    assert!(error.message.contains("un-awaited Promise"));
}
