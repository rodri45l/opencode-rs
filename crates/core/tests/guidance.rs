//! Port of packages/core/test/skill/guidance.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: described, permitted skills are rendered into the
//! `<available_skills>` guidance block; undescribed skills are omitted; ordered
//! permission rules apply last-match-wins; guidance is omitted entirely when the
//! agent denies all skills without a later specific allow; and an empty skill
//! list reports that no skills are available. Re-derived: the `SystemContext`
//! reconciliation and Effect layers are dropped.

use opencode_core::guidance::{SkillGuidance, SkillInfo, SkillPermission};

const NOTE: &str = "porting: skill guidance not implemented";

fn skill(name: &str, description: Option<&str>) -> SkillInfo {
    SkillInfo {
        name: name.to_string(),
        description: description.map(str::to_string),
        content: format!("{name} guidance"),
        location: format!("/skills/{name}/SKILL.md"),
    }
}

fn permission(resource: &str, effect: &str) -> SkillPermission {
    SkillPermission {
        resource: resource.to_string(),
        effect: effect.to_string(),
    }
}

#[test]
fn renders_described_permitted_skills() {
    let skills = [
        skill("hidden", None),
        skill("denied", Some("Must not be advertised")),
        skill("effect", Some("Build applications with Effect")),
    ];
    let permissions = [permission("denied", "deny")];

    let rendered = SkillGuidance::render(&skills, &permissions).expect(NOTE);

    assert!(rendered
        .starts_with("Skills provide specialized instructions and workflows for specific tasks."));
    assert!(rendered.contains("<available_skills>"));
    assert!(rendered.contains("<name>effect</name>"));
    assert!(rendered.contains("<description>Build applications with Effect</description>"));
    assert!(!rendered.contains("hidden"));
    assert!(!rendered.contains("denied"));
}

#[test]
fn omits_guidance_when_the_agent_denies_all_skills() {
    let permissions = [permission("*", "deny")];
    assert_eq!(
        SkillGuidance::render(&[skill("effect", Some("Build with Effect"))], &permissions)
            .expect(NOTE),
        ""
    );
}

#[test]
fn omits_guidance_when_a_resource_denial_follows_a_global_denial() {
    let permissions = [permission("*", "deny"), permission("hidden", "deny")];
    assert_eq!(
        SkillGuidance::render(&[skill("effect", Some("Build with Effect"))], &permissions)
            .expect(NOTE),
        ""
    );
}

#[test]
fn retains_specifically_allowed_skills_after_a_global_denial() {
    let permissions = [permission("*", "deny"), permission("effect", "allow")];
    let rendered =
        SkillGuidance::render(&[skill("effect", Some("Build with Effect"))], &permissions)
            .expect(NOTE);
    assert!(rendered.contains("<name>effect</name>"));
}

#[test]
fn omits_guidance_when_an_allowed_skill_is_denied_again() {
    let permissions = [
        permission("*", "deny"),
        permission("effect", "allow"),
        permission("effect", "deny"),
    ];
    assert_eq!(
        SkillGuidance::render(&[skill("effect", Some("Build with Effect"))], &permissions)
            .expect(NOTE),
        ""
    );
}

#[test]
fn reports_no_skills_when_the_list_is_empty() {
    let rendered = SkillGuidance::render(&[], &[]).expect(NOTE);
    assert!(rendered.contains("No skills are currently available."));
}

#[test]
fn availability_follows_last_match_wins() {
    assert!(SkillGuidance::is_available(
        "effect",
        &[permission("*", "deny"), permission("effect", "allow")]
    )
    .expect(NOTE));
    assert!(!SkillGuidance::is_available(
        "other",
        &[permission("*", "deny"), permission("effect", "allow")]
    )
    .expect(NOTE));
    assert!(!SkillGuidance::is_available(
        "effect",
        &[
            permission("*", "deny"),
            permission("effect", "allow"),
            permission("effect", "deny"),
        ]
    )
    .expect(NOTE));
}
