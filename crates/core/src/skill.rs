//! Skill source catalog (re-derived behavioural subset).
//!
//! Ports the observable behaviour of `packages/core/src/skill.ts` and
//! `skill/discovery.ts`: registered sources are deduplicated in order, later
//! sources win when the same skill name appears more than once, the catalog is
//! sorted by name, and skills denied by agent permissions are filtered out of
//! the available set. The `SkillV2`/`AgentV2`/`SkillDiscovery` service wiring,
//! the `SKILL.md` filesystem load, and URL pulling are dropped; the pure catalog
//! decisions remain.

use std::collections::BTreeMap;

use crate::CoreResult;

/// A registered skill source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillSource {
    /// The source kind (`directory` or `url`).
    pub kind: String,
    /// The directory path or URL.
    pub location: String,
}

/// A resolved skill entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillEntry {
    /// The skill name.
    pub name: String,
    /// The optional description.
    pub description: Option<String>,
    /// The skill location.
    pub location: String,
}

/// The skill catalog helpers.
#[derive(Debug, Default)]
pub struct SkillCatalog;

impl SkillCatalog {
    /// Deduplicate sources, preserving first-seen order.
    pub fn dedup_sources(sources: Vec<SkillSource>) -> CoreResult<Vec<SkillSource>> {
        let mut seen: Vec<SkillSource> = Vec::new();
        for source in sources {
            if !seen.contains(&source) {
                seen.push(source);
            }
        }
        Ok(seen)
    }

    /// Resolve skill-name precedence: later source indexes win, output sorted by
    /// name.
    pub fn resolve_precedence(entries: Vec<(u32, SkillEntry)>) -> CoreResult<Vec<SkillEntry>> {
        Ok(dedup_by_name(entries)
            .into_values()
            .map(|(_, entry)| entry)
            .collect())
    }

    /// The names available to an agent, excluding denied skill resources.
    pub fn available(
        entries: &[SkillEntry],
        denied_resources: &[String],
    ) -> CoreResult<Vec<String>> {
        Ok(entries
            .iter()
            .filter(|entry| !denied_resources.iter().any(|denied| denied == &entry.name))
            .map(|entry| entry.name.clone())
            .collect())
    }
}

/// Group entries by name, keeping the highest source index.
pub fn dedup_by_name(entries: Vec<(u32, SkillEntry)>) -> BTreeMap<String, (u32, SkillEntry)> {
    let mut map = BTreeMap::new();
    for (index, entry) in entries {
        match map.get(&entry.name) {
            Some((existing, _)) if *existing >= index => {}
            _ => {
                map.insert(entry.name.clone(), (index, entry));
            }
        }
    }
    map
}
