//! Port of packages/opencode/test/tool/shell.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the pure permission scan derived from a command string
//! (bash `patterns`/`always` with `BashArity` prefixes, redirects, cd-only
//! suppression, PowerShell cmdlet prefixes, external-directory globs), plus the
//! timeout message and description. The live subprocess/abort/truncation tests
//! need a real shell and spawner and are dropped.

#![allow(dead_code)]

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ShellError {
    NotImplemented(&'static str),
    InvalidInput(&'static str),
}

type Result<T> = std::result::Result<T, ShellError>;

#[derive(Debug, Clone, PartialEq)]
struct PermissionRequest {
    permission: String,
    patterns: Vec<String>,
    always: Vec<String>,
    metadata: Value,
}

fn scan_permissions(
    _command: &str,
    _cwd: &str,
    _is_powershell: bool,
) -> Result<Vec<PermissionRequest>> {
    Err(ShellError::NotImplemented("shell permission scan"))
}

fn arity_prefix(_tokens: &[String]) -> Result<Vec<String>> {
    Err(ShellError::NotImplemented("BashArity.prefix"))
}

fn shell_description(_shell_name: &str, _default_timeout_ms: u64) -> Result<String> {
    Err(ShellError::NotImplemented("shell tool description"))
}

fn timeout_message(_timeout_ms: u64) -> Result<String> {
    Err(ShellError::NotImplemented("shell timeout message"))
}

fn bash_request(requests: &[PermissionRequest]) -> Option<&PermissionRequest> {
    requests.iter().find(|request| request.permission == "bash")
}

fn external_directory_request(requests: &[PermissionRequest]) -> Option<&PermissionRequest> {
    requests
        .iter()
        .find(|request| request.permission == "external_directory")
}

#[test]
#[ignore = "porting: shell tool not implemented"]
fn asks_for_bash_permission_with_the_command_pattern() {
    let requests = scan_permissions("echo hello", "/project", false).expect("shell ported");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].permission, "bash");
    assert!(requests[0].patterns.contains(&"echo hello".to_string()));
}

#[test]
#[ignore = "porting: shell tool not implemented"]
fn asks_for_bash_permission_with_multiple_commands() {
    let requests =
        scan_permissions("echo foo && echo bar", "/project", false).expect("shell ported");
    assert_eq!(requests.len(), 1);
    assert!(requests[0].patterns.contains(&"echo foo".to_string()));
    assert!(requests[0].patterns.contains(&"echo bar".to_string()));
}

#[test]
#[ignore = "porting: shell tool not implemented"]
fn matches_redirects_in_permission_pattern() {
    let requests =
        scan_permissions("echo test > output.txt", "/project", false).expect("shell ported");
    let bash = bash_request(&requests).expect("bash request");
    assert!(bash
        .patterns
        .contains(&"echo test > output.txt".to_string()));
}

#[test]
#[ignore = "porting: shell tool not implemented"]
fn does_not_ask_for_bash_permission_when_command_is_cd_only() {
    let requests = scan_permissions("cd .", "/project", false).expect("shell ported");
    assert!(bash_request(&requests).is_none());
}

#[test]
#[ignore = "porting: shell tool not implemented"]
fn always_pattern_has_space_before_wildcard() {
    let requests = scan_permissions("ls -la", "/project", false).expect("shell ported");
    let bash = bash_request(&requests).expect("bash request");
    assert_eq!(bash.always[0], "ls *");
}

#[test]
#[ignore = "porting: shell tool not implemented"]
fn includes_always_patterns_for_auto_approval() {
    let requests =
        scan_permissions("git log --oneline -5", "/project", false).expect("shell ported");
    assert_eq!(requests.len(), 1);
    assert!(!requests[0].always.is_empty());
    assert!(requests[0].always.iter().any(|item| item.ends_with('*')));
}

#[test]
#[ignore = "porting: shell tool not implemented"]
fn arity_prefix_uses_the_dictionary_longest_match() {
    assert_eq!(
        arity_prefix(&["ls".to_string(), "-la".to_string()]).expect("ported"),
        vec!["ls"]
    );
    assert_eq!(
        arity_prefix(&[
            "git".to_string(),
            "checkout".to_string(),
            "main".to_string()
        ])
        .expect("ported"),
        vec!["git", "checkout"]
    );
    assert_eq!(
        arity_prefix(&[
            "docker".to_string(),
            "compose".to_string(),
            "up".to_string(),
            "svc".to_string()
        ])
        .expect("ported"),
        vec!["docker", "compose", "up"]
    );
}

#[test]
#[ignore = "porting: shell tool not implemented"]
fn uses_powershell_cmdlet_prefixes_for_always_allow() {
    let requests =
        scan_permissions("Remove-Item -Recurse tmp", "/project", true).expect("shell ported");
    let bash = bash_request(&requests).expect("bash request");
    assert!(bash.always.contains(&"Remove-Item *".to_string()));
    assert!(!bash.always.contains(&"Remove-Item -Recurse *".to_string()));
}

#[test]
#[ignore = "porting: shell tool not implemented"]
fn parses_powershell_conditionals_for_permission_prompts() {
    let requests = scan_permissions(
        "Write-Host foo; if ($?) { Write-Host bar }",
        "/project",
        true,
    )
    .expect("shell ported");
    let bash = bash_request(&requests).expect("bash request");
    assert!(bash.patterns.contains(&"Write-Host foo".to_string()));
    assert!(bash.patterns.contains(&"Write-Host bar".to_string()));
    assert!(bash.always.contains(&"Write-Host *".to_string()));
}

#[test]
#[ignore = "porting: shell tool not implemented"]
fn asks_for_external_directory_permission_for_wildcard_external_paths() {
    let requests = scan_permissions("cat /etc/*", "/project", false).expect("shell ported");
    let external = external_directory_request(&requests).expect("external_directory request");
    assert!(external.patterns.contains(&"/etc/*".to_string()));
}

#[test]
#[ignore = "porting: shell tool not implemented"]
fn asks_for_external_directory_permission_when_workdir_is_outside_project() {
    let requests = scan_permissions("echo ok", "/project", false).expect("shell ported");
    // `scan_permissions` receives the resolved workdir; an out-of-project cwd is
    // surfaced as an external_directory request keyed on that directory.
    let external = external_directory_request(&requests).expect("external_directory request");
    assert!(external
        .patterns
        .iter()
        .any(|pattern| pattern.ends_with("/*")));
}

#[test]
#[ignore = "porting: shell tool not implemented"]
fn does_not_ask_for_external_directory_permission_when_rm_inside_project() {
    let requests =
        scan_permissions("rm -rf /project/nested", "/project", false).expect("shell ported");
    assert!(external_directory_request(&requests).is_none());
}

#[test]
#[ignore = "porting: shell tool not implemented"]
fn description_includes_shell_name_and_timeout() {
    let description = shell_description("bash", 500).expect("shell ported");
    assert!(description.contains("bash"));
    assert!(description.contains("commands will time out after 500ms"));
}

#[test]
#[ignore = "porting: shell tool not implemented"]
fn timeout_message_names_the_timeout_and_retry_hint() {
    let message = timeout_message(500).expect("shell ported");
    assert!(message.contains("shell tool terminated command after exceeding timeout"));
    assert!(message.contains("exceeding timeout 500 ms"));
    assert!(message.contains("retry with a larger timeout value in milliseconds"));
}
