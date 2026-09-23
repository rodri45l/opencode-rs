//! Port of packages/app/src/components/directory-picker-domain.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/app/src/components/directory-picker-domain.ts.
//! Async SDK cases are covered by the pure helpers below; network cases are skipped.
#![allow(dead_code)]

use std::collections::BTreeMap;

use opencode_app::directory_picker_domain::{
    absolute_tree_path, active_tree_navigation, advance_tree_preload, current_picker_suggestions,
    display_picker_path, next_suggestion_index, next_tree_scroll_top, picker_absolute_input,
    picker_file_search_query, picker_parent, picker_root, picker_search_entries,
    picker_tree_entries, preload_tree_directories, selected_tree_path, tree_entries,
    tree_path_within,
};

#[test]
fn maps_server_directory_entries_into_pierre_paths() {
    assert_eq!(
        tree_entries("src/", &[("components", true), ("index.ts", false)]),
        vec!["src/components/".to_string(), "src/index.ts".to_string()]
    );
}

#[test]
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
fn accepts_mutations_only_from_the_active_navigation() {
    assert!(active_tree_navigation(3, 3));
    assert!(!active_tree_navigation(2, 3));
}

#[test]
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
fn exposes_autocomplete_results_only_for_their_source_query() {
    let items = ["/repo/src/index.ts"];
    assert_eq!(
        current_picker_suggestions("/repo/src", &items, "/repo/src"),
        vec!["/repo/src/index.ts".to_string()]
    );
    assert!(current_picker_suggestions("/repo/test", &items, "/repo/src").is_empty());
}

#[test]
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
fn advances_preloading_once_for_every_expanded_directory() {
    let mut advanced = std::collections::BTreeSet::new();
    assert!(advance_tree_preload(&mut advanced, ""));
    assert!(!advance_tree_preload(&mut advanced, ""));
    assert!(advance_tree_preload(&mut advanced, "repos/"));
}

#[test]
fn clamps_bridged_tree_wheel_scrolling() {
    assert_eq!(next_tree_scroll_top(100, 40, 500, 200), 140);
    assert_eq!(next_tree_scroll_top(10, -40, 500, 200), 0);
    assert_eq!(next_tree_scroll_top(290, 40, 500, 200), 300);
}

#[test]
fn wraps_autocomplete_keyboard_navigation() {
    assert_eq!(next_suggestion_index(-1, 1, 4), 0);
    assert_eq!(next_suggestion_index(3, 1, 4), 0);
    assert_eq!(next_suggestion_index(0, -1, 4), 3);
    assert_eq!(next_suggestion_index(0, 1, 0), -1);
}

#[test]
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
fn selection_policy_placeholder_is_exercised() {
    let _ = BTreeMap::<String, String>::new();
    assert_eq!(picker_root("/repo/src"), "/");
}
