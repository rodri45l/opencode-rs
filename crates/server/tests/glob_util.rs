//! Port of packages/opencode/test/util/glob.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned by the reference `Glob.scan`/`scanSync`/`match` cases:
//! pattern semantics, directory/file inclusion, dotfile handling, symlink
//! following, and absolute path output.

use opencode_server::port::glob_util::{glob_match, scan, scan_sync, ScanOptions};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

fn tmpdir() -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("oc_glob_{}_{}_{}", std::process::id(), nanos, n));
    fs::create_dir_all(&dir).expect("tmpdir");
    dir
}

fn cwd(dir: &Path) -> ScanOptions {
    ScanOptions {
        cwd: Some(dir.to_string_lossy().into_owned()),
        ..ScanOptions::default()
    }
}

#[test]
fn finds_files_matching_pattern() {
    let dir = tmpdir();
    fs::write(dir.join("a.txt"), "").unwrap();
    fs::write(dir.join("b.txt"), "").unwrap();
    fs::write(dir.join("c.md"), "").unwrap();

    let mut results = scan("*.txt", &cwd(&dir));
    results.sort();
    assert_eq!(results, vec!["a.txt", "b.txt"]);
}

#[test]
fn returns_absolute_paths_when_requested() {
    let dir = tmpdir();
    fs::write(dir.join("file.txt"), "").unwrap();

    let results = scan(
        "*.txt",
        &ScanOptions {
            absolute: true,
            ..cwd(&dir)
        },
    );
    assert_eq!(
        results[0],
        dir.join("file.txt").to_string_lossy().to_string()
    );
}

#[test]
fn excludes_directories_by_default() {
    let dir = tmpdir();
    fs::create_dir(dir.join("subdir")).unwrap();
    fs::write(dir.join("file.txt"), "").unwrap();

    assert_eq!(scan("*", &cwd(&dir)), vec!["file.txt"]);
}

#[test]
fn excludes_directories_when_include_is_file() {
    let dir = tmpdir();
    fs::create_dir(dir.join("subdir")).unwrap();
    fs::write(dir.join("file.txt"), "").unwrap();

    let options = ScanOptions {
        include: Some("file".to_string()),
        ..cwd(&dir)
    };
    assert_eq!(scan("*", &options), vec!["file.txt"]);
}

#[test]
fn includes_directories_when_include_is_all() {
    let dir = tmpdir();
    fs::create_dir(dir.join("subdir")).unwrap();
    fs::write(dir.join("file.txt"), "").unwrap();

    let options = ScanOptions {
        include: Some("all".to_string()),
        ..cwd(&dir)
    };
    let mut results = scan("*", &options);
    results.sort();
    assert_eq!(results, vec!["file.txt", "subdir"]);
}

#[test]
fn handles_nested_patterns() {
    let dir = tmpdir();
    fs::create_dir_all(dir.join("nested")).unwrap();
    fs::write(dir.join("nested").join("deep.txt"), "").unwrap();

    assert_eq!(scan("**/*.txt", &cwd(&dir)), vec!["nested/deep.txt"]);
}

#[test]
fn returns_empty_for_no_matches() {
    let dir = tmpdir();
    assert!(scan("*.nonexistent", &cwd(&dir)).is_empty());
}

#[cfg(unix)]
#[test]
fn does_not_follow_symlinks_by_default() {
    let dir = tmpdir();
    fs::create_dir(dir.join("realdir")).unwrap();
    fs::write(dir.join("realdir").join("file.txt"), "").unwrap();
    std::os::unix::fs::symlink(dir.join("realdir"), dir.join("linkdir")).unwrap();

    assert_eq!(scan("**/*.txt", &cwd(&dir)), vec!["realdir/file.txt"]);
}

#[cfg(unix)]
#[test]
fn follows_symlinks_when_requested() {
    let dir = tmpdir();
    fs::create_dir(dir.join("realdir")).unwrap();
    fs::write(dir.join("realdir").join("file.txt"), "").unwrap();
    std::os::unix::fs::symlink(dir.join("realdir"), dir.join("linkdir")).unwrap();

    let options = ScanOptions {
        symlink: true,
        ..cwd(&dir)
    };
    let mut results = scan("**/*.txt", &options);
    results.sort();
    assert_eq!(results, vec!["linkdir/file.txt", "realdir/file.txt"]);
}

#[test]
fn includes_dotfiles_when_dot_is_true() {
    let dir = tmpdir();
    fs::write(dir.join(".hidden"), "").unwrap();
    fs::write(dir.join("visible"), "").unwrap();

    let options = ScanOptions {
        dot: true,
        ..cwd(&dir)
    };
    let mut results = scan("*", &options);
    results.sort();
    assert_eq!(results, vec![".hidden", "visible"]);
}

#[test]
fn excludes_dotfiles_when_dot_is_false() {
    let dir = tmpdir();
    fs::write(dir.join(".hidden"), "").unwrap();
    fs::write(dir.join("visible"), "").unwrap();

    assert_eq!(scan("*", &cwd(&dir)), vec!["visible"]);
}

#[test]
fn scan_sync_finds_files_and_respects_options() {
    let dir = tmpdir();
    fs::write(dir.join("a.txt"), "").unwrap();
    fs::write(dir.join("b.txt"), "").unwrap();
    let mut results = scan_sync("*.txt", &cwd(&dir));
    results.sort();
    assert_eq!(results, vec!["a.txt", "b.txt"]);

    let dir2 = tmpdir();
    fs::create_dir(dir2.join("subdir")).unwrap();
    fs::write(dir2.join("file.txt"), "").unwrap();
    let options = ScanOptions {
        include: Some("all".to_string()),
        ..cwd(&dir2)
    };
    let mut all = scan_sync("*", &options);
    all.sort();
    assert_eq!(all, vec!["file.txt", "subdir"]);
}

#[test]
fn match_handles_simple_and_directory_patterns() {
    assert!(glob_match("*.txt", "file.txt"));
    assert!(!glob_match("*.txt", "file.js"));
    assert!(glob_match("**/*.js", "src/index.js"));
    assert!(!glob_match("**/*.js", "src/index.ts"));
}

#[test]
fn match_handles_dot_files() {
    assert!(glob_match(".*", ".gitignore"));
    assert!(glob_match("**/*.md", ".github/README.md"));
}

#[test]
fn match_handles_brace_expansion() {
    assert!(glob_match("*.{js,ts}", "file.js"));
    assert!(glob_match("*.{js,ts}", "file.ts"));
    assert!(!glob_match("*.{js,ts}", "file.py"));
}
