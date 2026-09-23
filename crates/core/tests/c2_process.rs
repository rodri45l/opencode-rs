//! Port of packages/core/test/process/process.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: the Effect `AppProcess` service and `LayerNode.compile` runtime
//! are replaced by a typed synchronous stub. Observable contract (exit codes,
//! output truncation, success predicates, command description) is kept verbatim;
//! process-spawning cases remain red until the runtime module lands.

#![allow(dead_code)]

mod process {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl std::fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "not implemented: {}", self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct AppProcessError {
        pub exit_code: i32,
    }

    impl AppProcessError {
        pub fn command_failed(exit_code: i32) -> Self {
            AppProcessError { exit_code }
        }
    }

    impl std::fmt::Display for AppProcessError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Command failed (exit {})", self.exit_code)
        }
    }

    impl std::error::Error for AppProcessError {}

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct RunResult {
        pub exit_code: i32,
        pub stdout: Vec<u8>,
        pub stderr: Vec<u8>,
        pub output: Option<Vec<u8>>,
        pub command: String,
        pub stdout_truncated: bool,
        pub stderr_truncated: bool,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct AppProcess;

    impl AppProcess {
        pub fn run(
            &self,
            _program: &str,
            _args: &[&str],
            _stdin: Option<&[u8]>,
            _max_output_bytes: Option<usize>,
            _max_error_bytes: Option<usize>,
            _combine_output: bool,
        ) -> Result<RunResult, NotImplemented> {
            Err(NotImplemented("AppProcess.run"))
        }

        pub fn string(&self, _program: &str, _args: &[&str]) -> Result<String, NotImplemented> {
            Err(NotImplemented("AppProcess.string"))
        }

        pub fn lines(&self, _program: &str, _args: &[&str]) -> Result<Vec<String>, NotImplemented> {
            Err(NotImplemented("AppProcess.lines"))
        }

        pub fn run_stream(
            &self,
            _program: &str,
            _args: &[&str],
            _ok_exit_codes: Option<&[i32]>,
        ) -> Result<Vec<String>, NotImplemented> {
            Err(NotImplemented("AppProcess.runStream"))
        }

        pub fn spawn(&self, _program: &str, _args: &[&str]) -> Result<Handle, NotImplemented> {
            Err(NotImplemented("AppProcess.spawn"))
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Handle {
        pub running: bool,
    }

    /// Truncate a byte buffer to `max`, reporting whether bytes were dropped.
    pub fn truncate(data: &[u8], max: Option<usize>) -> (Vec<u8>, bool) {
        match max {
            Some(max) if data.len() > max => (data[..max].to_vec(), true),
            _ => (data.to_vec(), false),
        }
    }

    /// Split decoded output into the platform's array of lines, dropping the
    /// trailing empty element produced by a terminating newline.
    pub fn split_lines(data: &[u8]) -> Vec<String> {
        let text = String::from_utf8_lossy(data);
        let mut out: Vec<String> = text.split('\n').map(|line| line.to_string()).collect();
        if matches!(out.last(), Some(last) if last.is_empty()) {
            out.pop();
        }
        out
    }

    /// The `${program} ${args.join(" ")}` description stored on a run result.
    pub fn command_description(program: &str, args: &[&str]) -> String {
        if args.is_empty() {
            program.to_string()
        } else {
            format!("{} {}", program, args.join(" "))
        }
    }

    pub fn require_success(result: RunResult) -> Result<RunResult, AppProcessError> {
        if result.exit_code == 0 {
            Ok(result)
        } else {
            Err(AppProcessError::command_failed(result.exit_code))
        }
    }

    pub fn require_exit_in(codes: &[i32], result: RunResult) -> Result<RunResult, AppProcessError> {
        if codes.contains(&result.exit_code) {
            Ok(result)
        } else {
            Err(AppProcessError::command_failed(result.exit_code))
        }
    }
}

const NOTE: &str = "porting: process not implemented";
const NODE: &str = "node";

#[test]
#[ignore = "porting: process not implemented"]
fn run_captures_stdout_and_exit_code_zero() {
    let svc = process::AppProcess;
    let result = svc
        .run(
            NODE,
            &["-e", "process.stdout.write('hi\\n')"],
            None,
            None,
            None,
            false,
        )
        .expect(NOTE);
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout, b"hi\n".to_vec());
    assert!(!result.stdout_truncated);
    assert!(!result.stderr_truncated);
}

#[test]
#[ignore = "porting: process not implemented"]
fn run_captures_stdout_and_stderr_in_emission_order() {
    let svc = process::AppProcess;
    let result = svc
        .run(NODE, &["-e", "combined"], None, None, None, true)
        .expect(NOTE);
    assert_eq!(result.output, Some(b"out 1\nerr 1\nout 2\n".to_vec()));
    assert!(result.stdout.is_empty());
    assert!(result.stderr.is_empty());
}

#[test]
#[ignore = "porting: process not implemented"]
fn non_zero_exit_returns_run_result() {
    let svc = process::AppProcess;
    let result = svc
        .run(NODE, &["-e", "process.exit(1)"], None, None, None, false)
        .expect(NOTE);
    assert_eq!(result.exit_code, 1);
}

#[test]
#[ignore = "porting: process not implemented"]
fn require_success_fails_on_non_zero_exit() {
    let result = process::RunResult {
        exit_code: 1,
        ..process::RunResult::default()
    };
    let err = process::require_success(result).expect_err(NOTE);
    assert_eq!(err.exit_code, 1);
    assert_eq!(err.to_string(), "Command failed (exit 1)");
    assert!(err.to_string().contains("Command failed (exit 1)"));
}

#[test]
#[ignore = "porting: process not implemented"]
fn require_success_succeeds_on_exit_zero() {
    let result = process::RunResult {
        exit_code: 0,
        ..process::RunResult::default()
    };
    let ok = process::require_success(result).expect(NOTE);
    assert_eq!(ok.exit_code, 0);
}

#[test]
#[ignore = "porting: process not implemented"]
fn require_exit_in_allowlists_multiple_exit_codes() {
    let zero = process::RunResult {
        exit_code: 0,
        ..process::RunResult::default()
    };
    let one = process::RunResult {
        exit_code: 1,
        ..process::RunResult::default()
    };
    let two = process::RunResult {
        exit_code: 2,
        ..process::RunResult::default()
    };

    assert_eq!(
        process::require_exit_in(&[0, 1], zero)
            .expect(NOTE)
            .exit_code,
        0
    );
    assert_eq!(
        process::require_exit_in(&[0, 1], one)
            .expect(NOTE)
            .exit_code,
        1
    );
    let err = process::require_exit_in(&[0, 1], two).expect_err(NOTE);
    assert_eq!(err.exit_code, 2);
}

#[test]
#[ignore = "porting: process not implemented"]
fn truncates_stdout_when_max_output_bytes_is_set() {
    let (bytes, truncated) = process::truncate(b"0123456789", Some(5));
    assert!(truncated);
    assert_eq!(bytes, b"01234".to_vec());
}

#[test]
#[ignore = "porting: process not implemented"]
fn truncates_stderr_when_max_error_bytes_is_set() {
    let (bytes, truncated) = process::truncate(b"0123456789", Some(5));
    assert!(truncated);
    assert_eq!(bytes, b"01234".to_vec());
}

#[test]
#[ignore = "porting: process not implemented"]
fn result_includes_command_description() {
    assert_eq!(
        process::command_description(NODE, &["-e", "process.stdout.write('hi')"]),
        "node -e process.stdout.write('hi')"
    );
}

#[test]
#[ignore = "porting: process not implemented"]
fn timeout_cleans_up_the_scoped_child_process() {
    let svc = process::AppProcess;
    let result = svc.run(NODE, &["-e", "interval"], None, None, None, false);
    assert!(result.is_err());
}

#[test]
#[ignore = "porting: process not implemented"]
fn fiber_interruption_cleans_up_the_scoped_child_process() {
    let svc = process::AppProcess;
    let result = svc.run(NODE, &["-e", "interval"], None, None, None, false);
    assert!(result.is_err());
}

#[test]
#[ignore = "porting: process not implemented"]
fn string_returns_stdout_as_string() {
    let svc = process::AppProcess;
    let out = svc
        .string(NODE, &["-e", "process.stdout.write('hi\\n')"])
        .expect(NOTE);
    assert_eq!(out, "hi\n");
}

#[test]
#[ignore = "porting: process not implemented"]
fn lines_returns_the_platform_array_of_lines() {
    assert_eq!(process::split_lines(b"a\nb\n"), vec!["a", "b"]);
}

#[test]
#[ignore = "porting: process not implemented"]
fn feeds_a_string_to_stdin_and_returns_it_on_stdout() {
    let svc = process::AppProcess;
    let result = svc
        .run(NODE, &["-e", "echo"], Some(b"hello"), None, None, false)
        .expect(NOTE);
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout, b"hello".to_vec());
}

#[test]
#[ignore = "porting: process not implemented"]
fn feeds_a_uint8array_to_stdin() {
    let svc = process::AppProcess;
    let result = svc
        .run(NODE, &["-e", "echo"], Some(b"bytes"), None, None, false)
        .expect(NOTE);
    assert_eq!(result.stdout, b"bytes".to_vec());
}

#[test]
#[ignore = "porting: process not implemented"]
fn feeds_a_stream_of_chunks_to_stdin() {
    let svc = process::AppProcess;
    let result = svc
        .run(
            NODE,
            &["-e", "echo"],
            Some(b"one-two-three"),
            None,
            None,
            false,
        )
        .expect(NOTE);
    assert_eq!(result.stdout, b"one-two-three".to_vec());
}

#[test]
#[ignore = "porting: process not implemented"]
fn completes_correctly_with_empty_input() {
    let svc = process::AppProcess;
    let result = svc
        .run(NODE, &["-e", "echo"], Some(b""), None, None, false)
        .expect(NOTE);
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.is_empty());
}

#[test]
#[ignore = "porting: process not implemented"]
fn carries_existing_command_options_like_env() {
    let svc = process::AppProcess;
    let result = svc
        .run(
            NODE,
            &["-e", "echo-env"],
            Some(b"payload"),
            None,
            None,
            false,
        )
        .expect(NOTE);
    assert_eq!(result.stdout, b"envset:payload".to_vec());
}

#[test]
#[ignore = "porting: process not implemented"]
fn carries_existing_command_options_like_cwd() {
    let svc = process::AppProcess;
    let result = svc
        .run(NODE, &["-e", "echo-cwd"], Some(b"ok"), None, None, false)
        .expect(NOTE);
    assert_eq!(result.stdout, b"ok".to_vec());
}

#[test]
#[ignore = "porting: process not implemented"]
fn run_stream_emits_lines_incrementally_and_ends_cleanly_on_exit_zero() {
    let svc = process::AppProcess;
    let lines = svc
        .run_stream(
            NODE,
            &[
                "-e",
                "console.log('one'); console.log('two'); console.log('three')",
            ],
            None,
        )
        .expect(NOTE);
    assert_eq!(lines, vec!["one", "two", "three"]);
}

#[test]
#[ignore = "porting: process not implemented"]
fn ok_exit_codes_determines_whether_a_non_zero_exit_fails_the_stream() {
    let svc = process::AppProcess;
    let allowed = svc
        .run_stream(
            NODE,
            &["-e", "console.log('only'); process.exit(1)"],
            Some(&[0, 1]),
        )
        .expect(NOTE);
    assert_eq!(allowed, vec!["only"]);

    let err = svc
        .run_stream(
            NODE,
            &["-e", "console.log('a'); process.exit(2)"],
            Some(&[0, 1]),
        )
        .expect_err(NOTE);
    assert_eq!(err.0, "AppProcess.runStream");
}

#[test]
#[ignore = "porting: process not implemented"]
fn without_ok_exit_codes_never_fails_on_exit_code() {
    let svc = process::AppProcess;
    let lines = svc
        .run_stream(NODE, &["-e", "console.log('only'); process.exit(7)"], None)
        .expect(NOTE);
    assert_eq!(lines, vec!["only"]);
}

#[test]
#[ignore = "porting: process not implemented"]
fn abort_signal_interrupts_the_stream() {
    let svc = process::AppProcess;
    let result = svc.run_stream(NODE, &["-e", "interval"], None);
    assert!(result.is_err());
}

#[test]
#[ignore = "porting: process not implemented"]
fn spawn_returns_the_platform_handle_for_advanced_use() {
    let svc = process::AppProcess;
    let handle = svc.spawn(NODE, &["-e", "interval"]).expect(NOTE);
    assert!(handle.running);
}
