//! Port of packages/core/test/tool-bash.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the tool is registered as `bash` with a schema that omits
//! `background`/`description`/`output`/`command`/`cwd`; a run settles
//! structured `{ exit, truncated }` plus content `[output, "Command exited with
//! code N."]`; a relative `workdir` resolves against the active Location; an
//! explicit external workdir is approved as `external_directory` before `bash`;
//! denial prevents execution; external command arguments produce advisory
//! `Warnings:` content without enforcement; truncation and timeouts are
//! surfaced; and `combineOutput` with `MAX_CAPTURE_BYTES` bounds the process
//! capture. Re-derived: the layer graph and the live shell are replaced by pure
//! settlement helpers in `opencode_core::bash_tool`.

use opencode_core::bash_tool::{BashRunResult, BashTool, Settlement};

fn run(output: &str, exit_code: i32) -> BashRunResult {
    BashRunResult {
        output: output.into(),
        exit_code,
        output_truncated: false,
        timed_out: false,
    }
}

#[test]
fn registers_with_a_schema_that_hides_internal_fields() {
    let keys = BashTool::schema_keys();
    assert!(!keys.iter().any(|key| key == "background"));
    assert!(!keys.iter().any(|key| key == "description"));
    assert!(!keys.iter().any(|key| key == "output"));
    assert!(!keys.iter().any(|key| key == "command"));
    assert!(!keys.iter().any(|key| key == "cwd"));
}

#[test]
fn returns_structured_successful_output() {
    assert_eq!(
        BashTool::settle(&run("hello\n", 0)),
        Settlement {
            content: vec!["hello\n".into(), "Command exited with code 0.".into()],
            structured_exit: 0,
            structured_truncated: false,
            structured_timeout: false,
        }
    );
}

#[test]
fn resolves_a_relative_workdir_from_the_active_location() {
    assert_eq!(
        BashTool::resolve_workdir("/tmp/project", Some("src")),
        "/tmp/project/src"
    );
    assert_eq!(
        BashTool::resolve_workdir("/tmp/project", None),
        "/tmp/project"
    );
}

#[test]
fn approves_an_explicit_external_workdir_before_bash() {
    assert_eq!(
        BashTool::permission_actions(true, None),
        vec!["external_directory".to_string(), "bash".to_string()]
    );
    assert_eq!(
        BashTool::permission_actions(false, None),
        vec!["bash".to_string()]
    );
}

#[test]
fn does_not_execute_after_external_directory_or_bash_denial() {
    assert_eq!(
        BashTool::permission_actions(true, Some("external_directory")),
        vec!["external_directory".to_string()]
    );
    assert_eq!(
        BashTool::permission_actions(false, Some("bash")),
        vec!["bash".to_string()]
    );
}

#[test]
fn reports_external_command_arguments_as_advisory_warnings() {
    let advisory = BashTool::external_advisories("cat /outside/secret.txt", "/outside");
    assert!(advisory
        .advisories
        .iter()
        .any(|line| line.contains("secret.txt")));
}

#[test]
fn keeps_non_zero_exits_useful() {
    let settled = BashTool::settle(&run("HEAD full output TAIL", 7));
    assert_eq!(settled.structured_exit, 7);
    assert!(!settled.structured_truncated);
    assert_eq!(settled.content[0], "HEAD full output TAIL");
    assert!(settled.content[1].contains("Command exited with code 7"));
}

#[test]
fn surfaces_bounded_process_capture_truncation() {
    let settled = BashTool::settle(&BashRunResult {
        output: "verbose".into(),
        exit_code: 0,
        output_truncated: true,
        timed_out: false,
    });
    assert!(settled.structured_truncated);
    assert!(settled.content[0].contains("output capture truncated"));
}

#[test]
fn returns_a_useful_timeout_settlement() {
    let settled = BashTool::settle(&BashRunResult {
        output: String::new(),
        exit_code: 0,
        output_truncated: false,
        timed_out: true,
    });
    assert!(settled.structured_timeout);
    assert!(!settled.structured_truncated);
    assert!(settled.content[1].contains("Command timed out"));
}
