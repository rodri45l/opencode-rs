//! Bounded tool output truncation.
//!
//! Ports the observable behaviour of `packages/opencode/src/tool/truncate.ts`:
//! output is capped by line count and byte count, per-call options override the
//! configured limits, head/tail directions are supported, and truncated output
//! is written to a managed file with a Grep (and optionally Task) hint.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// Default maximum preview lines.
pub const MAX_LINES: usize = 2000;
/// Default maximum preview bytes.
pub const MAX_BYTES: usize = 50 * 1024;

static COUNTER: AtomicU64 = AtomicU64::new(1);

/// Which end of the output to keep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Keep the beginning.
    Head,
    /// Keep the end.
    Tail,
}

/// Per-call truncation overrides.
#[derive(Debug, Clone, Copy, Default)]
pub struct TruncateOptions {
    /// Maximum lines to keep.
    pub max_lines: Option<usize>,
    /// Maximum bytes to keep.
    pub max_bytes: Option<usize>,
    /// Which end to keep.
    pub direction: Option<Direction>,
}

/// Effective truncation limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TruncateLimits {
    /// Maximum preview lines.
    pub max_lines: usize,
    /// Maximum preview bytes.
    pub max_bytes: usize,
}

/// The bounded output plus the managed file it was written to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruncateResult {
    /// The bounded preview (plus guidance when truncated).
    pub content: String,
    /// Whether truncation occurred.
    pub truncated: bool,
    /// Managed file path holding the full output, when truncated.
    pub output_path: Option<String>,
}

/// Bounded tool output truncation.
#[derive(Debug, Clone)]
pub struct Truncate {
    limits: TruncateLimits,
    dir: PathBuf,
}

impl Default for Truncate {
    fn default() -> Self {
        Self::new()
    }
}

impl Truncate {
    /// Default maximum preview lines.
    pub const MAX_LINES: usize = MAX_LINES;
    /// Default maximum preview bytes.
    pub const MAX_BYTES: usize = MAX_BYTES;

    /// Create a truncator with the default limits.
    pub fn new() -> Self {
        Self {
            limits: TruncateLimits {
                max_lines: MAX_LINES,
                max_bytes: MAX_BYTES,
            },
            dir: std::env::temp_dir().join("opencode-tool-output"),
        }
    }

    /// Create a truncator with configured limits.
    pub fn with_limits(limits: TruncateLimits) -> Self {
        Self {
            limits,
            dir: std::env::temp_dir().join("opencode-tool-output"),
        }
    }

    /// The effective limits.
    pub fn limits(&self) -> TruncateLimits {
        self.limits
    }

    /// Truncate `content`, without a Task hint.
    pub fn output(&self, content: &str, options: TruncateOptions) -> TruncateResult {
        self.output_with_task(content, options, false)
    }

    /// Truncate `content`, including the Task tool hint when `task_allowed`.
    pub fn output_with_task(
        &self,
        content: &str,
        options: TruncateOptions,
        task_allowed: bool,
    ) -> TruncateResult {
        let max_lines = options.max_lines.unwrap_or(self.limits.max_lines);
        let max_bytes = options.max_bytes.unwrap_or(self.limits.max_bytes);
        let direction = options.direction.unwrap_or(Direction::Head);

        let line_count = content.split('\n').count();
        let byte_len = content.len();

        let preview = if line_count > max_lines {
            let lines: Vec<&str> = content.split('\n').collect();
            let kept = match direction {
                Direction::Head => lines[..max_lines].join("\n"),
                Direction::Tail => lines[line_count - max_lines..].join("\n"),
            };
            format!("{kept}\n...{} lines truncated...", line_count - max_lines)
        } else if byte_len > max_bytes {
            let kept = match direction {
                Direction::Head => prefix_at_boundary(content, max_bytes),
                Direction::Tail => suffix_at_boundary(content, max_bytes),
            };
            format!("{kept}\n...{} bytes truncated...", byte_len - max_bytes)
        } else {
            return TruncateResult {
                content: content.to_string(),
                truncated: false,
                output_path: None,
            };
        };

        let path = self
            .dir
            .join(format!("tool_{}", COUNTER.fetch_add(1, Ordering::Relaxed)));
        let output_path = if std::fs::create_dir_all(&self.dir)
            .and_then(|_| std::fs::write(&path, content))
            .is_ok()
        {
            Some(path.to_string_lossy().into_owned())
        } else {
            None
        };

        let mut guidance = String::from(
            "\n\nThe tool call succeeded but the output was truncated. \
             Use Grep to search the full content or Read with offset/limit to view specific sections.",
        );
        if let Some(output_path) = &output_path {
            guidance.push_str(&format!(" Full output saved to: {output_path}"));
        }
        if task_allowed {
            guidance.push_str(" Or use the Task tool to process the full output.");
        }

        TruncateResult {
            content: format!("{preview}{guidance}"),
            truncated: true,
            output_path,
        }
    }
}

fn prefix_at_boundary(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_string()
}

fn suffix_at_boundary(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_string();
    }
    let mut start = text.len() - max_bytes;
    while start < text.len() && !text.is_char_boundary(start) {
        start += 1;
    }
    text[start..].to_string()
}
