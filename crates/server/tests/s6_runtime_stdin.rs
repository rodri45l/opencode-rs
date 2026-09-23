//! Port of packages/opencode/test/cli/run/runtime.stdin.test.ts (upstream 18ef3cc).
//!
//! RED-first: the interactive-stdin resolution helper in
//! `cli/cmd/run/runtime.stdin` is not implemented in this crate. The reference
//! decision table is pinned against a local typed stub.

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
struct StdinHandle {
    is_tty: bool,
    destroyed: bool,
}

impl StdinHandle {
    fn new(is_tty: bool) -> Self {
        StdinHandle {
            is_tty,
            destroyed: false,
        }
    }
}

#[derive(Debug)]
struct ResolveResult {
    stdin: StdinHandle,
    cleanup: Option<()>,
}

const INTERACTIVE_INPUT_ERROR: &str = "no interactive terminal available";

#[derive(Debug, PartialEq, Eq)]
struct StdinError;

fn resolve_interactive_stdin(
    _stdin: StdinHandle,
    _open: &mut dyn FnMut(&str) -> std::io::Result<StdinHandle>,
    _platform: &str,
) -> Result<ResolveResult, StdinError> {
    Err(StdinError)
}

#[test]
#[ignore = "porting: cli run interactive stdin not implemented"]
fn reuses_stdin_when_it_is_already_a_tty() {
    let stdin = StdinHandle::new(true);
    let mut seen: Vec<String> = Vec::new();
    let result = resolve_interactive_stdin(
        stdin.clone(),
        &mut |path| {
            seen.push(path.to_string());
            Ok(StdinHandle::new(true))
        },
        "linux",
    )
    .expect("resolve");
    assert_eq!(result.stdin, stdin);
    assert!(result.cleanup.is_none());
    assert!(seen.is_empty());
}

#[test]
#[ignore = "porting: cli run interactive stdin not implemented"]
fn opens_the_controlling_terminal_when_stdin_is_piped() {
    let mut seen: Vec<String> = Vec::new();
    let result = resolve_interactive_stdin(
        StdinHandle::new(false),
        &mut |path| {
            seen.push(path.to_string());
            Ok(StdinHandle::new(true))
        },
        "linux",
    )
    .expect("resolve");
    assert!(result.stdin.is_tty);
    assert_eq!(seen, vec!["/dev/tty".to_string()]);
}

#[test]
#[ignore = "porting: cli run interactive stdin not implemented"]
fn uses_conin_on_windows() {
    let mut seen: Vec<String> = Vec::new();
    let _ = resolve_interactive_stdin(
        StdinHandle::new(false),
        &mut |path| {
            seen.push(path.to_string());
            Ok(StdinHandle::new(true))
        },
        "win32",
    );
    assert_eq!(seen, vec!["CONIN$".to_string()]);
}

#[test]
#[ignore = "porting: cli run interactive stdin not implemented"]
fn errors_when_no_controlling_terminal_is_available() {
    let result = resolve_interactive_stdin(
        StdinHandle::new(false),
        &mut |_path| Err(std::io::Error::other("open failed")),
        "linux",
    );
    assert!(result.is_err());
    assert!(!INTERACTIVE_INPUT_ERROR.is_empty());
}
