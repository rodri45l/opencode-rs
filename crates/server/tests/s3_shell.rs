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

const CWD_COMMANDS: &[&str] = &[
    "cd",
    "chdir",
    "popd",
    "pushd",
    "push-location",
    "set-location",
];
const PROJECT_ROOT: &str = "/project";

fn arity_dictionary() -> &'static [(&'static str, usize)] {
    &[
        ("cat", 1),
        ("cd", 1),
        ("chmod", 1),
        ("chown", 1),
        ("cp", 1),
        ("echo", 1),
        ("env", 1),
        ("export", 1),
        ("grep", 1),
        ("kill", 1),
        ("killall", 1),
        ("ln", 1),
        ("ls", 1),
        ("mkdir", 1),
        ("mv", 1),
        ("ps", 1),
        ("pwd", 1),
        ("rm", 1),
        ("rmdir", 1),
        ("sleep", 1),
        ("source", 1),
        ("tail", 1),
        ("touch", 1),
        ("unset", 1),
        ("which", 1),
        ("aws", 3),
        ("az", 3),
        ("bazel", 2),
        ("brew", 2),
        ("bun", 2),
        ("bun run", 3),
        ("bun x", 3),
        ("cargo", 2),
        ("cargo add", 3),
        ("cargo run", 3),
        ("cmake", 2),
        ("composer", 2),
        ("deno", 2),
        ("deno task", 3),
        ("docker", 2),
        ("docker compose", 3),
        ("docker container", 3),
        ("docker image", 3),
        ("firebase", 2),
        ("flyctl", 2),
        ("gcloud", 3),
        ("gh", 3),
        ("git", 2),
        ("git config", 3),
        ("git remote", 3),
        ("git stash", 3),
        ("go", 2),
        ("gradle", 2),
        ("helm", 2),
        ("kubectl", 2),
        ("make", 2),
        ("mysql", 2),
        ("mvn", 2),
        ("npm", 2),
        ("npm exec", 3),
        ("npm init", 3),
        ("npm run", 3),
        ("npm view", 3),
        ("nvm", 2),
        ("nx", 2),
        ("openssl", 2),
        ("pip", 2),
        ("pipenv", 2),
        ("pnpm", 2),
        ("pnpm dlx", 3),
        ("pnpm exec", 3),
        ("pnpm run", 3),
        ("poetry", 2),
        ("podman", 2),
        ("psql", 2),
        ("pulumi", 2),
        ("python", 2),
        ("rustup", 2),
        ("systemctl", 2),
        ("terraform", 2),
        ("terraform workspace", 3),
        ("tmux", 2),
        ("turbo", 2),
        ("vercel", 2),
        ("yarn", 2),
        ("yarn dlx", 3),
        ("yarn run", 3),
    ]
}

fn split_segments(command: &str) -> Vec<String> {
    let normalized = command.replace(['&', '|', ';', '{', '}'], "\n");
    normalized
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

fn scan_permissions(
    command: &str,
    cwd: &str,
    is_powershell: bool,
) -> Result<Vec<PermissionRequest>> {
    let mut requests = Vec::new();

    if !path_within(cwd, PROJECT_ROOT) {
        let glob = join_glob(cwd);
        requests.push(PermissionRequest {
            permission: "external_directory".to_string(),
            patterns: vec![glob.clone()],
            always: vec![glob],
            metadata: serde_json::json!({ "command": command }),
        });
    }

    let mut patterns: Vec<String> = Vec::new();
    let mut always: Vec<String> = Vec::new();
    let mut external: Vec<String> = Vec::new();

    for segment in split_segments(command) {
        let tokens: Vec<String> = segment.split_whitespace().map(str::to_string).collect();
        if tokens.is_empty() {
            continue;
        }
        let head = if is_powershell {
            tokens[0].to_lowercase()
        } else {
            tokens[0].clone()
        };
        if CWD_COMMANDS.contains(&head.as_str()) {
            continue;
        }

        for token in &tokens {
            let unquoted = token.trim_matches(|c| c == '\'' || c == '"');
            if unquoted.starts_with('/') && !path_within(unquoted, PROJECT_ROOT) {
                let dir = if unquoted.ends_with('*') {
                    unquoted.trim_end_matches('*').trim_end_matches('/')
                } else {
                    unquoted
                };
                let glob = join_glob(dir);
                if !external.contains(&glob) {
                    external.push(glob);
                }
            }
        }

        if !patterns.contains(&segment) {
            patterns.push(segment.clone());
        }
        let prefix = arity_prefix(&tokens)?.join(" ");
        let pattern = format!("{prefix} *");
        if !always.contains(&pattern) {
            always.push(pattern);
        }
    }

    if !external.is_empty() {
        requests.push(PermissionRequest {
            permission: "external_directory".to_string(),
            patterns: external.clone(),
            always: external,
            metadata: serde_json::json!({ "command": command }),
        });
    }

    if !patterns.is_empty() {
        requests.push(PermissionRequest {
            permission: "bash".to_string(),
            patterns,
            always,
            metadata: serde_json::json!({ "command": command }),
        });
    }

    Ok(requests)
}

fn path_within(path: &str, root: &str) -> bool {
    let path = path.trim_end_matches('/');
    let root = root.trim_end_matches('/');
    path == root || path.starts_with(&format!("{root}/"))
}

fn join_glob(dir: &str) -> String {
    let dir = dir.trim_end_matches('/');
    if dir.is_empty() {
        "/*".to_string()
    } else {
        format!("{dir}/*")
    }
}

fn arity_prefix(tokens: &[String]) -> Result<Vec<String>> {
    if tokens.is_empty() {
        return Ok(Vec::new());
    }
    let dictionary = arity_dictionary();
    for len in (1..=tokens.len()).rev() {
        let prefix = tokens[..len].join(" ");
        if let Some((_, arity)) = dictionary.iter().find(|(key, _)| *key == prefix) {
            return Ok(tokens[..*arity].to_vec());
        }
    }
    Ok(tokens[..1].to_vec())
}

fn shell_description(shell_name: &str, default_timeout_ms: u64) -> Result<String> {
    Ok(format!(
        "Executes a given {shell_name} command with optional timeout. You can specify an optional timeout in milliseconds. If not specified, commands will time out after {default_timeout_ms}ms."
    ))
}

fn timeout_message(timeout_ms: u64) -> Result<String> {
    Ok(format!(
        "shell tool terminated command after exceeding timeout {timeout_ms} ms. If this command is expected to take longer and is not waiting for interactive input, retry with a larger timeout value in milliseconds."
    ))
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
fn asks_for_bash_permission_with_the_command_pattern() {
    let requests = scan_permissions("echo hello", "/project", false).expect("shell ported");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].permission, "bash");
    assert!(requests[0].patterns.contains(&"echo hello".to_string()));
}

#[test]
fn asks_for_bash_permission_with_multiple_commands() {
    let requests =
        scan_permissions("echo foo && echo bar", "/project", false).expect("shell ported");
    assert_eq!(requests.len(), 1);
    assert!(requests[0].patterns.contains(&"echo foo".to_string()));
    assert!(requests[0].patterns.contains(&"echo bar".to_string()));
}

#[test]
fn matches_redirects_in_permission_pattern() {
    let requests =
        scan_permissions("echo test > output.txt", "/project", false).expect("shell ported");
    let bash = bash_request(&requests).expect("bash request");
    assert!(bash
        .patterns
        .contains(&"echo test > output.txt".to_string()));
}

#[test]
fn does_not_ask_for_bash_permission_when_command_is_cd_only() {
    let requests = scan_permissions("cd .", "/project", false).expect("shell ported");
    assert!(bash_request(&requests).is_none());
}

#[test]
fn always_pattern_has_space_before_wildcard() {
    let requests = scan_permissions("ls -la", "/project", false).expect("shell ported");
    let bash = bash_request(&requests).expect("bash request");
    assert_eq!(bash.always[0], "ls *");
}

#[test]
fn includes_always_patterns_for_auto_approval() {
    let requests =
        scan_permissions("git log --oneline -5", "/project", false).expect("shell ported");
    assert_eq!(requests.len(), 1);
    assert!(!requests[0].always.is_empty());
    assert!(requests[0].always.iter().any(|item| item.ends_with('*')));
}

#[test]
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
fn uses_powershell_cmdlet_prefixes_for_always_allow() {
    let requests =
        scan_permissions("Remove-Item -Recurse tmp", "/project", true).expect("shell ported");
    let bash = bash_request(&requests).expect("bash request");
    assert!(bash.always.contains(&"Remove-Item *".to_string()));
    assert!(!bash.always.contains(&"Remove-Item -Recurse *".to_string()));
}

#[test]
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
fn asks_for_external_directory_permission_for_wildcard_external_paths() {
    let requests = scan_permissions("cat /etc/*", "/project", false).expect("shell ported");
    let external = external_directory_request(&requests).expect("external_directory request");
    assert!(external.patterns.contains(&"/etc/*".to_string()));
}

#[test]
fn asks_for_external_directory_permission_when_workdir_is_outside_project() {
    let requests = scan_permissions("echo ok", "/tmp/outside", false).expect("shell ported");
    // `scan_permissions` receives the resolved workdir; an out-of-project cwd is
    // surfaced as an external_directory request keyed on that directory.
    let external = external_directory_request(&requests).expect("external_directory request");
    assert!(external
        .patterns
        .iter()
        .any(|pattern| pattern.ends_with("/*")));
}

#[test]
fn does_not_ask_for_external_directory_permission_when_rm_inside_project() {
    let requests =
        scan_permissions("rm -rf /project/nested", "/project", false).expect("shell ported");
    assert!(external_directory_request(&requests).is_none());
}

#[test]
fn description_includes_shell_name_and_timeout() {
    let description = shell_description("bash", 500).expect("shell ported");
    assert!(description.contains("bash"));
    assert!(description.contains("commands will time out after 500ms"));
}

#[test]
fn timeout_message_names_the_timeout_and_retry_hint() {
    let message = timeout_message(500).expect("shell ported");
    assert!(message.contains("shell tool terminated command after exceeding timeout"));
    assert!(message.contains("exceeding timeout 500 ms"));
    assert!(message.contains("retry with a larger timeout value in milliseconds"));
}
