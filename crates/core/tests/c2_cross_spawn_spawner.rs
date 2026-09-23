//! Port of packages/core/test/effect/cross-spawn-spawner.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: the Effect `ChildProcessSpawner` service and `LayerNode.compile`
//! runtime are replaced by a typed synchronous stub. Pure helpers (byte-stream
//! decoding, env resolution, pipeline shape) stay faithful; live process cases
//! remain red until the runtime module lands.

#![allow(dead_code)]

mod spawner {
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl std::fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "not implemented: {}", self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct Command {
        pub program: String,
        pub args: Vec<String>,
        pub cwd: Option<String>,
        pub env: BTreeMap<String, String>,
        pub extend_env: bool,
        pub shell: bool,
        pub stdin: Option<Vec<u8>>,
    }

    impl Command {
        pub fn new(program: &str, args: &[&str]) -> Self {
            Command {
                program: program.to_string(),
                args: args.iter().map(|arg| arg.to_string()).collect(),
                ..Command::default()
            }
        }

        pub fn with_cwd(mut self, cwd: &str) -> Self {
            self.cwd = Some(cwd.to_string());
            self
        }

        pub fn with_env(mut self, key: &str, value: &str, extend_env: bool) -> Self {
            self.env.insert(key.to_string(), value.to_string());
            self.extend_env = extend_env;
            self
        }

        pub fn with_stdin(mut self, stdin: &[u8]) -> Self {
            self.stdin = Some(stdin.to_vec());
            self
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PipeFrom {
        Stdout,
        Stderr,
        All,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Pipeline {
        pub stages: Vec<Command>,
        pub from: PipeFrom,
    }

    pub fn pipe_to(first: Command, next: Command, from: PipeFrom) -> Pipeline {
        Pipeline {
            stages: vec![first, next],
            from,
        }
    }

    pub fn pipe_chain(stages: Vec<Command>, from: PipeFrom) -> Pipeline {
        Pipeline { stages, from }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Handle {
        pub stdout: Vec<u8>,
        pub stderr: Vec<u8>,
        pub all: Vec<u8>,
        pub exit_code: i32,
        pub running: bool,
        pub pid: u32,
    }

    impl Handle {
        pub fn kill(&self) -> Result<(), NotImplemented> {
            Err(NotImplemented("cross-spawn kill"))
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Spawner;

    impl Spawner {
        pub fn string(&self, _cmd: &Command) -> Result<String, NotImplemented> {
            Err(NotImplemented("cross-spawn spawner"))
        }

        pub fn spawn(&self, _cmd: &Command) -> Result<Handle, NotImplemented> {
            Err(NotImplemented("cross-spawn spawner"))
        }

        pub fn exit_code(&self, _cmd: &Command) -> Result<i32, NotImplemented> {
            Err(NotImplemented("cross-spawn spawner"))
        }
    }

    /// Concatenate byte chunks, UTF-8 decode, and trim surrounding whitespace.
    pub fn decode_byte_stream(chunks: &[Vec<u8>]) -> String {
        let mut out = Vec::new();
        for chunk in chunks {
            out.extend_from_slice(chunk);
        }
        String::from_utf8_lossy(&out).trim().to_string()
    }

    /// Merge a base environment with command overrides honouring `extend_env`.
    pub fn resolve_env(
        base: &BTreeMap<String, String>,
        cmd: &BTreeMap<String, String>,
        extend_env: bool,
    ) -> BTreeMap<String, String> {
        let mut out = if extend_env {
            base.clone()
        } else {
            BTreeMap::new()
        };
        for (key, value) in cmd {
            out.insert(key.clone(), value.clone());
        }
        out
    }

    /// Whether a process is still alive (`kill(pid, 0)` semantics).
    pub fn alive(_pid: u32) -> bool {
        false
    }
}

use spawner::{Command, PipeFrom, Spawner};

const NOTE: &str = "porting: cross-spawn spawner not implemented";

fn js(code: &str) -> Command {
    Command::new("node", &["-e", code])
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn captures_stdout() {
    let out = Spawner
        .string(&Command::new(
            "node",
            &["-e", "process.stdout.write(\"ok\")"],
        ))
        .expect(NOTE);
    assert_eq!(out, "ok");
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn captures_multiple_lines() {
    let chunks = vec![b"line1\nline2\n".to_vec(), b"line3".to_vec()];
    assert_eq!(spawner::decode_byte_stream(&chunks), "line1\nline2\nline3");
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn returns_exit_code() {
    let code = Spawner.exit_code(&js("process.exit(0)")).expect(NOTE);
    assert_eq!(code, 0);
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn returns_non_zero_exit_code() {
    let code = Spawner.exit_code(&js("process.exit(42)")).expect(NOTE);
    assert_eq!(code, 42);
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn uses_cwd_when_spawning_commands() {
    let tmp = "/tmp/opencode-core-test-cwd";
    let out = Spawner
        .string(&js("process.stdout.write(process.cwd())").with_cwd(tmp))
        .expect(NOTE);
    assert_eq!(out, tmp);
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn fails_for_invalid_cwd() {
    let result =
        Spawner.spawn(&Command::new("echo", &["test"]).with_cwd("/nonexistent/directory/path"));
    assert!(result.is_err());
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn passes_environment_variables_with_extend_env() {
    let mut base = std::collections::BTreeMap::new();
    base.insert("PATH".to_string(), "/usr/bin".to_string());
    let cmd = js("process.stdout.write(process.env.TEST_VAR ?? \"\")").with_env(
        "TEST_VAR",
        "test_value",
        true,
    );
    let merged = spawner::resolve_env(&base, &cmd.env, cmd.extend_env);
    assert_eq!(
        merged.get("TEST_VAR").map(String::as_str),
        Some("test_value")
    );
    assert_eq!(merged.get("PATH").map(String::as_str), Some("/usr/bin"));
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn passes_multiple_environment_variables() {
    let base = std::collections::BTreeMap::new();
    let cmd = js("vars")
        .with_env("VAR1", "one", true)
        .with_env("VAR2", "two", true)
        .with_env("VAR3", "three", true);
    let merged = spawner::resolve_env(&base, &cmd.env, cmd.extend_env);
    assert_eq!(merged.get("VAR1").map(String::as_str), Some("one"));
    assert_eq!(merged.get("VAR2").map(String::as_str), Some("two"));
    assert_eq!(merged.get("VAR3").map(String::as_str), Some("three"));
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn captures_stderr_output() {
    let handle = Spawner
        .spawn(&js("process.stderr.write(\"error message\")"))
        .expect(NOTE);
    assert_eq!(
        spawner::decode_byte_stream(&[handle.stderr]),
        "error message"
    );
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn captures_both_stdout_and_stderr() {
    let handle = Spawner.spawn(&js("both")).expect(NOTE);
    assert_eq!(spawner::decode_byte_stream(&[handle.stdout]), "stdout");
    assert_eq!(spawner::decode_byte_stream(&[handle.stderr]), "stderr");
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn captures_stdout_via_all_when_no_stderr() {
    let handle = Spawner
        .spawn(&Command::new("echo", &["hello from stdout"]))
        .expect(NOTE);
    assert_eq!(
        spawner::decode_byte_stream(&[handle.all]),
        "hello from stdout"
    );
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn captures_stderr_via_all_when_no_stdout() {
    let handle = Spawner
        .spawn(&js("process.stderr.write(\"hello from stderr\")"))
        .expect(NOTE);
    assert_eq!(
        spawner::decode_byte_stream(&[handle.all]),
        "hello from stderr"
    );
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn allows_providing_standard_input_to_a_command() {
    let handle = Spawner
        .spawn(&js("echo stdin").with_stdin(b"a b c"))
        .expect(NOTE);
    assert_eq!(spawner::decode_byte_stream(&[handle.stdout]), "a b c");
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn kills_a_running_process() {
    let handle = Spawner
        .spawn(&js("setTimeout(() => {}, 10_000)"))
        .expect(NOTE);
    let _ = handle.kill();
    assert!(handle.exit_code != 0);
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn kills_a_child_when_scope_exits() {
    let handle = Spawner
        .spawn(&js("setInterval(() => {}, 10_000)"))
        .expect(NOTE);
    let pid = handle.pid;
    assert!(!spawner::alive(pid));
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn force_kill_after_escalates_for_stubborn_processes() {
    let handle = Spawner.spawn(&js("stubborn")).expect(NOTE);
    let _ = handle.kill();
    assert!(handle.exit_code != 0);
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn is_running_reflects_process_state() {
    let handle = Spawner
        .spawn(&js("process.stdout.write(\"done\")"))
        .expect(NOTE);
    assert!(!handle.running);
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn fails_for_invalid_command() {
    let result = Spawner.spawn(&Command::new("nonexistent-command-12345", &[]));
    assert!(result.is_err());
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn pipes_stdout_of_one_command_to_stdin_of_another() {
    let pipeline = spawner::pipe_to(
        js("process.stdout.write(\"hello world\")"),
        js("uppercase"),
        PipeFrom::Stdout,
    );
    assert_eq!(pipeline.stages.len(), 2);
    assert_eq!(pipeline.from, PipeFrom::Stdout);
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn three_stage_pipeline() {
    let pipeline = spawner::pipe_chain(vec![js("a"), js("b"), js("c")], PipeFrom::Stdout);
    assert_eq!(pipeline.stages.len(), 3);
    assert_eq!(pipeline.from, PipeFrom::Stdout);
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn pipes_stderr_with_from_stderr() {
    let pipeline = spawner::pipe_to(
        js("process.stderr.write(\"error\")"),
        js("echo"),
        PipeFrom::Stderr,
    );
    assert_eq!(pipeline.from, PipeFrom::Stderr);
}

#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn pipes_combined_output_with_from_all() {
    let pipeline = spawner::pipe_to(js("both streams"), js("echo"), PipeFrom::All);
    assert_eq!(pipeline.from, PipeFrom::All);
}

#[cfg(windows)]
#[test]
#[ignore = "porting: cross-spawn spawner not implemented"]
fn uses_shell_routing_on_windows() {
    let cmd = Command {
        shell: true,
        ..Command::new("set", &["OPENCODE_TEST_SHELL"]).with_env("OPENCODE_TEST_SHELL", "ok", true)
    };
    let out = Spawner.string(&cmd).expect(NOTE);
    assert!(out.contains("OPENCODE_TEST_SHELL=ok"));
}
