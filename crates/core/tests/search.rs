//! Port of packages/core/test/filesystem/search.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `glob` lists files matching `**/*.ts` as relative paths and
//! `grep` filters by an include glob and reports line submatches.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use opencode_core::ripgrep::Ripgrep;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn tmp() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-search-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("tmpdir");
    dir
}

#[test]
fn globs_files_as_an_array() {
    let cwd = tmp();
    fs::create_dir_all(cwd.join("src")).expect("src");
    fs::write(cwd.join("src").join("match.ts"), "needle\n").expect("write");

    let result = Ripgrep::glob(cwd.to_str().expect("utf8"), "**/*.ts", 10).unwrap();
    let paths: Vec<&str> = result.iter().map(|item| item.path.as_str()).collect();
    assert_eq!(paths, vec!["src/match.ts"]);
}

#[test]
fn greps_files_with_include_filtering() {
    let cwd = tmp();
    fs::create_dir_all(cwd.join("src")).expect("src");
    fs::write(cwd.join("src").join("match.ts"), "needle\n").expect("write ts");
    fs::write(cwd.join("src").join("skip.txt"), "needle\n").expect("write txt");

    let result = Ripgrep::grep(cwd.to_str().expect("utf8"), "needle", "*.ts", 10).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].entry.path, "src/match.ts");
    assert_eq!(result[0].submatches[0].text, "needle");
}
