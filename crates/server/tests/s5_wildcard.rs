//! Port of packages/opencode/test/util/wildcard.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: glob-token matching (`?`, `*`, literal `+`), trailing
//! space+wildcard semantics, most-specific rule selection, structured command
//! matching with flag-specific precedence, slash normalisation and Unix path
//! case sensitivity.
#![allow(dead_code)]

use opencode_server::wildcard::{all, all_structured, match_pattern};

#[test]
fn match_handles_glob_tokens() {
    assert!(match_pattern("file1.txt", "file?.txt"));
    assert!(!match_pattern("file12.txt", "file?.txt"));
    assert!(match_pattern("foo+bar", "foo+bar"));
}

#[test]
fn match_with_trailing_space_wildcard_matches_command_with_or_without_args() {
    assert!(match_pattern("ls", "ls *"));
    assert!(match_pattern("ls -la", "ls *"));
    assert!(match_pattern("ls foo bar", "ls *"));

    assert!(match_pattern("ls", "ls*"));
    assert!(match_pattern("lstmeval", "ls*"));

    assert!(!match_pattern("lstmeval", "ls *"));

    assert!(match_pattern("git status", "git *"));
    assert!(match_pattern("git", "git *"));
    assert!(match_pattern("git commit -m foo", "git *"));
}

#[test]
fn all_picks_the_most_specific_pattern() {
    let rules = [("*", "deny"), ("git *", "ask"), ("git status", "allow")];
    assert_eq!(all("git status", &rules).as_deref(), Some("allow"));
    assert_eq!(all("git log", &rules).as_deref(), Some("ask"));
    assert_eq!(all("echo hi", &rules).as_deref(), Some("deny"));
}

#[test]
fn all_structured_matches_command_sequences() {
    let rules = [("git *", "ask"), ("git status*", "allow")];
    assert_eq!(
        all_structured("git", &["status", "--short"], &rules).as_deref(),
        Some("allow")
    );
    assert_eq!(
        all_structured(
            "npm",
            &["run", "build", "--watch"],
            &[("npm run *", "allow")]
        )
        .as_deref(),
        Some("allow")
    );
    assert_eq!(all_structured("ls", &["-la"], &rules), None);
}

#[test]
fn all_structured_prioritizes_flag_specific_patterns() {
    let rules = [
        ("find *", "allow"),
        ("find * -delete*", "ask"),
        ("sort*", "allow"),
        ("sort -o *", "ask"),
    ];
    assert_eq!(
        all_structured("find", &["src", "-delete"], &rules).as_deref(),
        Some("ask")
    );
    assert_eq!(
        all_structured("find", &["src", "-print"], &rules).as_deref(),
        Some("allow")
    );
    assert_eq!(
        all_structured("sort", &["-o", "out.txt"], &rules).as_deref(),
        Some("ask")
    );
    assert_eq!(
        all_structured("sort", &["--reverse"], &rules).as_deref(),
        Some("allow")
    );
}

#[test]
fn all_structured_handles_sed_flags() {
    let rules = [("sed * -i*", "ask"), ("sed -n*", "allow")];
    assert_eq!(
        all_structured("sed", &["-i", "file"], &rules).as_deref(),
        Some("ask")
    );
    assert_eq!(
        all_structured("sed", &["-i.bak", "file"], &rules).as_deref(),
        Some("ask")
    );
    assert_eq!(
        all_structured("sed", &["-n", "1p", "file"], &rules).as_deref(),
        Some("allow")
    );
    assert_eq!(
        all_structured("sed", &["-i", "-n", "/./p", "myfile.txt"], &rules).as_deref(),
        Some("ask")
    );
}

#[test]
fn match_normalizes_slashes_for_cross_platform_globbing() {
    assert!(match_pattern(
        "C:\\Windows\\System32\\*",
        "C:/Windows/System32/*"
    ));
    assert!(match_pattern(
        "C:/Windows/System32/drivers",
        "C:\\Windows\\System32\\*"
    ));
}

#[test]
fn match_handles_case_sensitivity_by_platform() {
    if cfg!(windows) {
        assert!(match_pattern(
            "C:\\windows\\system32\\hosts",
            "C:/Windows/System32/*"
        ));
        assert!(match_pattern(
            "c:/windows/system32/hosts",
            "C:\\Windows\\System32\\*"
        ));
    } else {
        assert!(!match_pattern("/users/test/file", "/Users/test/*"));
    }
}
