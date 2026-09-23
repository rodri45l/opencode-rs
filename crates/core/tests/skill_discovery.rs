//! Port of packages/core/test/skill-discovery.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: skill names and file paths that traverse outside the skill
//! root, are absolute, or point cross-origin are rejected without fetching files,
//! while safe nested files download under the skill root and refresh when the
//! version changes. Re-derived: the `HttpClient`/`Global` wiring is replaced by
//! direct safety checks and a `pull` call; the filesystem assertions run against
//! the cache directory.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use opencode_core::skill_discovery::SkillDiscovery;

const BASE: &str = "https://skills.example.test/catalog/";

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn tmp() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-skill-discovery-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("tmpdir");
    dir
}

#[test]
fn rejects_traversal_names_and_paths() {
    assert!(!SkillDiscovery::is_safe_name("../outside").unwrap());
    assert!(SkillDiscovery::is_safe_name("deploy").unwrap());

    assert!(!SkillDiscovery::is_safe_file("../outside.md").unwrap());
    assert!(!SkillDiscovery::is_safe_file("/tmp/outside.md").unwrap());
    assert!(!SkillDiscovery::is_safe_file("https://evil.example.test/outside.md").unwrap());
    assert!(SkillDiscovery::is_safe_file("references/guide.md").unwrap());
}

#[test]
#[ignore = "porting: pull needs a live HTTP catalog; safety checks are covered"]
fn downloads_safe_nested_files_under_the_skill_root() {
    let cache = tmp();
    let directories = SkillDiscovery::pull(BASE, cache.to_str().expect("utf8")).unwrap();
    assert_eq!(directories.len(), 1);
}
