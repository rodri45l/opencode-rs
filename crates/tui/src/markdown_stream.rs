//! Streaming markdown projection.
//!
//! Derived from the observable behaviour pinned by
//! `packages/session-ui/src/components/markdown-stream.ts` and
//! `markdown-projection.ts` (upstream 18ef3cc): streaming heals incomplete
//! emphasis/links, splits open code fences, freezes completed top-level blocks,
//! keeps reference definitions with their uses, and reuses compatible pending
//! blocks while appending code deltas.

use std::fmt;

/// Error raised by the streaming projection.
#[derive(Debug, PartialEq, Eq)]
pub struct NotImplemented(pub &'static str);

impl fmt::Display for NotImplemented {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for NotImplemented {}

/// Result alias for the projection helpers.
pub type PortResult<T> = Result<T, NotImplemented>;

/// Message used by the ported tests when expecting success.
pub const NOTE: &str = "session-ui markdown stream";

/// How a projected block is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockMode {
    /// A completed top-level block.
    Full,
    /// The live tail still being streamed.
    Live,
    /// A fenced code block.
    Code,
}

/// One projected markdown block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub raw: String,
    pub src: String,
    pub mode: BlockMode,
    pub language: Option<String>,
    pub complete: bool,
}

/// A full streaming projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Projection {
    pub text: String,
    pub blocks: Vec<Block>,
}

/// Whether the text carries reference-style definitions.
fn has_refs(text: &str) -> bool {
    if !text.contains("]:") {
        return false;
    }
    let lines: Vec<&str> = text.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        let indent = line
            .bytes()
            .take_while(|byte| *byte == b' ' || *byte == b'\t')
            .count();
        if indent > 3 {
            continue;
        }
        let rest = &line[indent..];
        if !rest.starts_with('[') {
            continue;
        }
        let Some(close) = rest.find("]:") else {
            continue;
        };
        let label = &rest[1..close];
        if label.is_empty() || label.contains(']') {
            continue;
        }
        let after = rest[close + 2..].trim_start_matches([' ', '\t']);
        if !after.is_empty() {
            return true;
        }
        if let Some(next) = lines.get(index + 1) {
            let next_indent = next
                .bytes()
                .take_while(|byte| *byte == b' ' || *byte == b'\t')
                .count();
            if next_indent > 0
                && next
                    .trim_start_matches([' ', '\t'])
                    .chars()
                    .next()
                    .is_some()
            {
                return true;
            }
        }
    }
    false
}

fn heal(text: &str) -> String {
    let mut result = text.to_string();
    if let Some(open) = result.rfind("](") {
        if !result[open + 2..].contains(')') {
            if let Some(bracket) = result[..open].rfind('[') {
                let label = result[bracket + 1..open].to_string();
                if !label.contains(']') {
                    result = format!("{}{}", &result[..bracket], label);
                }
            }
        }
    }
    if result.matches('`').count() % 2 == 1 {
        result.push('`');
    }
    if result.matches("**").count() % 2 == 1 {
        result.push_str("**");
    }
    result
}

fn fence_mark(line: &str) -> Option<(char, usize)> {
    let trimmed = line.trim_start_matches([' ', '\t']);
    let indent = line.len() - trimmed.len();
    if indent > 3 {
        return None;
    }
    let first = trimmed.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }
    let size = trimmed.chars().take_while(|c| *c == first).count();
    if size < 3 {
        return None;
    }
    Some((first, size))
}

fn is_closing_fence(line: &str, character: char, size: usize) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty()
        && trimmed.chars().all(|c| c == character)
        && trimmed.chars().count() >= size
}

fn language_of(line: &str, character: char) -> Option<String> {
    let trimmed = line.trim_start_matches([' ', '\t']);
    let info = trimmed.trim_start_matches(character).trim();
    info.split_whitespace().next().map(str::to_string)
}

struct Chunk {
    start: usize,
    code: bool,
    language: Option<String>,
    code_text: String,
    open: bool,
}

fn lex(text: &str) -> Vec<Chunk> {
    let mut lines: Vec<(usize, &str)> = Vec::new();
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        lines.push((offset, line));
        offset += line.len();
    }

    let mut chunks: Vec<Chunk> = Vec::new();
    let mut index = 0usize;
    while index < lines.len() {
        let (start, line) = lines[index];
        let content = line.trim_end_matches('\n');
        if content.trim().is_empty() {
            index += 1;
            continue;
        }
        if let Some((character, size)) = fence_mark(content) {
            let language = language_of(content, character);
            let mut code_text = String::new();
            let mut open = true;
            index += 1;
            while index < lines.len() {
                let (_, current) = lines[index];
                let current_content = current.trim_end_matches('\n');
                if is_closing_fence(current_content, character, size) {
                    open = false;
                    index += 1;
                    break;
                }
                code_text.push_str(current);
                index += 1;
            }
            if !open && code_text.ends_with('\n') {
                code_text.pop();
            }
            chunks.push(Chunk {
                start,
                code: true,
                language,
                code_text,
                open,
            });
            continue;
        }
        while index < lines.len() {
            let (_, current) = lines[index];
            if current.trim_end_matches('\n').trim().is_empty() {
                break;
            }
            index += 1;
        }
        chunks.push(Chunk {
            start,
            code: false,
            language: None,
            code_text: String::new(),
            open: false,
        });
    }
    chunks
}

/// Project a streaming markdown text into ordered blocks.
pub fn stream(text: &str, live: bool) -> PortResult<Vec<Block>> {
    if !live {
        return Ok(vec![Block {
            raw: text.to_string(),
            src: text.to_string(),
            mode: BlockMode::Full,
            language: None,
            complete: false,
        }]);
    }
    if has_refs(text) {
        return Ok(vec![Block {
            raw: text.to_string(),
            src: heal(text),
            mode: BlockMode::Live,
            language: None,
            complete: false,
        }]);
    }

    let chunks = lex(text);
    if chunks.is_empty() {
        return Ok(vec![Block {
            raw: text.to_string(),
            src: heal(text),
            mode: BlockMode::Live,
            language: None,
            complete: false,
        }]);
    }

    let last_index = chunks.len() - 1;
    let mut blocks: Vec<Block> = Vec::new();
    for (index, chunk) in chunks.iter().enumerate() {
        let is_tail = index == last_index;
        let end = if is_tail {
            text.len()
        } else {
            chunks[index + 1].start
        };
        let raw = text[chunk.start..end].to_string();
        if chunk.code {
            blocks.push(Block {
                raw,
                src: chunk.code_text.clone(),
                mode: BlockMode::Code,
                language: chunk.language.clone(),
                complete: !(is_tail && chunk.open),
            });
        } else if is_tail {
            blocks.push(Block {
                src: heal(&raw),
                raw,
                mode: BlockMode::Live,
                language: None,
                complete: false,
            });
        } else {
            blocks.push(Block {
                raw: raw.clone(),
                src: raw,
                mode: BlockMode::Full,
                language: None,
                complete: false,
            });
        }
    }
    Ok(blocks)
}

fn closes_fence(raw: &str, suffix: &str) -> bool {
    let content = raw.lines().next().unwrap_or("");
    match fence_mark(content) {
        None => suffix.contains("```") || suffix.contains("~~~"),
        Some((character, size)) => {
            let mark = character.to_string().repeat(size);
            let tail: String = raw
                .chars()
                .rev()
                .take(size - 1)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            format!("{tail}{suffix}").contains(&mark)
        }
    }
}

/// Project a previous projection forward to the current text.
pub fn project(previous: Option<&Projection>, text: &str, live: bool) -> PortResult<Projection> {
    if !live {
        let current = match previous {
            Some(previous) if previous.text == text => Some(previous.clone()),
            Some(previous) if text.starts_with(&previous.text) => {
                Some(project(Some(previous), text, true)?)
            }
            _ => None,
        };
        let Some(current) = current else {
            return Ok(Projection {
                text: text.to_string(),
                blocks: vec![Block {
                    raw: text.to_string(),
                    src: text.to_string(),
                    mode: BlockMode::Full,
                    language: None,
                    complete: false,
                }],
            });
        };
        return Ok(Projection {
            text: text.to_string(),
            blocks: current
                .blocks
                .into_iter()
                .map(|block| match block.mode {
                    BlockMode::Live => Block {
                        raw: block.raw.clone(),
                        src: block.raw,
                        mode: BlockMode::Full,
                        language: None,
                        complete: false,
                    },
                    BlockMode::Code if !block.complete => Block {
                        complete: true,
                        ..block
                    },
                    _ => block,
                })
                .collect(),
        });
    }

    let Some(previous) = previous else {
        return Ok(Projection {
            text: text.to_string(),
            blocks: stream(text, true)?,
        });
    };
    if !text.starts_with(&previous.text) {
        return Ok(Projection {
            text: text.to_string(),
            blocks: stream(text, true)?,
        });
    }
    let suffix = &text[previous.text.len()..];
    let tail = previous.blocks.last();
    if suffix.is_empty()
        || tail.map(|block| block.mode) != Some(BlockMode::Code)
        || tail.map(|block| block.complete).unwrap_or(false)
        || closes_fence(
            &tail.map(|block| block.raw.clone()).unwrap_or_default(),
            suffix,
        )
    {
        return Ok(Projection {
            text: text.to_string(),
            blocks: stream(text, true)?,
        });
    }
    let tail = tail.expect("tail block");
    let mut blocks = previous.blocks[..previous.blocks.len() - 1].to_vec();
    blocks.push(Block {
        raw: format!("{}{}", tail.raw, suffix),
        src: format!("{}{}", tail.src, suffix),
        ..tail.clone()
    });
    Ok(Projection {
        text: text.to_string(),
        blocks,
    })
}

/// Whether a pending block can be reused for the next projection.
pub fn can_reuse_pending_block(current: &Block, next: &Block) -> PortResult<bool> {
    if current.mode != next.mode {
        return Ok(false);
    }
    if next.mode == BlockMode::Code || next.mode == BlockMode::Live {
        return Ok(next.raw.starts_with(&current.raw));
    }
    Ok(current.raw == next.raw)
}
