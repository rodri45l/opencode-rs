//! Port of packages/opencode/test/session/instruction.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: system paths cover the project root (and optional global
//! home), nearby subdirectory `AGENTS.md` files resolve once per message and
//! can be cleared, files already reported by read metadata are skipped, and
//! Claude Code prompt files are skipped when disabled. The remote-URL
//! (`HttpClient`) case is `test.todo` upstream and is not ported.

use opencode_server::port::session::{Instruction, InstructionError};
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "opencode-instruction-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn write(dir: &Path, relative: &str, content: &str) -> PathBuf {
    let file = dir.join(relative);
    std::fs::create_dir_all(file.parent().expect("parent")).expect("dirs");
    std::fs::write(&file, content).expect("write");
    file
}

#[test]
fn returns_empty_when_agents_md_is_at_project_root() -> Result<(), InstructionError> {
    let dir = temp_dir("root");
    let agents = write(&dir, "AGENTS.md", "# Root Instructions");
    write(&dir, "src/file.ts", "const x = 1");

    let instruction = Instruction::new(dir.clone());
    assert!(instruction
        .system_paths()
        .contains(&agents.to_string_lossy().into_owned()));

    let results = instruction.resolve(
        &[],
        &dir.join("src").join("file.ts").to_string_lossy(),
        "msg_message-test-1",
    )?;
    assert!(results.is_empty());
    Ok(())
}

#[test]
fn returns_agents_md_from_subdirectory_not_in_system_paths() -> Result<(), InstructionError> {
    let dir = temp_dir("subdir");
    let agents = write(&dir, "subdir/AGENTS.md", "# Subdir Instructions");
    write(&dir, "subdir/nested/file.ts", "const x = 1");

    let instruction = Instruction::new(dir.clone());
    assert!(!instruction
        .system_paths()
        .contains(&agents.to_string_lossy().into_owned()));

    let results = instruction.resolve(
        &[],
        &dir.join("subdir")
            .join("nested")
            .join("file.ts")
            .to_string_lossy(),
        "msg_message-test-2",
    )?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].filepath, agents.to_string_lossy().to_string());
    Ok(())
}

#[test]
fn does_not_reload_agents_md_when_reading_it_directly() -> Result<(), InstructionError> {
    let dir = temp_dir("direct");
    let agents = write(&dir, "subdir/AGENTS.md", "# Subdir Instructions");
    write(&dir, "subdir/nested/file.ts", "const x = 1");

    let instruction = Instruction::new(dir.clone());
    let results = instruction.resolve(&[], &agents.to_string_lossy(), "msg_message-test-3")?;
    assert!(results.is_empty());
    Ok(())
}

#[test]
fn does_not_reattach_same_nearby_instructions_twice_for_one_message() -> Result<(), InstructionError>
{
    let dir = temp_dir("claim");
    write(&dir, "subdir/AGENTS.md", "# Subdir Instructions");
    write(&dir, "subdir/nested/file.ts", "const x = 1");
    let file = dir.join("subdir").join("nested").join("file.ts");

    let instruction = Instruction::new(dir.clone());
    let first = instruction.resolve(&[], &file.to_string_lossy(), "msg_message-claim-1")?;
    let second = instruction.resolve(&[], &file.to_string_lossy(), "msg_message-claim-1")?;

    assert_eq!(first.len(), 1);
    assert!(second.is_empty());
    Ok(())
}

#[test]
fn clear_allows_nearby_instructions_to_be_attached_again() -> Result<(), InstructionError> {
    let dir = temp_dir("clear");
    write(&dir, "subdir/AGENTS.md", "# Subdir Instructions");
    write(&dir, "subdir/nested/file.ts", "const x = 1");
    let file = dir.join("subdir").join("nested").join("file.ts");

    let instruction = Instruction::new(dir.clone());
    let first = instruction.resolve(&[], &file.to_string_lossy(), "msg_message-claim-2")?;
    instruction.clear("msg_message-claim-2")?;
    let second = instruction.resolve(&[], &file.to_string_lossy(), "msg_message-claim-2")?;

    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    Ok(())
}

#[test]
fn skips_instructions_already_reported_by_prior_read_metadata() -> Result<(), InstructionError> {
    let dir = temp_dir("loaded");
    let agents = write(&dir, "subdir/AGENTS.md", "# Subdir Instructions");
    write(&dir, "subdir/nested/file.ts", "const x = 1");
    let file = dir.join("subdir").join("nested").join("file.ts");

    let loaded = vec![vec![agents.to_string_lossy().into_owned()]];
    let instruction = Instruction::new(dir.clone());
    let results = instruction.resolve(&loaded, &file.to_string_lossy(), "msg_message-claim-3")?;
    assert!(results.is_empty());
    Ok(())
}

#[test]
fn loads_both_project_and_global_agents_md_when_both_exist() -> Result<(), InstructionError> {
    let global = temp_dir("global");
    let project = temp_dir("project");
    let global_agents = write(&global, "AGENTS.md", "# Global Instructions");
    let project_agents = write(&project, "AGENTS.md", "# Project Instructions");

    let mut instruction = Instruction::new(project.clone());
    instruction.global_home = Some(global.clone());
    let paths = instruction.system_paths();
    assert!(paths.contains(&project_agents.to_string_lossy().into_owned()));
    assert!(paths.contains(&global_agents.to_string_lossy().into_owned()));

    let rules = instruction.system()?;
    assert_eq!(rules.len(), 2);
    assert_eq!(
        rules[0],
        format!(
            "Instructions from: {}\n# Global Instructions",
            global_agents.display()
        )
    );
    assert_eq!(
        rules[1],
        format!(
            "Instructions from: {}\n# Project Instructions",
            project_agents.display()
        )
    );
    Ok(())
}

#[test]
fn skips_project_and_global_claude_md_when_disabled() -> Result<(), InstructionError> {
    let global = temp_dir("global-claude");
    let project = temp_dir("project-claude");
    let global_claude = write(&global, ".claude/CLAUDE.md", "# Global Claude");
    let project_claude = write(&project, "CLAUDE.md", "# Project Claude");

    let mut instruction = Instruction::new(project.clone());
    instruction.global_home = Some(global.clone());
    instruction.disable_claude_code_prompt = true;

    let paths = instruction.system_paths();
    assert!(!paths.contains(&global_claude.to_string_lossy().into_owned()));
    assert!(!paths.contains(&project_claude.to_string_lossy().into_owned()));
    assert!(instruction.system()?.is_empty());
    Ok(())
}

#[test]
fn uses_global_config_agents_md() -> Result<(), InstructionError> {
    let global = temp_dir("global-config");
    let project = temp_dir("project-config");
    let global_agents = write(&global, "AGENTS.md", "# Global Instructions");

    let mut instruction = Instruction::new(project.clone());
    instruction.global_home = Some(global.clone());
    assert!(instruction
        .system_paths()
        .contains(&global_agents.to_string_lossy().into_owned()));
    Ok(())
}
