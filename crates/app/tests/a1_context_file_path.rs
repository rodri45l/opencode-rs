//! Port of packages/app/src/context/file/path.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/app/src/context/file/path.ts.

use opencode_app::context_file_path::{
    create_path_helpers, encode_file_path, strip_query_and_hash, unquote_git_path,
};

#[test]
fn normalizes_file_inputs_against_workspace_root() {
    let path = create_path_helpers("/repo");
    assert_eq!(
        path.normalize("file:///repo/src/app.ts?x=1#h"),
        "src/app.ts"
    );
    assert_eq!(path.normalize("/repo/src/app.ts"), "src/app.ts");
    assert_eq!(path.normalize("./src/app.ts"), "src/app.ts");
    assert_eq!(path.normalize_dir("src/components///"), "src/components");
    assert_eq!(path.tab("src/app.ts"), "file://src/app.ts");
    assert_eq!(
        path.path_from_tab("file://src/app.ts"),
        Some("src/app.ts".to_string())
    );
    assert_eq!(path.path_from_tab("other://src/app.ts"), None);
}

#[test]
fn normalizes_windows_absolute_paths_with_mixed_separators() {
    let path = create_path_helpers("C:\\repo");
    assert_eq!(path.normalize("C:\\repo\\src\\app.ts"), "src\\app.ts");
    assert_eq!(path.normalize("C:/repo/src/app.ts"), "src/app.ts");
    assert_eq!(path.normalize("file://C:/repo/src/app.ts"), "src/app.ts");
    assert_eq!(path.normalize("c:\\repo\\src\\app.ts"), "src\\app.ts");
}

#[test]
fn normalizes_windows_directory_separators() {
    let path = create_path_helpers("C:\\repo");
    assert_eq!(path.normalize_dir("frontend\\"), "frontend");
    assert_eq!(path.normalize_dir("frontend\\src\\"), "frontend/src");
    assert_eq!(path.normalize_dir("C:\\repo\\frontend\\"), "frontend");
}

#[test]
fn normalizes_separators_for_windows_roots_written_with_forward_slashes() {
    let path = create_path_helpers("C:/repo");
    assert_eq!(path.normalize_dir("frontend\\src\\"), "frontend/src");
}

#[test]
fn normalizes_separators_for_windows_unc_roots() {
    let path = create_path_helpers("\\\\server\\share");
    assert_eq!(
        path.normalize_dir("\\\\server\\share\\frontend\\"),
        "frontend"
    );
}

#[test]
fn preserves_backslashes_in_posix_directory_names() {
    let path = create_path_helpers("/repo");
    assert_eq!(path.normalize_dir("literal\\name\\"), "literal\\name\\");
    assert_eq!(path.normalize_dir("literal\\name/"), "literal\\name");
}

#[test]
fn keeps_query_hash_stripping_behavior_stable() {
    assert_eq!(strip_query_and_hash("a/b.ts#L12?x=1"), "a/b.ts");
    assert_eq!(strip_query_and_hash("a/b.ts?x=1#L12"), "a/b.ts");
    assert_eq!(strip_query_and_hash("a/b.ts"), "a/b.ts");
}

#[test]
fn unquotes_git_escaped_octal_path_strings() {
    assert_eq!(unquote_git_path("\"a/\\303\\251.txt\""), "a/\u{00e9}.txt");
    assert_eq!(unquote_git_path("\"plain\\nname\""), "plain\nname");
    assert_eq!(unquote_git_path("a/b/c.ts"), "a/b/c.ts");
}

#[test]
fn encode_file_path_handles_absolute_relative_and_special_characters() {
    let cases: &[(&str, &str)] = &[
        (
            "/home/user/project/README.md",
            "/home/user/project/README.md",
        ),
        (
            "/home/user/file#name with spaces.txt",
            "/home/user/file%23name%20with%20spaces.txt",
        ),
        ("src/components/App.tsx", "src/components/App.tsx"),
        ("/", "/"),
        (
            "/Users/kelvin/Projects/opencode/README.md",
            "/Users/kelvin/Projects/opencode/README.md",
        ),
        (
            "/Users/kelvin/My Documents/file.txt",
            "/Users/kelvin/My%20Documents/file.txt",
        ),
        (
            "D:\\dev\\projects\\opencode\\README.bs.md",
            "/D:/dev/projects/opencode/README.bs.md",
        ),
        (
            "D:\\dev\\projects\\opencode/README.bs.md",
            "/D:/dev/projects/opencode/README.bs.md",
        ),
        (
            "C:\\Program Files\\MyApp\\file with spaces.txt",
            "/C:/Program%20Files/MyApp/file%20with%20spaces.txt",
        ),
        ("C:\\", "/C:/"),
        ("src\\components\\App.tsx", "src/components/App.tsx"),
        ("c:\\users\\test\\file.txt", "/c:/users/test/file.txt"),
        ("/usr/local/bin/app", "/usr/local/bin/app"),
        ("", ""),
        ("//path//to///file.txt", "//path//to///file.txt"),
        ("/D:/path/file.txt", "/D:/path/file.txt"),
        ("D:", "/D:"),
        ("C:\\Users\\test\\", "/C:/Users/test/"),
        ("/path/to/file#name.txt", "/path/to/file%23name.txt"),
        ("/path/to/file?name.txt", "/path/to/file%3Fname.txt"),
        ("/path/to/file%name.txt", "/path/to/file%25name.txt"),
        ("/var/log/app.log", "/var/log/app.log"),
    ];
    for (input, expected) in cases {
        assert_eq!(&encode_file_path(input), expected, "input={input}");
    }
}

#[test]
fn encode_file_path_encodes_unicode() {
    let result = encode_file_path("/home/user/文档/README.md");
    assert!(result.contains("%E6%96%87%E6%A1%A3"));
}

#[test]
fn encode_file_path_handles_paths_with_dots_and_long_paths() {
    let with_dots = encode_file_path("C:\\Users\\..\\test\\.\\file.txt");
    assert!(with_dots.contains(".."));
    assert!(with_dots.contains("/./"));

    let long = format!(
        "C:\\Users\\test\\{}file.txt",
        "verylongdirectoryname\\".repeat(20)
    );
    assert!(!encode_file_path(&long).contains('\\'));
}
