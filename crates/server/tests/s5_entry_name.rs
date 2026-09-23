//! Port of packages/opencode/test/config/entry-name.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `configEntryNameFromPath` strips `agent/`/`agents/`
//! prefixes, preserves nested subdirectories, normalises Windows separators,
//! falls back to the basename, and is anchored at the caller-supplied relative
//! path (regression #25713).
#![allow(dead_code)]

use opencode_server::config_entry_name::config_entry_name_from_path;

const AGENT_PREFIXES: [&str; 2] = ["agent/", "agents/"];

#[test]
fn strips_an_agents_prefix_and_returns_the_bare_name() {
    assert_eq!(
        config_entry_name_from_path("agents/build.md", &AGENT_PREFIXES),
        "build"
    );
}

#[test]
fn strips_an_agent_singular_prefix() {
    assert_eq!(
        config_entry_name_from_path("agent/build.md", &AGENT_PREFIXES),
        "build"
    );
}

#[test]
fn preserves_nested_subdirectories_in_the_key() {
    assert_eq!(
        config_entry_name_from_path("agents/team/build.md", &AGENT_PREFIXES),
        "team/build"
    );
}

#[test]
fn normalizes_windows_style_backslashes() {
    assert_eq!(
        config_entry_name_from_path("agents\\team\\build.md", &AGENT_PREFIXES),
        "team/build"
    );
}

#[test]
fn falls_back_to_basename_when_no_prefix_matches() {
    assert_eq!(
        config_entry_name_from_path("orphaned.md", &AGENT_PREFIXES),
        "orphaned"
    );
    assert_eq!(
        config_entry_name_from_path("anywhere/orphaned.md", &[]),
        "orphaned"
    );
}

#[test]
fn regression_25713_caller_passes_relative_path_parent_agent_segment_is_irrelevant() {
    // The caller passes `path.relative(dir, item)`, so the relative path is
    // always rooted at `agents/`; a `/home/agent/` parent segment never leaks in.
    let relative = "agents/build.md";
    assert_eq!(
        config_entry_name_from_path(relative, &AGENT_PREFIXES),
        "build"
    );
}

#[test]
fn regression_25713_parent_agents_segment_is_irrelevant() {
    let relative = "agents/build.md";
    assert_eq!(
        config_entry_name_from_path(relative, &AGENT_PREFIXES),
        "build"
    );
}
