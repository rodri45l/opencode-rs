//! Streaming markdown block projection.
//!
//! Port of packages/session-ui/src/components/markdown-stream.ts and
//! markdown-projection.ts behaviour (upstream 18ef3cc). Only the host-neutral
//! block projection is ported; rendering is human-verified.

/// How a projected block is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockMode {
    Full,
    Live,
    Code,
}

/// A projected markdown block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub raw: String,
    pub src: String,
    pub mode: BlockMode,
    pub language: Option<String>,
    pub complete: Option<bool>,
}

/// A markdown projection with its accumulated source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Projection {
    pub text: String,
    pub blocks: Vec<Block>,
}

/// Project `text`, finalizing the live tail when `streaming` is false.
pub fn project(_previous: Option<&Projection>, text: &str, streaming: bool) -> Projection {
    let mut blocks = tokenize(text);
    let last = blocks.len().saturating_sub(1);
    for (index, block) in blocks.iter_mut().enumerate() {
        if block.mode == BlockMode::Code {
            continue;
        }
        if index == last && streaming {
            block.mode = BlockMode::Live;
            block.src = heal_live(&block.raw);
        } else {
            block.mode = BlockMode::Full;
            block.src = block.raw.clone();
        }
    }
    if !streaming {
        for block in blocks.iter_mut() {
            if block.mode == BlockMode::Code && block.complete.is_none() {
                block.complete = Some(true);
            }
        }
    }
    Projection {
        text: text.to_string(),
        blocks,
    }
}

/// Project `text` and return only its blocks.
pub fn stream(text: &str, streaming: bool) -> Vec<Block> {
    project(None, text, streaming).blocks
}

/// Whether a freshly projected block can reuse the previous pending block.
pub fn can_reuse_pending_block(previous: &Block, next: &Block) -> bool {
    previous.mode == next.mode && next.raw.starts_with(&previous.raw)
}

fn tokenize(text: &str) -> Vec<Block> {
    let mut lines: Vec<&str> = text.split_inclusive('\n').collect();
    if text.is_empty() {
        lines.clear();
    }
    let mut blocks: Vec<Block> = Vec::new();
    let mut prose: Vec<&str> = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim_start();
        if is_fence_opener(trimmed) {
            flush_prose(&mut prose, &mut blocks);
            let fence = fence_marker(trimmed);
            let mut raw = String::from(line);
            let mut code = String::new();
            let mut complete = false;
            index += 1;
            while index < lines.len() {
                let body = lines[index];
                raw.push_str(body);
                if is_fence_closer(body.trim_start(), &fence) {
                    complete = true;
                    index += 1;
                    break;
                }
                code.push_str(body);
                index += 1;
            }
            if complete && code.ends_with('\n') {
                code.pop();
                if code.ends_with('\r') {
                    code.pop();
                }
            }
            while index < lines.len() && lines[index].trim().is_empty() {
                raw.push_str(lines[index]);
                index += 1;
            }
            let language = fence_language(trimmed);
            blocks.push(Block {
                raw,
                src: code,
                mode: BlockMode::Code,
                language: if language.is_empty() {
                    None
                } else {
                    Some(language)
                },
                complete: complete.then_some(true),
            });
            continue;
        }
        if line.trim().is_empty() {
            prose.push(line);
            flush_prose(&mut prose, &mut blocks);
            index += 1;
            continue;
        }
        prose.push(line);
        index += 1;
    }
    flush_prose(&mut prose, &mut blocks);

    merge_reference_definitions(&mut blocks);
    blocks
}

fn flush_prose(prose: &mut Vec<&str>, blocks: &mut Vec<Block>) {
    if prose.is_empty() {
        return;
    }
    let raw = prose.concat();
    prose.clear();
    blocks.push(Block {
        raw: raw.clone(),
        src: raw,
        mode: BlockMode::Full,
        language: None,
        complete: None,
    });
}

fn merge_reference_definitions(blocks: &mut Vec<Block>) {
    let mut index = 0;
    while index < blocks.len() {
        if index > 0 && is_reference_definition(&blocks[index].raw) {
            let merged = blocks.remove(index);
            blocks[index - 1].raw.push_str(&merged.raw);
        } else {
            index += 1;
        }
    }
}

fn is_reference_definition(raw: &str) -> bool {
    let first = raw.lines().next().unwrap_or("").trim_start();
    if !first.starts_with('[') {
        return false;
    }
    first.contains("]:")
}

fn is_fence_opener(line: &str) -> bool {
    let marker = fence_marker(line);
    marker.len() >= 3
}

fn is_fence_closer(line: &str, opener: &str) -> bool {
    let trimmed = line.trim_end();
    if !trimmed.starts_with(opener) {
        return false;
    }
    let rest = &trimmed[opener.len()..];
    rest.chars().all(|ch| ch == opener.chars().next().unwrap())
}

fn fence_marker(line: &str) -> String {
    let ch = line.chars().next().unwrap_or(' ');
    if ch != '`' && ch != '~' {
        return String::new();
    }
    line.chars().take_while(|c| *c == ch).collect()
}

fn fence_language(line: &str) -> String {
    let marker = fence_marker(line);
    line[marker.len()..]
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string()
}

fn heal_live(raw: &str) -> String {
    let mut value = raw.to_string();
    if let Some(close) = value.find("](") {
        if let Some(open) = value[..close].rfind('[') {
            if !value[close..].contains(')') {
                let text = value[open + 1..close].to_string();
                value = format!("{}{}", &value[..open], text);
            }
        }
    }
    if value.matches("**").count() % 2 == 1 {
        value.push_str("**");
    }
    if value.matches('`').count() % 2 == 1 {
        value.push('`');
    }
    value
}
