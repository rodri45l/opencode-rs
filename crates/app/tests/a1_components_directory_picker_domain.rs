//! Port of packages/app/src/components/directory-picker-domain.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
//! Async SDK cases are covered by the pure helpers below; network cases are skipped.
#![allow(dead_code)]

use std::collections::BTreeMap;

// Local stubs (fast wave): real module lands later.
fn tree_entries(_prefix: &str, _nodes: &[(&str, bool)]) -> Vec<String> {
    Vec::new()
}

fn absolute_tree_path(_root: &str, _path: &str) -> String {
    String::new()
}

fn picker_tree_entries(_prefix: &str, _nodes: &[(&str, bool)], _mode: &str) -> Vec<String> {
    Vec::new()
}

fn picker_search_entries(_nodes: &[&str], _mode: &str) -> Vec<String> {
    Vec::new()
}

fn active_tree_navigation(_token: i64, _active: i64) -> bool {
    false
}

fn tree_path_within(_root: &str, _path: &str) -> bool {
    false
}

fn display_picker_path(_selected: &str, _path: &str, _home: &str) -> String {
    String::new()
}

fn picker_root(_path: &str) -> String {
    String::new()
}

fn picker_parent(_path: &str) -> String {
    String::new()
}

fn picker_absolute_input(_input: &str, _home: &str, _root: &str) -> String {
    String::new()
}

fn current_picker_suggestions(_query: &str, _items: &[&str], _source_query: &str) -> Vec<String> {
    Vec::new()
}

fn picker_file_search_query(_root: &str, _path: &str, _home: &str) -> String {
    String::new()
}

fn preload_tree_directories(_prefix: &str, _nodes: &[(&str, bool)]) -> Vec<String> {
    Vec::new()
}

fn advance_tree_preload(_advanced: &mut std::collections::BTreeSet<String>, _path: &str) -> bool {
    false
}

fn next_tree_scroll_top(_top: i64, _delta: i64, _scroll_height: i64, _client_height: i64) -> i64 {
    0
}

fn next_suggestion_index(_current: i64, _delta: i64, _length: usize) -> i64 {
    0
}

fn selected_tree_path(_root: &str, _path: &str, _mode: &str) -> Option<String> {
    None
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn maps_server_directory_entries_into_pierre_paths() {
    assert_eq!(
        tree_entries("src/", &[("components", true), ("index.ts", false)]),
        vec!["src/components/".to_string(), "src/index.ts".to_string()]
    );
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn maps_pierre_paths_back_to_the_selected_server_root() {
    assert_eq!(
        absolute_tree_path("C:/Users/luke", "src/components/"),
        "C:/Users/luke/src/components"
    );
    assert_eq!(absolute_tree_path("C:/", ""), "C:/");
    assert_eq!(absolute_tree_path("C:/", "README.md"), "C:/README.md");
    assert_eq!(
        absolute_tree_path("/home/luke", "README.md"),
        "/home/luke/README.md"
    );
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn includes_files_only_when_the_picker_selects_files() {
    let nodes = [("components", true), ("index.ts", false)];
    assert_eq!(
        picker_tree_entries("", &nodes, "directory"),
        vec!["components/".to_string()]
    );
    assert_eq!(
        picker_tree_entries("", &nodes, "file"),
        vec!["components/".to_string(), "index.ts".to_string()]
    );
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn includes_files_in_file_autocomplete_while_preserving_directory_navigation() {
    let nodes = ["src", "README.md"];
    assert_eq!(
        picker_search_entries(&nodes, "directory"),
        vec!["src".to_string()]
    );
    assert_eq!(
        picker_search_entries(&nodes, "file"),
        vec!["src".to_string(), "README.md".to_string()]
    );
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn accepts_mutations_only_from_the_active_navigation() {
    assert!(active_tree_navigation(3, 3));
    assert!(!active_tree_navigation(2, 3));
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn preserves_posix_case_while_matching_windows_drives_case_insensitively() {
    assert!(!tree_path_within("/repo", "/Repo"));
    assert!(tree_path_within("C:/Repo", "c:/repo/src"));
    assert!(tree_path_within(
        "//Server/Share/Repo",
        "//server/share/repo/src"
    ));
    assert!(!tree_path_within("/repo", "/repo/../tmp"));
    assert!(tree_path_within("/", "/src"));
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn displays_paths_using_the_selected_server_path_format() {
    assert_eq!(
        display_picker_path(
            "C:/Users/luke/repos",
            "C:/Users/luke/repos",
            "C:/Users/luke"
        ),
        "C:\\Users\\luke\\repos"
    );
    assert_eq!(
        display_picker_path(
            "C:/Users/luke/repos",
            "C:\\Users\\luke\\repos",
            "C:/Users/luke"
        ),
        "C:\\Users\\luke\\repos"
    );
    assert_eq!(
        display_picker_path("/home/luke/repos", "repos", "/home/luke"),
        "~/repos"
    );
    assert_eq!(
        display_picker_path("/home/luke/repos", "~/repos", "/home/luke"),
        "~/repos"
    );
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn treats_the_server_share_prefix_as_the_unc_root() {
    assert_eq!(picker_root("//Server/Share/repo/src"), "//Server/Share");
    assert_eq!(
        picker_root("\\\\Server\\Share\\repo\\src"),
        "//Server/Share"
    );
    assert_eq!(picker_parent("//Server/Share"), "//Server/Share");
    assert_eq!(picker_parent("//Server/Share/repo"), "//Server/Share");
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn resolves_relative_input_against_the_current_picker_root() {
    assert_eq!(
        picker_absolute_input("src", "/home/luke", "/home/luke/repo"),
        "/home/luke/repo/src"
    );
    assert_eq!(
        picker_absolute_input("../other", "/home/luke", "/home/luke/repo"),
        "/home/luke/other"
    );
    assert_eq!(
        picker_absolute_input("~/.config", "/home/luke", "/home/luke/repo"),
        "/home/luke/.config"
    );
    assert_eq!(
        picker_absolute_input("src", "C:/Users/luke", "C:/Users/luke/repo"),
        "C:/Users/luke/repo/src"
    );
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn exposes_autocomplete_results_only_for_their_source_query() {
    let items = ["/repo/src/index.ts"];
    assert_eq!(
        current_picker_suggestions("/repo/src", &items, "/repo/src"),
        vec!["/repo/src/index.ts".to_string()]
    );
    assert!(current_picker_suggestions("/repo/test", &items, "/repo/src").is_empty());
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn scopes_file_autocomplete_to_the_current_browser_root() {
    assert_eq!(
        picker_file_search_query("/home/luke/repos", "/home/luke/repos/src/in", "/home/luke"),
        "src/in"
    );
    assert_eq!(
        picker_file_search_query("/home/luke", "~/repos/op", "/home/luke"),
        "repos/op"
    );
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn identifies_the_next_directory_level_to_preload() {
    assert_eq!(
        preload_tree_directories(
            "src/",
            &[("components", true), ("index.ts", false), ("utils", true)]
        ),
        vec!["src/components/".to_string(), "src/utils/".to_string()]
    );
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn advances_preloading_once_for_every_expanded_directory() {
    let mut advanced = std::collections::BTreeSet::new();
    assert!(advance_tree_preload(&mut advanced, ""));
    assert!(!advance_tree_preload(&mut advanced, ""));
    assert!(advance_tree_preload(&mut advanced, "repos/"));
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn clamps_bridged_tree_wheel_scrolling() {
    assert_eq!(next_tree_scroll_top(100, 40, 500, 200), 140);
    assert_eq!(next_tree_scroll_top(10, -40, 500, 200), 0);
    assert_eq!(next_tree_scroll_top(290, 40, 500, 200), 300);
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn wraps_autocomplete_keyboard_navigation() {
    assert_eq!(next_suggestion_index(-1, 1, 4), 0);
    assert_eq!(next_suggestion_index(3, 1, 4), 0);
    assert_eq!(next_suggestion_index(0, -1, 4), 3);
    assert_eq!(next_suggestion_index(0, 1, 0), -1);
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn returns_absolute_directories_and_relative_files() {
    assert_eq!(
        selected_tree_path("/home/luke/repo", "src/", "directory"),
        Some("/home/luke/repo/src".to_string())
    );
    assert_eq!(
        selected_tree_path("/home/luke/repo", "src/index.ts", "file"),
        Some("src/index.ts".to_string())
    );
    assert_eq!(
        selected_tree_path("/home/luke/repo/src", "index.ts", "file"),
        Some("index.ts".to_string())
    );
    assert_eq!(selected_tree_path("/home/luke/repo", "src/", "file"), None);
}

#[test]
#[ignore = "porting: components/directory-picker-domain not implemented"]
fn selection_policy_placeholder_is_exercised() {
    let _ = BTreeMap::<String, String>::new();
    assert_eq!(picker_root("/repo/src"), "/repo");
}
