//! Port of packages/codemode/test/codemode.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/codemode/src/index.ts. Re-derived behavioural
//! subset: the restricted JavaScript interpreter is not ported, so these are
//! red-first `#[ignore]` tests over the typed [`CodeMode`] surface.

use opencode_codemode::{CodeMode, CodeModeErrorKind, ExecutionLimits};

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn preserves_explicit_safe_tool_failures() {
    let result =
        CodeMode::execute("return await tools.host.call({})").expect("code-mode execution");
    assert!(!result.ok);
    let error = result.error.expect("failure carries an error");
    assert_eq!(error.kind, CodeModeErrorKind::ToolFailure);
    assert_eq!(error.message, "Authorized request was refused");
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn sanitizes_unknown_host_failures_and_defects() {
    let result =
        CodeMode::execute("return await tools.host.call({})").expect("code-mode execution");
    let error = result.error.as_ref().expect("failure carries an error");
    assert_eq!(error.kind, CodeModeErrorKind::ToolFailure);
    assert_eq!(error.message, "Tool execution failed");
    let rendered = format!("{result:?}");
    assert!(!rendered.contains("typed-secret"));
    assert!(!rendered.contains("defect-secret"));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn sanitizes_invalid_host_output() {
    let result =
        CodeMode::execute("return await tools.host.call({})").expect("code-mode execution");
    let error = result.error.as_ref().expect("failure carries an error");
    assert_eq!(error.kind, CodeModeErrorKind::InvalidToolOutput);
    assert_eq!(error.message, "Invalid output from tool 'host.call'.");
    assert!(!format!("{result:?}").contains("invalid-output-secret"));
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn captures_console_output_as_bounded_result_logs() {
    let result = CodeMode::execute(
        r#"
        const returned = console.log("Thread info:", { name: "Demo", count: 2 })
        console.warn("careful")
        return returned
      "#,
    )
    .expect("code-mode execution");

    assert!(result.ok);
    assert_eq!(result.value, Some(serde_json::Value::Null));
    assert_eq!(
        result.logs,
        vec![
            "Thread info: {\"name\":\"Demo\",\"count\":2}".to_string(),
            "[warn] careful".to_string(),
        ]
    );
    assert!(result.tool_calls.is_empty());
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn truncates_an_oversized_result_value_with_a_marker() {
    let limits = ExecutionLimits {
        max_output_bytes: Some(40),
        ..Default::default()
    };
    let result = CodeMode::execute_with_limits(
        &format!("return {{ data: \"{}\" }}", "x".repeat(200)),
        &limits,
    )
    .expect("code-mode execution");

    assert!(result.ok);
    assert_eq!(result.truncated, Some(true));
    let value = result.value.expect("value present");
    assert!(value.is_string());
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn rejects_invalid_configuration_and_discovery_limits() {
    let zero_timeout = ExecutionLimits {
        timeout_ms: Some(0),
        ..Default::default()
    };
    assert!(CodeMode::execute_with_limits("return 1", &zero_timeout).is_err());

    let negative_tool_calls = ExecutionLimits {
        max_tool_calls: Some(-1),
        ..Default::default()
    };
    assert!(CodeMode::execute_with_limits("return 1", &negative_tool_calls).is_err());
}

#[test]
#[ignore = "porting: code-mode interpreter not implemented"]
fn enforces_the_tool_call_limit_as_a_diagnostic() {
    let limits = ExecutionLimits {
        max_tool_calls: Some(0),
        ..Default::default()
    };
    let result = CodeMode::execute_with_limits(
        "return await tools.orders.lookup({ id: \"order_42\" })",
        &limits,
    )
    .expect("code-mode execution");
    assert!(!result.ok);
    assert_eq!(
        result.error.expect("failure carries an error").kind,
        CodeModeErrorKind::ToolCallLimitExceeded
    );
}
