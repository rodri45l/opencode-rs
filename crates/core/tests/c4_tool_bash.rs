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
//! capture. Re-derived: the `ToolRegistry`/`PermissionV2`/`AppProcess`/`Config`
//! layer graph and the live shell are replaced by pure settlement helpers.

#![allow(dead_code)]

const NOTE: &str = "porting: bash tool settlement not implemented";

pub const MAX_CAPTURE_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BashRunResult {
    pub output: String,
    pub exit_code: i32,
    pub output_truncated: bool,
    pub timed_out: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settlement {
    pub content: Vec<String>,
    pub structured_exit: i32,
    pub structured_truncated: bool,
    pub structured_timeout: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalArgs {
    pub workdir_external: bool,
    pub advisories: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortError {
    NotImplemented(&'static str),
}

impl std::fmt::Display for PortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortError::NotImplemented(topic) => write!(f, "not implemented: {topic}"),
        }
    }
}

impl std::error::Error for PortError {}

pub struct BashTool;

impl BashTool {
    pub fn schema_keys() -> Result<Vec<String>, PortError> {
        Err(PortError::NotImplemented(NOTE))
    }

    pub fn settle(_run: &BashRunResult) -> Result<Settlement, PortError> {
        Err(PortError::NotImplemented(NOTE))
    }

    pub fn resolve_workdir(_location: &str, _workdir: Option<&str>) -> Result<String, PortError> {
        Err(PortError::NotImplemented(NOTE))
    }

    pub fn permission_actions(
        _workdir_external: bool,
        _denied: Option<&str>,
    ) -> Result<Vec<String>, PortError> {
        Err(PortError::NotImplemented(NOTE))
    }

    pub fn external_advisories(
        _command: &str,
        _external_root: &str,
    ) -> Result<ExternalArgs, PortError> {
        Err(PortError::NotImplemented(NOTE))
    }
}

fn run(output: &str, exit_code: i32) -> BashRunResult {
    BashRunResult {
        output: output.into(),
        exit_code,
        output_truncated: false,
        timed_out: false,
    }
}

#[test]
#[ignore = "porting: bash tool settlement not implemented"]
fn registers_with_a_schema_that_hides_internal_fields() {
    let keys = BashTool::schema_keys().expect(NOTE);
    assert!(!keys.iter().any(|key| key == "background"));
    assert!(!keys.iter().any(|key| key == "description"));
    assert!(!keys.iter().any(|key| key == "output"));
    assert!(!keys.iter().any(|key| key == "command"));
    assert!(!keys.iter().any(|key| key == "cwd"));
}

#[test]
#[ignore = "porting: bash tool settlement not implemented"]
fn returns_structured_successful_output() {
    assert_eq!(
        BashTool::settle(&run("hello\n", 0)).expect(NOTE),
        Settlement {
            content: vec!["hello\n".into(), "Command exited with code 0.".into()],
            structured_exit: 0,
            structured_truncated: false,
            structured_timeout: false,
        }
    );
}

#[test]
#[ignore = "porting: bash tool settlement not implemented"]
fn resolves_a_relative_workdir_from_the_active_location() {
    assert_eq!(
        BashTool::resolve_workdir("/tmp/project", Some("src")).expect(NOTE),
        "/tmp/project/src"
    );
    assert_eq!(
        BashTool::resolve_workdir("/tmp/project", None).expect(NOTE),
        "/tmp/project"
    );
}

#[test]
#[ignore = "porting: bash tool settlement not implemented"]
fn approves_an_explicit_external_workdir_before_bash() {
    assert_eq!(
        BashTool::permission_actions(true, None).expect(NOTE),
        vec!["external_directory".to_string(), "bash".to_string()]
    );
    assert_eq!(
        BashTool::permission_actions(false, None).expect(NOTE),
        vec!["bash".to_string()]
    );
}

#[test]
#[ignore = "porting: bash tool settlement not implemented"]
fn does_not_execute_after_external_directory_or_bash_denial() {
    assert_eq!(
        BashTool::permission_actions(true, Some("external_directory")).expect(NOTE),
        vec!["external_directory".to_string()]
    );
    assert_eq!(
        BashTool::permission_actions(false, Some("bash")).expect(NOTE),
        vec!["bash".to_string()]
    );
}

#[test]
#[ignore = "porting: bash tool settlement not implemented"]
fn reports_external_command_arguments_as_advisory_warnings() {
    let advisory =
        BashTool::external_advisories("cat /outside/secret.txt", "/outside").expect(NOTE);
    assert!(advisory
        .advisories
        .iter()
        .any(|line| line.contains("secret.txt")));
}

#[test]
#[ignore = "porting: bash tool settlement not implemented"]
fn keeps_non_zero_exits_useful() {
    let settled = BashTool::settle(&run("HEAD full output TAIL", 7)).expect(NOTE);
    assert_eq!(settled.structured_exit, 7);
    assert!(!settled.structured_truncated);
    assert_eq!(settled.content[0], "HEAD full output TAIL");
    assert!(settled.content[1].contains("Command exited with code 7"));
}

#[test]
#[ignore = "porting: bash tool settlement not implemented"]
fn surfaces_bounded_process_capture_truncation() {
    let settled = BashTool::settle(&BashRunResult {
        output: "verbose".into(),
        exit_code: 0,
        output_truncated: true,
        timed_out: false,
    })
    .expect(NOTE);
    assert!(settled.structured_truncated);
    assert!(settled.content[0].contains("output capture truncated"));
}

#[test]
#[ignore = "porting: bash tool settlement not implemented"]
fn returns_a_useful_timeout_settlement() {
    let settled = BashTool::settle(&BashRunResult {
        output: String::new(),
        exit_code: 0,
        output_truncated: false,
        timed_out: true,
    })
    .expect(NOTE);
    assert!(settled.structured_timeout);
    assert!(!settled.structured_truncated);
    assert!(settled.content[1].contains("Command timed out"));
}
