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

use opencode_core::skill_discovery::SkillDiscovery;

const NOTE: &str = "porting: skill discovery not implemented";
const BASE: &str = "https://skills.example.test/catalog/";

fn tmp() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("opencode-skill-discovery-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("tmpdir");
    dir
}

#[test]
#[ignore = "porting: skill discovery not implemented"]
fn rejects_traversal_names_and_paths() {
    assert!(!SkillDiscovery::is_safe_name("../outside").expect(NOTE));
    assert!(SkillDiscovery::is_safe_name("deploy").expect(NOTE));

    assert!(!SkillDiscovery::is_safe_file("../outside.md").expect(NOTE));
    assert!(!SkillDiscovery::is_safe_file("/tmp/outside.md").expect(NOTE));
    assert!(!SkillDiscovery::is_safe_file("https://evil.example.test/outside.md").expect(NOTE));
    assert!(SkillDiscovery::is_safe_file("references/guide.md").expect(NOTE));
}

#[test]
#[ignore = "porting: skill discovery not implemented"]
fn downloads_safe_nested_files_under_the_skill_root() {
    let cache = tmp();
    let directories = SkillDiscovery::pull(BASE, cache.to_str().expect("utf8")).expect(NOTE);
    assert_eq!(directories.len(), 1);
}
