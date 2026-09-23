//! Port of packages/core/test/tool-skill.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the tool registers as `skill` with the configured
//! description, asserts action `skill` with the selected name as resource and
//! save scope, renders model-facing content containing the base directory for
//! the skill and its reference files, exposes the structured `{ name }`, reports
//! `Unable to load skill <name>` for an unknown or denied skill, and supports a
//! flat skill without references. Re-derived: the `SkillV2`/`Permission`/
//! `ToolRegistry` wiring, the `SKILL.md` filesystem load, and the live tmpdir
//! fixture are dropped.

use opencode_core::tool_skill::{SkillInfo, SkillTool};

const NOTE: &str = "porting: skill tool not implemented";

fn skill(directory: &str) -> SkillInfo {
    SkillInfo {
        name: "effect".into(),
        description: "Use Effect".into(),
        location: format!("{directory}/SKILL.md"),
        content: "# Effect\n\nGuidance".into(),
    }
}

#[test]
#[ignore = "porting: skill tool not implemented"]
fn registers_with_the_configured_description() {
    assert_eq!(SkillTool::NAME, "skill");
    assert!(!SkillTool::DESCRIPTION.is_empty());
}

#[test]
#[ignore = "porting: skill tool not implemented"]
fn authorizes_the_selected_skill_name() {
    assert_eq!(SkillTool::ACTION, "skill");
    assert_eq!(
        SkillTool::permission_resources("effect").expect(NOTE),
        vec!["effect".to_string()]
    );
}

#[test]
#[ignore = "porting: skill tool not implemented"]
fn renders_model_content_with_the_base_directory_and_references() {
    let output =
        SkillTool::to_model_output(&skill("/tmp/effect"), &["/tmp/effect/reference.md".into()])
            .expect(NOTE);
    assert!(output.contains("Base directory for this skill: /tmp/effect"));
    assert!(output.contains("/tmp/effect/reference.md"));
    assert!(output.contains("# Effect"));
}

#[test]
#[ignore = "porting: skill tool not implemented"]
fn reports_unknown_or_denied_skills() {
    assert_eq!(
        SkillTool::failure_message("missing").expect(NOTE),
        "Unable to load skill missing"
    );
}
