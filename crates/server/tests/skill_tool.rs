//! Port of packages/opencode/test/tool/skill.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the skill tool asks `skill` permission, returns a
//! `<skill_content>` block with the base directory and bundled files, and
//! preserves the typed not-found message.

use opencode_server::port::tools::{SkillArgs, SkillTool};
use opencode_server::tools::{ToolContext, ToolError, ToolResult};
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("opencode-skill-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn context(directory: &Path) -> ToolContext {
    ToolContext {
        session_id: "ses_test".to_string(),
        message_id: "msg_test".to_string(),
        agent: "build".to_string(),
        directory: directory.to_path_buf(),
        ..ToolContext::default()
    }
}

fn write_skill(dir: &Path, name: &str, description: &str) -> PathBuf {
    let skill = dir.join(".opencode").join("skill").join(name);
    std::fs::create_dir_all(skill.join("scripts")).expect("skill dirs");
    std::fs::write(
        skill.join("SKILL.md"),
        format!("---\nname: {name}\ndescription: {description}\n---\n\n# Tool Skill\n\nUse this skill.\n"),
    )
    .expect("skill md");
    std::fs::write(skill.join("scripts").join("demo.txt"), "demo").expect("reference");
    skill
}

#[test]
fn execute_returns_skill_content_block_with_files() -> Result<(), ToolError> {
    let dir = temp_dir("content");
    let skill = write_skill(&dir, "tool-skill", "Skill for tool tests.");

    let mut ctx = context(&dir);
    let result: ToolResult = SkillTool::new().execute(
        SkillArgs {
            name: "tool-skill".to_string(),
        },
        &mut ctx,
    )?;

    assert_eq!(ctx.requests.len(), 1);
    assert_eq!(ctx.requests[0].permission, "skill");
    assert!(ctx.requests[0].patterns.iter().any(|p| p == "tool-skill"));
    assert!(ctx.requests[0].always.iter().any(|p| p == "tool-skill"));
    assert_eq!(
        result.metadata["dir"].as_str(),
        Some(skill.to_string_lossy().as_ref())
    );
    assert!(result
        .output
        .contains("<skill_content name=\"tool-skill\">"));
    assert!(result.output.contains(&format!(
        "Base directory for this skill: {}",
        skill.display()
    )));
    let file = skill.join("scripts").join("demo.txt");
    assert!(result
        .output
        .contains(&format!("<file>{}</file>", file.display())));
    Ok(())
}

#[test]
fn execute_preserves_not_found_message() {
    let dir = temp_dir("missing");
    let error = SkillTool::new()
        .execute(
            SkillArgs {
                name: "missing-skill".to_string(),
            },
            &mut context(&dir),
        )
        .expect_err("missing skill");
    assert!(error
        .to_string()
        .contains("Skill \"missing-skill\" not found."));
}
