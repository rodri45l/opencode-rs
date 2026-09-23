//! Skill discovery and formatting.
//!
//! Ports the observable behaviour of `packages/opencode/src/skill/index.ts`
//! (`Skill.fmt`, discovery directories, and the typed missing-skill error).

use std::path::PathBuf;

/// A discovered skill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillInfo {
    /// Skill name.
    pub name: String,
    /// Skill description, omitted when the frontmatter has none.
    pub description: Option<String>,
    /// Absolute `SKILL.md` path, or `<built-in>`.
    pub location: String,
    /// Skill body.
    pub content: String,
}

/// A typed missing-skill failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillNotFound {
    /// The requested skill name.
    pub name: String,
}

impl std::fmt::Display for SkillNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Skill \"{}\" not found.", self.name)
    }
}

impl std::error::Error for SkillNotFound {}

/// A typed skill-discovery failure (invalid frontmatter or name mismatch).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillError {
    /// The behaviour has not been ported yet.
    NotImplemented(&'static str),
    /// The requested skill does not exist.
    NotFound(SkillNotFound),
    /// The `SKILL.md` frontmatter is invalid.
    Invalid {
        /// File path.
        path: String,
        /// Diagnostic.
        message: String,
    },
    /// The frontmatter name does not match the directory name.
    NameMismatch {
        /// File path.
        path: String,
        /// Name derived from the path.
        expected: String,
        /// Name declared in frontmatter.
        actual: String,
    },
}

impl std::fmt::Display for SkillError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(what) => write!(f, "not implemented: {what}"),
            Self::NotFound(error) => error.fmt(f),
            Self::Invalid { path, message } => write!(f, "{path}: {message}"),
            Self::NameMismatch {
                path,
                expected,
                actual,
            } => write!(
                f,
                "{path}: skill name mismatch: expected {expected}, found {actual}"
            ),
        }
    }
}

impl std::error::Error for SkillError {}

/// Format skills for the system prompt.
pub fn format_skills(skills: &[SkillInfo], verbose: bool) -> String {
    let parts: Vec<String> = skills
        .iter()
        .filter_map(|skill| {
            let description = skill.description.as_deref()?;
            Some(if verbose {
                format!(
                    "<skill name=\"{}\">\n<location>{}</location>\n{description}\n</skill>",
                    xml_escape(&skill.name),
                    xml_escape(&skill.location),
                )
            } else {
                let name = &skill.name;
                format!("{name}: {description}")
            })
        })
        .collect();

    if parts.is_empty() {
        return "No skills are currently available.".to_string();
    }
    parts.join("\n")
}

fn xml_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// The skill catalog for a project.
#[derive(Debug, Clone)]
pub struct SkillCatalog {
    /// Project directory.
    pub directory: PathBuf,
    /// Global home directory, when configured.
    pub home: Option<PathBuf>,
    /// Skip `.claude/skills`.
    pub disable_claude_code_skills: bool,
    /// Skip `.claude/skills` and `.agents/skills`.
    pub disable_external_skills: bool,
}

impl SkillCatalog {
    /// Create a catalog rooted at `directory`.
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
            home: None,
            disable_claude_code_skills: false,
            disable_external_skills: false,
        }
    }

    /// All discovered skills, including built-ins.
    pub fn all(&self) -> Result<Vec<SkillInfo>, SkillError> {
        Err(SkillError::NotImplemented("skill::all"))
    }

    /// The directories that contained a `SKILL.md`.
    pub fn dirs(&self) -> Result<Vec<String>, SkillError> {
        Err(SkillError::NotImplemented("skill::dirs"))
    }

    /// Require a skill by name.
    pub fn require(&self, name: &str) -> Result<SkillInfo, SkillError> {
        Err(SkillError::NotFound(SkillNotFound {
            name: name.to_string(),
        }))
    }
}
