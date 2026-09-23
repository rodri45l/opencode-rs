//! Port of packages/opencode/test/skill/skill.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `Skill.fmt` renders XML-safe verbose locations and the
//! "no skills" fallback; discovery finds and skips skills across the project,
//! `.claude/skills`, and `.agents/skills` directories and honours the disable
//! flags; the typed missing/invalid/mismatch errors keep their identity.
//! (The remote discovery pull is a different module and is not ported here.)

use opencode_server::port::skill::{
    format_skills, SkillCatalog, SkillError, SkillInfo, SkillNotFound,
};
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-skill-catalog-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn write_skill(dir: &std::path::Path, relative_dir: &str, name: &str, description: Option<&str>) {
    let skill_dir = dir.join(relative_dir).join(name);
    std::fs::create_dir_all(&skill_dir).expect("skill dir");
    let frontmatter = match description {
        Some(description) => format!("---\nname: {name}\ndescription: {description}\n---\n"),
        None => format!("---\nname: {name}\n---\n"),
    };
    std::fs::write(
        skill_dir.join("SKILL.md"),
        format!("{frontmatter}\n# {name}\n"),
    )
    .expect("skill md");
}

#[test]
fn formats_verbose_locations_as_xml_safe_filesystem_paths() {
    let skills = vec![
        SkillInfo {
            name: "tagged-skill".to_string(),
            description: Some("A tagged skill.".to_string()),
            location: "/tmp/plugin.git#v1.3.0/SKILL.md".to_string(),
            content: String::new(),
        },
        SkillInfo {
            name: "built-in-skill".to_string(),
            description: Some("A built-in skill.".to_string()),
            location: "<built-in>".to_string(),
            content: String::new(),
        },
    ];

    let output = format_skills(&skills, true);
    assert!(output.contains("<location>/tmp/plugin.git#v1.3.0/SKILL.md</location>"));
    assert!(output.contains("<location>&lt;built-in&gt;</location>"));
    assert!(!output.contains("file://"));
    assert!(!output.contains("%23"));
}

#[test]
fn no_skills_fallback_is_used_when_nothing_is_available() {
    assert_eq!(
        format_skills(&[], false),
        "No skills are currently available."
    );
    assert_eq!(
        format_skills(&[], true),
        "No skills are currently available."
    );
}

#[test]
fn exposes_tagged_expected_skill_failure_classes() {
    let invalid = SkillError::Invalid {
        path: "/tmp/SKILL.md".to_string(),
        message: "Invalid skill frontmatter".to_string(),
    };
    let mismatch = SkillError::NameMismatch {
        path: "/tmp/SKILL.md".to_string(),
        expected: "expected-skill".to_string(),
        actual: "actual-skill".to_string(),
    };

    assert!(matches!(invalid, SkillError::Invalid { .. }));
    assert!(matches!(mismatch, SkillError::NameMismatch { .. }));
}

#[test]
#[ignore = "porting: skill.discovery not implemented"]
fn discovers_skills_from_the_opencode_skill_directory() -> Result<(), SkillError> {
    let dir = temp_dir("opencode");
    write_skill(
        &dir,
        ".opencode/skill",
        "test-skill",
        Some("A test skill for verification."),
    );

    let skills = SkillCatalog::new(dir.clone()).all()?;
    let list: Vec<&SkillInfo> = skills
        .iter()
        .filter(|skill| skill.location != "<built-in>")
        .collect();
    assert_eq!(list.len(), 1);
    let item = list
        .iter()
        .find(|skill| skill.name == "test-skill")
        .expect("skill");
    assert_eq!(
        item.description.as_deref(),
        Some("A test skill for verification.")
    );
    assert!(item.location.contains("skill/test-skill/SKILL.md"));
    Ok(())
}

#[test]
#[ignore = "porting: skill.discovery not implemented"]
fn returns_skill_directories() -> Result<(), SkillError> {
    let dir = temp_dir("dirs");
    write_skill(
        &dir,
        ".opencode/skill",
        "dir-skill",
        Some("Skill for dirs test."),
    );

    let dirs = SkillCatalog::new(dir.clone()).dirs()?;
    assert!(dirs.iter().any(|entry| entry.ends_with("skill/dir-skill")));
    assert_eq!(dirs.len(), 1);
    Ok(())
}

#[test]
#[ignore = "porting: skill.discovery not implemented"]
fn skips_skills_with_missing_frontmatter() -> Result<(), SkillError> {
    let dir = temp_dir("no-frontmatter");
    let skill_dir = dir.join(".opencode/skill/no-frontmatter");
    std::fs::create_dir_all(&skill_dir).expect("dirs");
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "# No Frontmatter\n\nJust some content without YAML frontmatter.\n",
    )
    .expect("md");

    let skills = SkillCatalog::new(dir.clone()).all()?;
    let list: Vec<&SkillInfo> = skills
        .iter()
        .filter(|skill| skill.location != "<built-in>")
        .collect();
    assert!(list.is_empty());
    Ok(())
}

#[test]
#[ignore = "porting: skill.discovery not implemented"]
fn discovers_skills_without_descriptions() -> Result<(), SkillError> {
    let dir = temp_dir("no-description");
    write_skill(&dir, ".opencode/skill", "manual-skill", None);

    let skills = SkillCatalog::new(dir.clone()).all()?;
    let list: Vec<SkillInfo> = skills
        .into_iter()
        .filter(|skill| skill.location != "<built-in>")
        .collect();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].description, None);
    assert_eq!(
        format_skills(&list, false),
        "No skills are currently available."
    );
    Ok(())
}

#[test]
#[ignore = "porting: skill.discovery not implemented"]
fn discovers_skills_from_the_claude_skills_directory() -> Result<(), SkillError> {
    let dir = temp_dir("claude");
    write_skill(
        &dir,
        ".claude/skills",
        "claude-skill",
        Some("A skill in the .claude/skills directory."),
    );

    let skills = SkillCatalog::new(dir.clone()).all()?;
    let list: Vec<&SkillInfo> = skills
        .iter()
        .filter(|skill| skill.location != "<built-in>")
        .collect();
    assert_eq!(list.len(), 1);
    assert!(list[0]
        .location
        .contains(".claude/skills/claude-skill/SKILL.md"));
    Ok(())
}

#[test]
#[ignore = "porting: skill.discovery not implemented"]
fn discovers_skills_from_the_agents_skills_directory() -> Result<(), SkillError> {
    let dir = temp_dir("agents");
    write_skill(
        &dir,
        ".agents/skills",
        "agent-skill",
        Some("A skill in the .agents/skills directory."),
    );

    let skills = SkillCatalog::new(dir.clone()).all()?;
    let list: Vec<&SkillInfo> = skills
        .iter()
        .filter(|skill| skill.location != "<built-in>")
        .collect();
    assert_eq!(list.len(), 1);
    assert!(list[0]
        .location
        .contains(".agents/skills/agent-skill/SKILL.md"));
    Ok(())
}

#[test]
#[ignore = "porting: skill.discovery not implemented"]
fn skips_claude_code_skills_when_disabled() -> Result<(), SkillError> {
    let dir = temp_dir("disable-claude");
    write_skill(
        &dir,
        ".claude/skills",
        "claude-skill",
        Some("Claude skill."),
    );
    write_skill(&dir, ".agents/skills", "agent-skill", Some("Agent skill."));

    let mut catalog = SkillCatalog::new(dir.clone());
    catalog.disable_claude_code_skills = true;
    let names: Vec<String> = catalog
        .all()?
        .into_iter()
        .filter(|skill| skill.location != "<built-in>")
        .map(|skill| skill.name)
        .collect();
    assert_eq!(names, ["agent-skill".to_string()]);
    Ok(())
}

#[test]
#[ignore = "porting: skill.discovery not implemented"]
fn skips_external_skill_directories_when_disabled() -> Result<(), SkillError> {
    let dir = temp_dir("disable-external");
    write_skill(
        &dir,
        ".claude/skills",
        "claude-skill",
        Some("Claude skill."),
    );
    write_skill(&dir, ".agents/skills", "agent-skill", Some("Agent skill."));
    write_skill(
        &dir,
        ".opencode/skill",
        "opencode-skill",
        Some("OpenCode skill."),
    );

    let mut catalog = SkillCatalog::new(dir.clone());
    catalog.disable_external_skills = true;
    let names: Vec<String> = catalog
        .all()?
        .into_iter()
        .filter(|skill| skill.location != "<built-in>")
        .map(|skill| skill.name)
        .collect();
    assert_eq!(names, ["opencode-skill".to_string()]);
    Ok(())
}

#[test]
#[ignore = "porting: skill.discovery not implemented"]
fn returns_empty_when_no_skills_exist() -> Result<(), SkillError> {
    let dir = temp_dir("empty");
    let skills = SkillCatalog::new(dir.clone()).all()?;
    let list: Vec<&SkillInfo> = skills
        .iter()
        .filter(|skill| skill.location != "<built-in>")
        .collect();
    assert!(list.is_empty());
    Ok(())
}

#[test]
#[ignore = "porting: skill.discovery not implemented"]
fn fails_with_typed_error_when_requiring_a_missing_skill() {
    let dir = temp_dir("missing");
    let error = SkillCatalog::new(dir.clone())
        .require("missing-skill")
        .expect_err("missing skill");
    assert_eq!(
        error,
        SkillError::NotFound(SkillNotFound {
            name: "missing-skill".to_string(),
        })
    );
    assert!(error
        .to_string()
        .contains("Skill \"missing-skill\" not found."));
}
