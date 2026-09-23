//! File tools: `read`, `edit`, `apply_patch`, `glob`.
//!
//! Ports the observable behaviour of `packages/opencode/src/tool/{read,edit,
//! apply_patch,glob}.ts`: path resolution and external-directory permission,
//! binary/image handling, line/byte truncation, instruction loading, exact and
//! fuzzy replacement, unified diff metadata, and glob enumeration.

use crate::tools::{
    assert_external_directory, Attachment, ExternalDirectoryOptions, PermissionRequest,
    ToolContext, ToolError, ToolResult,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Arguments for [`ReadTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadArgs {
    /// File to read.
    pub file_path: String,
    /// 1-based line offset.
    pub offset: Option<u64>,
    /// Maximum lines to return.
    pub limit: Option<u64>,
}

/// The `read` tool.
#[derive(Debug, Default)]
pub struct ReadTool;

const DEFAULT_READ_LIMIT: usize = 2000;
const MAX_LINE_LENGTH: usize = 2000;
const MAX_LINE_SUFFIX: &str = "... (line truncated to 2000 chars)";
const MAX_BYTES: usize = 50 * 1024;
const SAMPLE_BYTES: usize = 4096;
const INSTRUCTION_FILES: [&str; 3] = ["AGENTS.md", "CLAUDE.md", "CONTEXT.md"];

impl ReadTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// Read a file or directory, honouring offset/limit and emitting metadata.
    pub fn execute(&self, args: ReadArgs, ctx: &mut ToolContext) -> Result<ToolResult, ToolError> {
        let path = resolve(&ctx.directory, &args.file_path);
        let title = relative(&ctx.directory, &path);

        let is_directory = path.is_dir();
        assert_external_directory(
            ctx,
            Some(&path.to_string_lossy()),
            &ExternalDirectoryOptions {
                kind: Some(if is_directory { "directory" } else { "file" }.to_string()),
                bypass: false,
            },
        )?;
        ctx.ask(PermissionRequest {
            permission: "read".to_string(),
            patterns: vec![title.clone()],
            always: vec!["*".to_string()],
            metadata: Value::Null,
        });

        if !path.exists() {
            return Err(ToolError::Message(format!(
                "File not found: {}",
                path.to_string_lossy()
            )));
        }

        if is_directory {
            return Ok(self.read_directory(&path, &title, &args));
        }

        let loaded = resolve_instructions(&ctx.directory, &path);

        let bytes = fs::read(&path).map_err(|error| ToolError::Message(error.to_string()))?;
        let sample = &bytes[..bytes.len().min(SAMPLE_BYTES)];
        let mime =
            sniff_attachment_mime(sample, &crate::fs_util::mime_type(&path.to_string_lossy()));

        if is_supported_image(&mime) || mime == "application/pdf" {
            let message = if mime == "application/pdf" {
                "PDF read successfully"
            } else {
                "Image read successfully"
            };
            return Ok(ToolResult {
                title,
                output: message.to_string(),
                metadata: json!({
                    "preview": message,
                    "truncated": false,
                    "loaded": loaded.iter().map(|item| item.0.clone()).collect::<Vec<_>>(),
                }),
                attachments: Some(vec![Attachment {
                    kind: "file".to_string(),
                    mime: mime.clone(),
                    url: format!("data:{mime};base64,{}", base64(&bytes)),
                }]),
            });
        }

        if is_binary_file(&path, sample) {
            return Err(ToolError::Message(format!(
                "Cannot read binary file: {}",
                path.to_string_lossy()
            )));
        }

        let text = String::from_utf8_lossy(&bytes).into_owned();
        let text = text.strip_prefix('\u{feff}').unwrap_or(&text).to_string();
        let all_lines = split_lines(&text);
        let count = all_lines.len();
        let offset = args.offset.unwrap_or(1).max(1) as usize;
        let limit = args
            .limit
            .map(|value| value as usize)
            .unwrap_or(DEFAULT_READ_LIMIT);

        if count < offset && !(count == 0 && offset == 1) {
            return Err(ToolError::Message(format!(
                "Offset {offset} is out of range for this file ({count} lines)"
            )));
        }

        let start = offset - 1;
        let mut raw: Vec<String> = Vec::new();
        let mut bytes_used = 0usize;
        let mut more = false;
        let mut cut = false;
        for line in all_lines.iter().skip(start) {
            if raw.len() >= limit {
                more = true;
                break;
            }
            let line = if line.chars().count() > MAX_LINE_LENGTH {
                let truncated: String = line.chars().take(MAX_LINE_LENGTH).collect();
                format!("{truncated}{MAX_LINE_SUFFIX}")
            } else {
                line.clone()
            };
            let size = line.len() + if raw.is_empty() { 0 } else { 1 };
            if bytes_used + size <= MAX_BYTES {
                raw.push(line);
                bytes_used += size;
            } else {
                cut = true;
                more = true;
                break;
            }
        }

        let last = offset + raw.len().saturating_sub(1);
        let next = last + 1;
        let truncated = more || cut;

        let mut output = format!(
            "<path>{}</path>\n<type>file</type>\n<content>\n",
            path.to_string_lossy()
        );
        output += &raw
            .iter()
            .enumerate()
            .map(|(index, line)| format!("{}: {line}", index + offset))
            .collect::<Vec<_>>()
            .join("\n");
        if cut {
            output += &format!(
                "\n\n(Output capped at 50 KB. Showing lines {offset}-{last}. Use offset={next} to continue.)"
            );
        } else if more {
            output += &format!(
                "\n\n(Showing lines {offset}-{last} of {count}. Use offset={next} to continue.)"
            );
        } else {
            output += &format!("\n\n(End of file - total {count} lines)");
        }
        output += "\n</content>";
        if !loaded.is_empty() {
            let joined = loaded
                .iter()
                .map(|item| item.1.clone())
                .collect::<Vec<_>>()
                .join("\n\n");
            output += &format!("\n\n<system-reminder>\n{joined}\n</system-reminder>");
        }

        Ok(ToolResult {
            title,
            output,
            metadata: json!({
                "preview": raw.iter().take(20).cloned().collect::<Vec<_>>().join("\n"),
                "truncated": truncated,
                "loaded": loaded.iter().map(|item| item.0.clone()).collect::<Vec<_>>(),
                "display": {
                    "type": "file",
                    "path": path.to_string_lossy(),
                    "text": raw.join("\n"),
                    "lineStart": offset,
                    "lineEnd": last,
                    "totalLines": count,
                    "truncated": truncated,
                },
            }),
            attachments: None,
        })
    }

    fn read_directory(&self, path: &Path, title: &str, args: &ReadArgs) -> ToolResult {
        let mut entries: Vec<String> = fs::read_dir(path)
            .map(|iter| {
                iter.flatten()
                    .map(|entry| {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if entry.path().is_dir() {
                            format!("{name}/")
                        } else {
                            name
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        entries.sort();
        let offset = args.offset.unwrap_or(1).max(1) as usize;
        let limit = args
            .limit
            .map(|value| value as usize)
            .unwrap_or(DEFAULT_READ_LIMIT);
        let start = offset - 1;
        let sliced: Vec<String> = entries.iter().skip(start).take(limit).cloned().collect();
        let truncated = start + sliced.len() < entries.len();
        let mut output = format!(
            "<path>{}</path>\n<type>directory</type>\n<entries>\n",
            path.to_string_lossy()
        );
        output += &sliced.join("\n");
        if truncated {
            output += &format!(
                "\n(Showing {} of {} entries. Use 'offset' parameter to read beyond entry {})",
                sliced.len(),
                entries.len(),
                offset + sliced.len()
            );
        } else {
            output += &format!("\n({} entries)", entries.len());
        }
        output += "\n</entries>";
        ToolResult {
            title: title.to_string(),
            output,
            metadata: json!({
                "preview": sliced.iter().take(20).cloned().collect::<Vec<_>>().join("\n"),
                "truncated": truncated,
                "loaded": [],
                "display": {
                    "type": "directory",
                    "path": path.to_string_lossy(),
                    "entries": sliced,
                    "offset": offset,
                    "totalEntries": entries.len(),
                    "truncated": truncated,
                },
            }),
            attachments: None,
        }
    }
}

/// Arguments for [`EditTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditArgs {
    /// File to edit.
    pub file_path: String,
    /// Text to replace; empty creates a new file.
    pub old_string: String,
    /// Replacement text.
    pub new_string: String,
    /// Replace every occurrence instead of exactly one.
    pub replace_all: Option<bool>,
}

/// The `edit` tool.
#[derive(Debug, Default)]
pub struct EditTool;

impl EditTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// Apply a string replacement to a file, creating it when `old_string` is empty.
    pub fn execute(&self, args: EditArgs, ctx: &mut ToolContext) -> Result<ToolResult, ToolError> {
        if args.old_string == args.new_string {
            return Err(ToolError::Message(
                "No changes to apply: oldString and newString are identical.".to_string(),
            ));
        }
        let path = resolve(&ctx.directory, &args.file_path);
        let title = relative(&ctx.directory, &path);
        assert_external_directory(
            ctx,
            Some(&path.to_string_lossy()),
            &ExternalDirectoryOptions::default(),
        )?;

        if args.old_string.is_empty() {
            if path.exists() {
                return Err(ToolError::Message(
                    "oldString cannot be empty when editing an existing file. Provide the exact text to replace, or use write for an intentional full-file replacement."
                        .to_string(),
                ));
            }
            let (bom, text) = split_bom(&args.new_string);
            ctx.ask(PermissionRequest {
                permission: "edit".to_string(),
                patterns: vec![title.clone()],
                always: vec!["*".to_string()],
                metadata: Value::Null,
            });
            write_with_dirs(&path, &join_bom(&text, bom))?;
            let diff = unified_diff(&path.to_string_lossy(), "", &text);
            return Ok(edit_result(title, diff, 0, text.lines().count(), &path));
        }

        if path.is_dir() {
            return Err(ToolError::Message(format!(
                "Path is a directory, not a file: {}",
                path.to_string_lossy()
            )));
        }
        if !path.exists() {
            return Err(ToolError::Message(format!(
                "File {} not found",
                path.to_string_lossy()
            )));
        }

        let raw =
            fs::read_to_string(&path).map_err(|error| ToolError::Message(error.to_string()))?;
        let (source_bom, content_old) = split_bom(&raw);
        let ending = if content_old.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let old = to_line_ending(&normalize_line_endings(&args.old_string), ending);
        let replacement = to_line_ending(&normalize_line_endings(&args.new_string), ending);
        let content_new = replace(
            &content_old,
            &old,
            &replacement,
            args.replace_all.unwrap_or(false),
        )?;
        let (new_bom, content_new) = split_bom(&content_new);
        let desired_bom = source_bom || new_bom;

        let diff = unified_diff(
            &path.to_string_lossy(),
            &normalize_line_endings(&content_old),
            &normalize_line_endings(&content_new),
        );
        ctx.ask(PermissionRequest {
            permission: "edit".to_string(),
            patterns: vec![title.clone()],
            always: vec!["*".to_string()],
            metadata: json!({ "filepath": path.to_string_lossy(), "diff": diff }),
        });
        write_with_dirs(&path, &join_bom(&content_new, desired_bom))?;

        let additions = diff
            .lines()
            .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
            .count();
        let deletions = diff
            .lines()
            .filter(|line| line.starts_with('-') && !line.starts_with("---"))
            .count();
        Ok(edit_result(title, diff, additions, deletions, &path))
    }
}

fn edit_result(
    title: String,
    diff: String,
    additions: usize,
    deletions: usize,
    path: &Path,
) -> ToolResult {
    ToolResult {
        title,
        output: "Edit applied successfully".to_string(),
        metadata: json!({
            "diff": diff,
            "filediff": {
                "file": path.to_string_lossy(),
                "patch": diff,
                "additions": additions,
                "deletions": deletions,
            },
        }),
        attachments: None,
    }
}

/// Arguments for [`ApplyPatchTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyPatchArgs {
    /// The freeform patch body.
    pub patch_text: String,
}

/// The `apply_patch` tool.
#[derive(Debug, Default)]
pub struct ApplyPatchTool;

impl ApplyPatchTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// Apply a freeform patch against the instance directory.
    pub fn execute(
        &self,
        args: ApplyPatchArgs,
        ctx: &mut ToolContext,
    ) -> Result<ToolResult, ToolError> {
        crate::patch::apply(&args.patch_text, ctx)
    }
}

/// Arguments for [`GlobTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobArgs {
    /// Glob pattern.
    pub pattern: String,
    /// Directory to search; defaults to the instance directory.
    pub path: Option<String>,
}

/// The `glob` tool.
#[derive(Debug, Default)]
pub struct GlobTool;

impl GlobTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// List files matching a glob pattern.
    pub fn execute(&self, args: GlobArgs, ctx: &mut ToolContext) -> Result<ToolResult, ToolError> {
        ctx.ask(PermissionRequest {
            permission: "glob".to_string(),
            patterns: vec![args.pattern.clone()],
            always: vec!["*".to_string()],
            metadata: json!({ "pattern": args.pattern, "path": args.path }),
        });

        let search = match args.path.as_deref() {
            Some(path) => resolve(&ctx.directory, path),
            None => ctx.directory.clone(),
        };
        if search.is_file() {
            return Err(ToolError::Message(format!(
                "glob path must be a directory: {}",
                search.to_string_lossy()
            )));
        }
        assert_external_directory(
            ctx,
            Some(&search.to_string_lossy()),
            &ExternalDirectoryOptions {
                kind: Some("directory".to_string()),
                bypass: false,
            },
        )?;

        let limit = 100;
        let mut files = crate::port::glob_util::scan(
            &args.pattern,
            &crate::port::glob_util::ScanOptions {
                cwd: Some(search.to_string_lossy().into_owned()),
                absolute: true,
                include: Some("file".to_string()),
                symlink: false,
                dot: true,
            },
        );
        files.sort();
        let truncated = files.len() >= limit;
        files.truncate(limit);

        let mut output = Vec::new();
        if files.is_empty() {
            output.push("No files found".to_string());
        } else {
            output.extend(files.iter().cloned());
            if truncated {
                output.push(String::new());
                output.push(
                    "(Results are truncated: showing first 100 results. Consider using a more specific path or pattern.)"
                        .to_string(),
                );
            }
        }

        Ok(ToolResult {
            title: relative(&ctx.directory, &search),
            output: output.join("\n"),
            metadata: json!({ "count": files.len(), "truncated": truncated }),
            attachments: None,
        })
    }
}

/// One selectable answer to a question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionOption {
    /// Option label.
    pub label: String,
    /// Option description.
    pub description: String,
}

/// A question posed to the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionSpec {
    /// The question text.
    pub question: String,
    /// Short header (the reference warns above 30 chars).
    pub header: String,
    /// Selectable options.
    pub options: Vec<QuestionOption>,
    /// Whether multiple options may be selected.
    pub multiple: Option<bool>,
}

/// Arguments for [`QuestionTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionArgs {
    /// Questions to ask.
    pub questions: Vec<QuestionSpec>,
}

/// The `question` tool.
#[derive(Debug, Default)]
pub struct QuestionTool;

impl QuestionTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// Ask the user the supplied questions and wait for replies.
    pub fn execute(
        &self,
        _args: QuestionArgs,
        _ctx: &mut ToolContext,
    ) -> Result<ToolResult, ToolError> {
        Err(ToolError::NotImplemented("tool::QuestionTool::execute"))
    }
}

/// Arguments for [`SkillTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillArgs {
    /// Skill name.
    pub name: String,
}

/// The `skill` tool.
#[derive(Debug, Default)]
pub struct SkillTool;

impl SkillTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// Load a skill's `SKILL.md` and reference files.
    pub fn execute(
        &self,
        _args: SkillArgs,
        _ctx: &mut ToolContext,
    ) -> Result<ToolResult, ToolError> {
        Err(ToolError::NotImplemented("tool::SkillTool::execute"))
    }
}

/// Arguments for [`LspTool`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspArgs {
    /// Operation name, e.g. `goToDefinition` or `workspaceSymbol`.
    pub operation: String,
    /// File the operation targets.
    pub file_path: String,
    /// 1-based line for position-based operations.
    pub line: Option<u64>,
    /// 1-based character for position-based operations.
    pub character: Option<u64>,
    /// Query for `workspaceSymbol`.
    pub query: Option<String>,
}

/// The `lsp` tool.
#[derive(Debug, Default)]
pub struct LspTool;

impl LspTool {
    /// Create the tool.
    pub fn new() -> Self {
        Self
    }

    /// Run an LSP operation and format the result.
    pub fn execute(&self, _args: LspArgs, _ctx: &mut ToolContext) -> Result<ToolResult, ToolError> {
        Err(ToolError::NotImplemented("tool::LspTool::execute"))
    }
}

/// The instance-relative metadata a tool attaches, kept for the ported tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruncateLimits {
    /// Maximum lines before truncation.
    pub max_lines: usize,
    /// Maximum bytes before truncation.
    pub max_bytes: usize,
}

/// The default truncation limits exposed by the `truncate` tool.
pub fn truncate_limits() -> TruncateLimits {
    TruncateLimits {
        max_lines: 2000,
        max_bytes: 50 * 1024,
    }
}

/// Placeholder metadata value so callers can assert JSON shapes.
pub fn empty_metadata() -> Value {
    Value::Null
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn resolve(directory: &Path, file_path: &str) -> PathBuf {
    let path = Path::new(file_path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        directory.join(path)
    }
}

fn relative(directory: &Path, path: &Path) -> String {
    path.strip_prefix(directory)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn base64(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn split_bom(text: &str) -> (bool, String) {
    match text.strip_prefix('\u{feff}') {
        Some(rest) => (true, rest.to_string()),
        None => (false, text.to_string()),
    }
}

fn join_bom(text: &str, bom: bool) -> String {
    if bom {
        format!("\u{feff}{text}")
    } else {
        text.to_string()
    }
}

fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n")
}

fn to_line_ending(text: &str, ending: &str) -> String {
    if ending == "\n" {
        text.to_string()
    } else {
        text.replace('\n', "\r\n")
    }
}

fn write_with_dirs(path: &Path, content: &str) -> Result<(), ToolError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| ToolError::Message(error.to_string()))?;
    }
    fs::write(path, content).map_err(|error| ToolError::Message(error.to_string()))
}

fn split_lines(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = text.split('\n').map(|line| line.to_string()).collect();
    if text.ends_with('\n') {
        lines.pop();
    }
    lines
}

fn sniff_attachment_mime(sample: &[u8], fallback: &str) -> String {
    if sample.starts_with(&[0x89, b'P', b'N', b'G']) {
        return "image/png".to_string();
    }
    if sample.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return "image/jpeg".to_string();
    }
    if sample.starts_with(b"GIF8") {
        return "image/gif".to_string();
    }
    if sample.len() >= 12 && sample.starts_with(b"RIFF") && &sample[8..12] == b"WEBP" {
        return "image/webp".to_string();
    }
    if sample.starts_with(b"%PDF") {
        return "application/pdf".to_string();
    }
    fallback.to_string()
}

fn is_supported_image(mime: &str) -> bool {
    matches!(
        mime,
        "image/jpeg" | "image/png" | "image/gif" | "image/webp"
    )
}

fn is_binary_file(path: &Path, bytes: &[u8]) -> bool {
    let extension = path
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    const BINARY_EXTENSIONS: &[&str] = &[
        "zip", "tar", "gz", "exe", "dll", "so", "class", "jar", "war", "7z", "doc", "docx", "xls",
        "xlsx", "ppt", "pptx", "odt", "ods", "odp", "bin", "dat", "obj", "o", "a", "lib", "wasm",
        "pyc", "pyo",
    ];
    if BINARY_EXTENSIONS.contains(&extension.as_str()) {
        return true;
    }
    if bytes.is_empty() {
        return false;
    }
    let mut non_printable = 0usize;
    for byte in bytes {
        if *byte == 0 {
            return true;
        }
        if *byte < 9 || (*byte > 13 && *byte < 32) {
            non_printable += 1;
        }
    }
    non_printable as f64 / bytes.len() as f64 > 0.3
}

fn resolve_instructions(root: &Path, target: &Path) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let Some(mut current) = target.parent().map(Path::to_path_buf) else {
        return results;
    };
    loop {
        for file in INSTRUCTION_FILES {
            let candidate = current.join(file);
            if candidate.exists() && candidate != target {
                if let Ok(content) = fs::read_to_string(&candidate) {
                    if !content.is_empty() {
                        results.push((
                            candidate.to_string_lossy().into_owned(),
                            format!(
                                "Instructions from: {}\n{content}",
                                candidate.to_string_lossy()
                            ),
                        ));
                    }
                }
                break;
            }
        }
        if current == root {
            break;
        }
        match current.parent() {
            Some(parent) if parent != current => current = parent.to_path_buf(),
            _ => break,
        }
    }
    results
}

/// Apply an edit replacement, mirroring the reference replacer chain.
fn replace(content: &str, old: &str, new: &str, replace_all: bool) -> Result<String, ToolError> {
    let mut not_found = true;
    for candidate in replacers(content, old) {
        let Some(index) = content.find(&candidate) else {
            continue;
        };
        not_found = false;
        if is_disproportionate(&candidate, old) {
            return Err(ToolError::Message(
                "Refusing replacement because the matched span is much larger than oldString. Re-read the file and provide the full exact oldString for the intended replacement."
                    .to_string(),
            ));
        }
        if replace_all {
            return Ok(content.replace(&candidate, new));
        }
        if content.rfind(&candidate) != Some(index) {
            continue;
        }
        let mut result = String::with_capacity(content.len());
        result.push_str(&content[..index]);
        result.push_str(new);
        result.push_str(&content[index + candidate.len()..]);
        return Ok(result);
    }
    if not_found {
        Err(ToolError::Message(
            "Could not find oldString in the file. It must match exactly, including whitespace, indentation, and line endings."
                .to_string(),
        ))
    } else {
        Err(ToolError::Message(
            "Found multiple matches for oldString. Provide more surrounding context to make the match unique."
                .to_string(),
        ))
    }
}

fn is_disproportionate(search: &str, old: &str) -> bool {
    let old_lines = old.split('\n').count();
    let search_lines = search.split('\n').count();
    if search_lines >= (old_lines + 3).max(old_lines * 2) {
        return true;
    }
    if old_lines == 1 {
        return false;
    }
    search.trim().len() > (old.trim().len() + 500).max(old.trim().len() * 4)
}

fn replacers(content: &str, find: &str) -> Vec<String> {
    let mut out = Vec::new();
    if content.contains(find) {
        out.push(find.to_string());
    }
    line_trimmed(content, find, &mut out);
    block_anchor(content, find, &mut out);
    whitespace_normalized(content, find, &mut out);
    indentation_flexible(content, find, &mut out);
    trimmed_boundary(content, find, &mut out);
    context_aware(content, find, &mut out);
    let mut start = 0;
    while let Some(index) = content[start..].find(find) {
        out.push(find.to_string());
        start += index + find.len();
    }
    out
}

fn line_trimmed(content: &str, find: &str, out: &mut Vec<String>) {
    let original: Vec<&str> = content.split('\n').collect();
    let mut search: Vec<&str> = find.split('\n').collect();
    if search.last() == Some(&"") {
        search.pop();
    }
    if search.is_empty() || search.len() > original.len() {
        return;
    }
    for i in 0..=original.len() - search.len() {
        if search
            .iter()
            .enumerate()
            .all(|(j, line)| original[i + j].trim() == line.trim())
        {
            out.push(original[i..i + search.len()].join("\n"));
        }
    }
}

fn block_anchor(content: &str, find: &str, out: &mut Vec<String>) {
    let original: Vec<&str> = content.split('\n').collect();
    let mut search: Vec<&str> = find.split('\n').collect();
    if search.len() < 3 {
        return;
    }
    if search.last() == Some(&"") {
        search.pop();
    }
    let first = search[0].trim();
    let last = search[search.len() - 1].trim();
    let block_size = search.len();
    let max_delta = (block_size as f64 * 0.25).floor().max(1.0) as usize;
    let mut candidates: Vec<(usize, usize)> = Vec::new();
    for (i, line) in original.iter().enumerate() {
        if line.trim() != first {
            continue;
        }
        for (j, candidate) in original.iter().enumerate().skip(i + 2) {
            if candidate.trim() == last {
                let actual = j - i + 1;
                if actual.abs_diff(block_size) <= max_delta {
                    candidates.push((i, j));
                }
                break;
            }
        }
    }
    if candidates.is_empty() {
        return;
    }
    let mut best: Option<(f64, (usize, usize))> = None;
    for (start, end) in &candidates {
        let actual = end - start + 1;
        let lines_to_check = (block_size - 2).min(actual - 2);
        let mut similarity = 0.0;
        if lines_to_check > 0 {
            for j in 1..block_size - 1 {
                if j >= actual - 1 {
                    break;
                }
                let a = original[start + j].trim();
                let b = search[j].trim();
                let max_len = a.chars().count().max(b.chars().count());
                if max_len == 0 {
                    continue;
                }
                similarity += 1.0 - levenshtein(a, b) as f64 / max_len as f64;
            }
            similarity /= lines_to_check as f64;
        } else {
            similarity = 1.0;
        }
        if best
            .as_ref()
            .map(|(value, _)| similarity > *value)
            .unwrap_or(true)
        {
            best = Some((similarity, (*start, *end)));
        }
    }
    if let Some((similarity, (start, end))) = best {
        if similarity >= 0.65 {
            out.push(original[start..=end].join("\n"));
        }
    }
}

fn whitespace_normalized(content: &str, find: &str, out: &mut Vec<String>) {
    let normalize = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
    let normalized_find = normalize(find);
    if normalized_find.is_empty() {
        return;
    }
    for line in content.split('\n') {
        if normalize(line) == normalized_find {
            out.push(line.to_string());
        }
    }
    let find_lines: Vec<&str> = find.split('\n').collect();
    if find_lines.len() > 1 {
        let lines: Vec<&str> = content.split('\n').collect();
        for i in 0..=lines.len().saturating_sub(find_lines.len()) {
            let block = lines[i..i + find_lines.len()].join("\n");
            if normalize(&block) == normalized_find {
                out.push(block);
            }
        }
    }
}

fn indentation_flexible(content: &str, find: &str, out: &mut Vec<String>) {
    let remove_indent = |text: &str| {
        let lines: Vec<&str> = text.split('\n').collect();
        let min = lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.len() - line.trim_start().len())
            .min()
            .unwrap_or(0);
        lines
            .iter()
            .map(|line| {
                if line.trim().is_empty() {
                    (*line).to_string()
                } else {
                    line.chars().skip(min).collect()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let normalized_find = remove_indent(find);
    let content_lines: Vec<&str> = content.split('\n').collect();
    let find_len = find.split('\n').count();
    for i in 0..=content_lines.len().saturating_sub(find_len) {
        let block = content_lines[i..i + find_len].join("\n");
        if remove_indent(&block) == normalized_find {
            out.push(block);
        }
    }
}

fn trimmed_boundary(content: &str, find: &str, out: &mut Vec<String>) {
    let trimmed = find.trim();
    if trimmed == find {
        return;
    }
    if content.contains(trimmed) {
        out.push(trimmed.to_string());
    }
    let lines: Vec<&str> = content.split('\n').collect();
    let find_len = find.split('\n').count();
    for i in 0..=lines.len().saturating_sub(find_len) {
        let block = lines[i..i + find_len].join("\n");
        if block.trim() == trimmed {
            out.push(block);
        }
    }
}

fn context_aware(content: &str, find: &str, out: &mut Vec<String>) {
    let mut find_lines: Vec<&str> = find.split('\n').collect();
    if find_lines.len() < 3 {
        return;
    }
    if find_lines.last() == Some(&"") {
        find_lines.pop();
    }
    let content_lines: Vec<&str> = content.split('\n').collect();
    let first = find_lines[0].trim();
    let last = find_lines[find_lines.len() - 1].trim();
    for i in 0..content_lines.len() {
        if content_lines[i].trim() != first {
            continue;
        }
        for j in i + 2..content_lines.len() {
            if content_lines[j].trim() == last {
                let block_lines = &content_lines[i..=j];
                if block_lines.len() == find_lines.len() {
                    let mut matching = 0;
                    let mut total = 0;
                    for k in 1..block_lines.len() - 1 {
                        let a = block_lines[k].trim();
                        let b = find_lines[k].trim();
                        if !a.is_empty() || !b.is_empty() {
                            total += 1;
                            if a == b {
                                matching += 1;
                            }
                        }
                    }
                    if total == 0 || matching as f64 / total as f64 >= 0.5 {
                        out.push(block_lines.join("\n"));
                        return;
                    }
                }
                break;
            }
        }
    }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() || b.is_empty() {
        return a.len().max(b.len());
    }
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        current[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            current[j] = (previous[j] + 1)
                .min(current[j - 1] + 1)
                .min(previous[j - 1] + cost);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()]
}

/// Build a unified diff using the common prefix/suffix of the two texts.
pub(crate) fn unified_diff(path: &str, old: &str, new: &str) -> String {
    let old_lines: Vec<&str> = old.split('\n').collect();
    let new_lines: Vec<&str> = new.split('\n').collect();
    let mut prefix = 0;
    while prefix < old_lines.len()
        && prefix < new_lines.len()
        && old_lines[prefix] == new_lines[prefix]
    {
        prefix += 1;
    }
    let mut suffix = 0;
    while suffix < old_lines.len() - prefix
        && suffix < new_lines.len() - prefix
        && old_lines[old_lines.len() - 1 - suffix] == new_lines[new_lines.len() - 1 - suffix]
    {
        suffix += 1;
    }
    let removed = &old_lines[prefix..old_lines.len() - suffix];
    let added = &new_lines[prefix..new_lines.len() - suffix];

    let mut out = String::new();
    out.push_str(&format!("Index: {path}\n"));
    out.push_str("===================================================================\n");
    out.push_str(&format!("--- {path}\n"));
    out.push_str(&format!("+++ {path}\n"));
    out.push_str(&format!(
        "@@ -{},{} +{},{} @@\n",
        prefix + 1,
        removed.len(),
        prefix + 1,
        added.len()
    ));
    for line in removed {
        out.push('-');
        out.push_str(line);
        out.push('\n');
    }
    for line in added {
        out.push('+');
        out.push_str(line);
        out.push('\n');
    }
    for line in &old_lines[old_lines.len() - suffix..] {
        out.push(' ');
        out.push_str(line);
        out.push('\n');
    }
    out
}
