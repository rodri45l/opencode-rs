//! Port of packages/core/test/skill.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: registered sources are deduplicated in order, later
//! sources win when the same skill name appears more than once, the catalog is
//! sorted by name, and skills denied by agent permissions are filtered out of
//! the available set. Re-derived: the `SkillV2`/`AgentV2`/`SkillDiscovery`
//! wiring, the `SKILL.md` filesystem load, and URL pulling are dropped.

use opencode_core::skill::{SkillCatalog, SkillEntry, SkillSource};

const NOTE: &str = "porting: skill catalog not implemented";

fn directory(location: &str) -> SkillSource {
    SkillSource {
        kind: "directory".into(),
        location: location.into(),
    }
}

fn entry(name: &str, description: &str, location: &str) -> SkillEntry {
    SkillEntry {
        name: name.into(),
        description: Some(description.into()),
        location: location.into(),
    }
}

#[test]
#[ignore = "porting: skill catalog not implemented"]
fn registers_sources_and_deduplicates_in_order() {
    let sources = vec![
        directory("/first"),
        directory("/first"),
        directory("/second"),
    ];
    assert_eq!(
        SkillCatalog::dedup_sources(sources).expect(NOTE),
        vec![directory("/first"), directory("/second")]
    );
}

#[test]
#[ignore = "porting: skill catalog not implemented"]
fn resolves_later_source_precedence() {
    let entries = vec![
        (0, entry("foo", "foo", "/first/foo.md")),
        (0, entry("review", "First", "/first/review/SKILL.md")),
        (1, entry("review", "Second", "/second/review/SKILL.md")),
    ];
    let resolved = SkillCatalog::resolve_precedence(entries).expect(NOTE);
    assert_eq!(
        resolved
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>(),
        vec!["foo", "review"]
    );
    assert_eq!(resolved[1].description.as_deref(), Some("Second"));
    assert_eq!(resolved[1].location, "/second/review/SKILL.md");
}

#[test]
#[ignore = "porting: skill catalog not implemented"]
fn filters_skills_for_agents() {
    let entries = vec![
        entry("deploy", "Deploy production", "/deploy/SKILL.md"),
        entry("review", "Review code", "/review/SKILL.md"),
    ];
    assert_eq!(
        SkillCatalog::available(&entries, &["deploy".to_string()]).expect(NOTE),
        vec!["review".to_string()]
    );
    assert_eq!(
        SkillCatalog::available(&entries, &[]).expect(NOTE),
        vec!["deploy".to_string(), "review".to_string()]
    );
}
