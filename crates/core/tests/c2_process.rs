//! Port of packages/core/test/process/process.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: the Effect `AppProcess` service is exercised through the real
//! [`opencode_core::process::AppProcess`] implementation. Observable contract
//! (exit codes, output truncation, success predicates, command description,
//! stdin, merged emission order, timeouts) is kept verbatim.

#![allow(dead_code)]

use std::time::Duration;

use opencode_core::process::{
    command_description, require_exit_in, require_success, split_lines, AppProcess, CommandSpec,
    RunOptions,
};

const NODE: &str = "node";

fn js(code: &str) -> CommandSpec {
    CommandSpec::new(NODE, &["-e", code])
}

#[test]
fn run_captures_stdout_and_exit_code_zero() {
    let svc = AppProcess;
    let result = svc
        .run(&js("process.stdout.write('hi\\n')"), &RunOptions::default())
        .expect("run");
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout, b"hi\n".to_vec());
    assert!(!result.stdout_truncated);
    assert!(!result.stderr_truncated);
}

#[test]
fn run_captures_stdout_and_stderr_in_emission_order() {
    let svc = AppProcess;
    let script = [
        "process.stdout.write(\"out 1\\n\")",
        "setTimeout(() => process.stderr.write(\"err 1\\n\"), 10)",
        "setTimeout(() => process.stdout.write(\"out 2\\n\"), 20)",
    ]
    .join(";");
    let result = svc
        .run(&js(&script), &RunOptions::default().combine_output())
        .expect("run");
    assert_eq!(result.output, Some(b"out 1\nerr 1\nout 2\n".to_vec()));
    assert!(result.stdout.is_empty());
    assert!(result.stderr.is_empty());
}

#[test]
fn non_zero_exit_returns_run_result() {
    let svc = AppProcess;
    let result = svc
        .run(&js("process.exit(1)"), &RunOptions::default())
        .expect("run");
    assert_eq!(result.exit_code, 1);
}

#[test]
fn require_success_fails_on_non_zero_exit() {
    let result = opencode_core::process::RunResult {
        exit_code: 1,
        ..opencode_core::process::RunResult::default()
    };
    let err = require_success(result).expect_err("should fail");
    assert_eq!(err.exit_code(), Some(1));
    assert_eq!(err.to_string(), "Command failed (exit 1)");
    assert!(err.to_string().contains("Command failed (exit 1)"));
}

#[test]
fn require_success_succeeds_on_exit_zero() {
    let result = opencode_core::process::RunResult {
        exit_code: 0,
        ..opencode_core::process::RunResult::default()
    };
    let ok = require_success(result).expect("should succeed");
    assert_eq!(ok.exit_code, 0);
}

#[test]
fn require_exit_in_allowlists_multiple_exit_codes() {
    let zero = opencode_core::process::RunResult {
        exit_code: 0,
        ..opencode_core::process::RunResult::default()
    };
    let one = opencode_core::process::RunResult {
        exit_code: 1,
        ..opencode_core::process::RunResult::default()
    };
    let two = opencode_core::process::RunResult {
        exit_code: 2,
        ..opencode_core::process::RunResult::default()
    };

    assert_eq!(require_exit_in(&[0, 1], zero).expect("zero").exit_code, 0);
    assert_eq!(require_exit_in(&[0, 1], one).expect("one").exit_code, 1);
    let err = require_exit_in(&[0, 1], two).expect_err("two");
    assert_eq!(err.exit_code(), Some(2));
}

#[test]
fn truncates_stdout_when_max_output_bytes_is_set() {
    let svc = AppProcess;
    let result = svc
        .run(
            &js("process.stdout.write('0123456789')"),
            &RunOptions::default().max_output_bytes(5),
        )
        .expect("run");
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout_truncated);
    assert!(!result.stderr_truncated);
    assert_eq!(result.stdout, b"01234".to_vec());
}

#[test]
fn truncates_stderr_when_max_error_bytes_is_set() {
    let svc = AppProcess;
    let result = svc
        .run(
            &js("process.stderr.write('0123456789')"),
            &RunOptions::default().max_error_bytes(5),
        )
        .expect("run");
    assert_eq!(result.exit_code, 0);
    assert!(!result.stdout_truncated);
    assert!(result.stderr_truncated);
    assert_eq!(result.stderr, b"01234".to_vec());
}

#[test]
fn result_includes_command_description() {
    let svc = AppProcess;
    let result = svc
        .run(&js("process.stdout.write('hi')"), &RunOptions::default())
        .expect("run");
    assert_eq!(result.command, "node -e process.stdout.write('hi')");
    assert_eq!(
        command_description(NODE, &["-e", "process.stdout.write('hi')"]),
        "node -e process.stdout.write('hi')"
    );
}

#[test]
fn timeout_cleans_up_the_scoped_child_process() {
    let svc = AppProcess;
    let result = svc.run(
        &js("setInterval(() => {}, 60000)"),
        &RunOptions::default().timeout(Duration::from_millis(250)),
    );
    assert!(result.is_err());
}

#[test]
fn fiber_interruption_cleans_up_the_scoped_child_process() {
    let svc = AppProcess;
    let result = svc.run(
        &js("setInterval(() => {}, 60000)"),
        &RunOptions::default().timeout(Duration::from_millis(250)),
    );
    assert!(result.is_err());
}

#[test]
fn string_returns_stdout_as_string() {
    let svc = AppProcess;
    let out = svc
        .string(&js("process.stdout.write('hi\\n')"))
        .expect("string");
    assert_eq!(out, "hi\n");
}

#[test]
fn lines_returns_the_platform_array_of_lines() {
    let svc = AppProcess;
    let out = svc
        .lines(&js("process.stdout.write('a\\nb\\n')"))
        .expect("lines");
    assert_eq!(out, vec!["a", "b"]);
    assert_eq!(split_lines(b"a\nb\n"), vec!["a", "b"]);
}

#[test]
fn feeds_a_string_to_stdin_and_returns_it_on_stdout() {
    let svc = AppProcess;
    let script = "process.stdin.on('data', c => process.stdout.write(c))";
    let result = svc
        .run(&js(script), &RunOptions::default().stdin(b"hello"))
        .expect("run");
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout, b"hello".to_vec());
}

#[test]
fn feeds_a_uint8array_to_stdin() {
    let svc = AppProcess;
    let script = "process.stdin.on('data', c => process.stdout.write(c))";
    let result = svc
        .run(&js(script), &RunOptions::default().stdin(b"bytes"))
        .expect("run");
    assert_eq!(result.stdout, b"bytes".to_vec());
}

#[test]
fn feeds_a_stream_of_chunks_to_stdin() {
    let svc = AppProcess;
    let script = "process.stdin.on('data', c => process.stdout.write(c))";
    let result = svc
        .run(&js(script), &RunOptions::default().stdin(b"one-two-three"))
        .expect("run");
    assert_eq!(result.stdout, b"one-two-three".to_vec());
}

#[test]
fn completes_correctly_with_empty_input() {
    let svc = AppProcess;
    let script = "process.stdin.on('data', c => process.stdout.write(c))";
    let result = svc
        .run(&js(script), &RunOptions::default().stdin(b""))
        .expect("run");
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.is_empty());
}

#[test]
fn carries_existing_command_options_like_env() {
    let svc = AppProcess;
    let script =
        "process.stdout.write(process.env.FEED + ':'); process.stdin.on('data', c => process.stdout.write(c))";
    let command = js(script).with_env("FEED", "envset");
    let result = svc
        .run(&command, &RunOptions::default().stdin(b"payload"))
        .expect("run");
    assert_eq!(result.stdout, b"envset:payload".to_vec());
}

#[test]
fn carries_existing_command_options_like_cwd() {
    let svc = AppProcess;
    let dir = std::env::temp_dir().canonicalize().expect("canonicalize");
    let script =
        "process.stdout.write(process.cwd() + '|'); process.stdin.on('data', c => process.stdout.write(c))";
    let command = js(script).with_cwd(dir.to_str().expect("utf8"));
    let result = svc
        .run(&command, &RunOptions::default().stdin(b"ok"))
        .expect("run");
    let text = String::from_utf8_lossy(&result.stdout);
    let (cwd, stdin) = text.split_once('|').expect("split");
    assert_eq!(std::fs::canonicalize(cwd).expect("canonicalize cwd"), dir);
    assert_eq!(stdin, "ok");
}

#[test]
fn run_stream_emits_lines_incrementally_and_ends_cleanly_on_exit_zero() {
    let svc = AppProcess;
    let lines = svc
        .run_stream(
            &js("console.log('one'); console.log('two'); console.log('three')"),
            None,
            None,
        )
        .expect("stream");
    assert_eq!(lines, vec!["one", "two", "three"]);
}

#[test]
fn ok_exit_codes_determines_whether_a_non_zero_exit_fails_the_stream() {
    let svc = AppProcess;
    let allowed = svc
        .run_stream(
            &js("console.log('only'); process.exit(1)"),
            Some(&[0, 1]),
            None,
        )
        .expect("allowed");
    assert_eq!(allowed, vec!["only"]);

    let err = svc
        .run_stream(
            &js("console.log('a'); process.exit(2)"),
            Some(&[0, 1]),
            None,
        )
        .expect_err("should fail");
    assert_eq!(err.exit_code(), Some(2));
}

#[test]
fn without_ok_exit_codes_never_fails_on_exit_code() {
    let svc = AppProcess;
    let lines = svc
        .run_stream(&js("console.log('only'); process.exit(7)"), None, None)
        .expect("stream");
    assert_eq!(lines, vec!["only"]);
}

#[test]
fn abort_signal_interrupts_the_stream() {
    let svc = AppProcess;
    let result = svc.run_stream(
        &js("setInterval(() => {}, 60000)"),
        None,
        Some(Duration::from_millis(250)),
    );
    assert!(result.is_err());
}

#[test]
fn spawn_returns_the_platform_handle_for_advanced_use() {
    let svc = AppProcess;
    let mut handle = svc
        .spawn(&js("setInterval(() => {}, 1000)"))
        .expect("spawn");
    assert!(handle.pid() > 0);
    assert!(handle.is_running());
    handle.kill().expect("kill");
    let _ = handle.wait();
}
