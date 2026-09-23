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

use crate::{CoreError, CoreResult};

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
    pub fn dedup_sources(_sources: Vec<SkillSource>) -> CoreResult<Vec<SkillSource>> {
        Err(CoreError::NotImplemented(
            "skill::SkillCatalog::dedup_sources",
        ))
    }

    /// Resolve skill-name precedence: later source indexes win, output sorted by
    /// name.
    pub fn resolve_precedence(_entries: Vec<(u32, SkillEntry)>) -> CoreResult<Vec<SkillEntry>> {
        Err(CoreError::NotImplemented(
            "skill::SkillCatalog::resolve_precedence",
        ))
    }

    /// The names available to an agent, excluding denied skill resources.
    pub fn available(
        _entries: &[SkillEntry],
        _denied_resources: &[String],
    ) -> CoreResult<Vec<String>> {
        Err(CoreError::NotImplemented("skill::SkillCatalog::available"))
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
