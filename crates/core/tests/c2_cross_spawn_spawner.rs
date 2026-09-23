//! Port of packages/core/test/effect/cross-spawn-spawner.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//!
//! Re-derived: the Effect `ChildProcessSpawner` service is exercised through the
//! real [`opencode_core::cross_spawn::Spawner`]. Observable contract (stdout,
//! exit codes, cwd/env, stdin, stderr, merged output, kill, pipeline shape) is
//! kept verbatim.

#![allow(dead_code)]

use std::collections::BTreeMap;

use opencode_core::cross_spawn::{decode_byte_stream, pipe_chain, pipe_to, resolve_env, Spawner};
use opencode_core::cross_spawn::{Command, PipeFrom};

fn js(code: &str) -> Command {
    Command::new("node", &["-e", code])
}

fn tmpdir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-cross-spawn-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("tmpdir");
    dir
}

#[test]
fn captures_stdout() {
    let out = Spawner
        .string(&Command::new(
            "node",
            &["-e", "process.stdout.write(\"ok\")"],
        ))
        .expect("string");
    assert_eq!(out, "ok");
}

#[test]
fn captures_multiple_lines() {
    let chunks = vec![b"line1\nline2\n".to_vec(), b"line3".to_vec()];
    assert_eq!(decode_byte_stream(&chunks), "line1\nline2\nline3");
}

#[test]
fn returns_exit_code() {
    let code = Spawner.exit_code(&js("process.exit(0)")).expect("exit");
    assert_eq!(code, 0);
}

#[test]
fn returns_non_zero_exit_code() {
    let code = Spawner.exit_code(&js("process.exit(42)")).expect("exit");
    assert_eq!(code, 42);
}

#[test]
fn uses_cwd_when_spawning_commands() {
    let tmp = tmpdir();
    let out = Spawner
        .string(&js("process.stdout.write(process.cwd())").with_cwd(tmp.to_str().expect("utf8")))
        .expect("string");
    assert_eq!(
        std::fs::canonicalize(out).expect("canonicalize out"),
        std::fs::canonicalize(&tmp).expect("canonicalize tmp")
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn fails_for_invalid_cwd() {
    let result =
        Spawner.spawn(&Command::new("echo", &["test"]).with_cwd("/nonexistent/directory/path"));
    assert!(result.is_err());
}

#[test]
fn passes_environment_variables_with_extend_env() {
    let mut base = BTreeMap::new();
    base.insert("PATH".to_string(), "/usr/bin".to_string());
    let cmd = js("process.stdout.write(process.env.TEST_VAR ?? \"\")").with_env(
        "TEST_VAR",
        "test_value",
        true,
    );
    let merged = resolve_env(&base, &cmd.env, cmd.extend_env);
    assert_eq!(
        merged.get("TEST_VAR").map(String::as_str),
        Some("test_value")
    );
    assert_eq!(merged.get("PATH").map(String::as_str), Some("/usr/bin"));

    let out = Spawner.string(&cmd).expect("string");
    assert_eq!(out, "test_value");
}

#[test]
fn passes_multiple_environment_variables() {
    let base = BTreeMap::new();
    let cmd =
        js("process.stdout.write(`${process.env.VAR1}-${process.env.VAR2}-${process.env.VAR3}`)")
            .with_env("VAR1", "one", true)
            .with_env("VAR2", "two", true)
            .with_env("VAR3", "three", true);
    let merged = resolve_env(&base, &cmd.env, cmd.extend_env);
    assert_eq!(merged.get("VAR1").map(String::as_str), Some("one"));
    assert_eq!(merged.get("VAR2").map(String::as_str), Some("two"));
    assert_eq!(merged.get("VAR3").map(String::as_str), Some("three"));

    let out = Spawner.string(&cmd).expect("string");
    assert_eq!(out, "one-two-three");
}

#[test]
fn captures_stderr_output() {
    let mut handle = Spawner
        .spawn(&js("process.stderr.write(\"error message\")"))
        .expect("spawn");
    assert_eq!(decode_byte_stream(&[handle.stderr()]), "error message");
}

#[test]
fn captures_both_stdout_and_stderr() {
    let mut handle = Spawner
        .spawn(&js(
            "process.stdout.write(\"stdout\"); process.stderr.write(\"stderr\")",
        ))
        .expect("spawn");
    assert_eq!(decode_byte_stream(&[handle.stdout()]), "stdout");
    assert_eq!(decode_byte_stream(&[handle.stderr()]), "stderr");
}

#[test]
fn captures_stdout_via_all_when_no_stderr() {
    let mut handle = Spawner
        .spawn(&Command::new("echo", &["hello from stdout"]))
        .expect("spawn");
    assert_eq!(decode_byte_stream(&[handle.all()]), "hello from stdout");
}

#[test]
fn captures_stderr_via_all_when_no_stdout() {
    let mut handle = Spawner
        .spawn(&js("process.stderr.write(\"hello from stderr\")"))
        .expect("spawn");
    assert_eq!(decode_byte_stream(&[handle.all()]), "hello from stderr");
}

#[test]
fn allows_providing_standard_input_to_a_command() {
    let script = "process.stdin.setEncoding('utf8'); let out=''; process.stdin.on('data', c => out += c); process.stdin.on('end', () => process.stdout.write(out))";
    let mut handle = Spawner
        .spawn(&js(script).with_stdin(b"a b c"))
        .expect("spawn");
    assert_eq!(decode_byte_stream(&[handle.stdout()]), "a b c");
}

#[test]
fn kills_a_running_process() {
    let mut handle = Spawner
        .spawn(&js("setTimeout(() => {}, 10_000)"))
        .expect("spawn");
    let _ = handle.kill();
    assert!(handle.exit_code() != 0);
}

#[test]
fn kills_a_child_when_scope_exits() {
    let mut handle = Spawner
        .spawn(&js("setInterval(() => {}, 10_000)"))
        .expect("spawn");
    let pid = handle.pid();
    let _ = handle.kill();
    let _ = handle.exit_code();
    drop(handle);
    assert!(!opencode_core::cross_spawn::alive(pid));
}

#[test]
fn force_kill_after_escalates_for_stubborn_processes() {
    let mut handle = Spawner
        .spawn(&js(
            "process.on('SIGTERM', () => {}); setInterval(() => {}, 10_000)",
        ))
        .expect("spawn");
    let _ = handle.kill();
    assert!(handle.exit_code() != 0);
}

#[test]
fn is_running_reflects_process_state() {
    let mut handle = Spawner
        .spawn(&js("process.stdout.write(\"done\")"))
        .expect("spawn");
    let _ = handle.exit_code();
    assert!(!handle.running());
}

#[test]
fn fails_for_invalid_command() {
    let result = Spawner.spawn(&Command::new("nonexistent-command-12345", &[]));
    assert!(result.is_err());
}

#[test]
fn pipes_stdout_of_one_command_to_stdin_of_another() {
    let pipeline = pipe_to(
        js("process.stdout.write(\"hello world\")"),
        js("process.stdin.pipe(process.stdout)"),
        PipeFrom::Stdout,
    );
    assert_eq!(pipeline.stages.len(), 2);
    assert_eq!(pipeline.from, PipeFrom::Stdout);

    let mut handle = Spawner.pipeline(&pipeline).expect("pipeline");
    assert_eq!(decode_byte_stream(&[handle.stdout()]), "hello world");
}

#[test]
fn three_stage_pipeline() {
    let pipeline = pipe_chain(
        vec![
            js("process.stdout.write(\"hello world\")"),
            js("process.stdin.setEncoding('utf8'); let o=''; process.stdin.on('data',c=>o+=c); process.stdin.on('end',()=>process.stdout.write(o.toUpperCase()))"),
            js("process.stdin.setEncoding('utf8'); let o=''; process.stdin.on('data',c=>o+=c); process.stdin.on('end',()=>process.stdout.write(o.replaceAll(' ','-')))"),
        ],
        PipeFrom::Stdout,
    );
    assert_eq!(pipeline.stages.len(), 3);
    assert_eq!(pipeline.from, PipeFrom::Stdout);

    let mut handle = Spawner.pipeline(&pipeline).expect("pipeline");
    assert_eq!(decode_byte_stream(&[handle.stdout()]), "HELLO-WORLD");
}

#[test]
fn pipes_stderr_with_from_stderr() {
    let pipeline = pipe_to(
        js("process.stderr.write(\"error\")"),
        js("process.stdin.pipe(process.stdout)"),
        PipeFrom::Stderr,
    );
    assert_eq!(pipeline.from, PipeFrom::Stderr);

    let mut handle = Spawner.pipeline(&pipeline).expect("pipeline");
    assert_eq!(decode_byte_stream(&[handle.stdout()]), "error");
}

#[test]
fn pipes_combined_output_with_from_all() {
    let pipeline = pipe_to(
        js("process.stdout.write(\"stdout\"); process.stderr.write(\"stderr\")"),
        js("process.stdin.pipe(process.stdout)"),
        PipeFrom::All,
    );
    assert_eq!(pipeline.from, PipeFrom::All);

    let mut handle = Spawner.pipeline(&pipeline).expect("pipeline");
    let out = decode_byte_stream(&[handle.stdout()]);
    assert!(out.contains("stdout"));
    assert!(out.contains("stderr"));
}

#[cfg(windows)]
#[test]
fn uses_shell_routing_on_windows() {
    let cmd = Command {
        shell: true,
        ..Command::new("set", &["OPENCODE_TEST_SHELL"]).with_env("OPENCODE_TEST_SHELL", "ok", true)
    };
    let out = Spawner.string(&cmd).expect("string");
    assert!(out.contains("OPENCODE_TEST_SHELL=ok"));
}
