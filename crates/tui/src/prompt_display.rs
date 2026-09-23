//! Prompt display-width helpers.
//!
//! Port of packages/tui/src/prompt/display.ts behaviour (upstream 18ef3cc).
//! Grapheme segmentation and Unicode width are re-derived for the scripts and
//! emoji exercised by the reference tests.

#[derive(Debug, Clone, Copy)]
struct Grapheme {
    start: usize,
    end: usize,
    width: usize,
}

/// The display width of a prompt value, counting newlines as one column.
pub fn prompt_offset_width(value: &str) -> usize {
    graphemes(value).iter().map(|grapheme| grapheme.width).sum()
}

/// Slice a prompt value by display columns.
pub fn display_slice(value: &str, start: usize, end: usize) -> &str {
    let start = display_offset_index(value, start);
    let end = display_offset_index(value, end);
    &value[start..end]
}

/// The grapheme at a display column.
pub fn display_char_at(value: &str, offset: usize) -> Option<&str> {
    let mut width = 0;
    for grapheme in graphemes(value) {
        let next = width + grapheme.width;
        if offset == width || offset < next {
            return Some(&value[grapheme.start..grapheme.end]);
        }
        width = next;
    }
    None
}

/// The display column of a mention trigger `@`, if the value ends in one.
pub fn mention_trigger_index(value: &str, offset: Option<usize>) -> Option<usize> {
    let offset = offset.unwrap_or_else(|| prompt_offset_width(value));
    let text = display_slice(value, 0, offset);
    let index = text.rfind('@')?;
    let before = if index == 0 {
        None
    } else {
        text[..index].chars().next_back()
    };
    let query = &text[index..];
    if before.is_none_or(char::is_whitespace) && !query.chars().any(char::is_whitespace) {
        Some(prompt_offset_width(&text[..index]))
    } else {
        None
    }
}

fn display_offset_index(value: &str, offset: usize) -> usize {
    if offset == 0 {
        return 0;
    }
    let mut width = 0;
    for grapheme in graphemes(value) {
        let next = width + grapheme.width;
        if next > offset {
            return grapheme.start;
        }
        width = next;
    }
    value.len()
}

fn graphemes(value: &str) -> Vec<Grapheme> {
    let chars: Vec<(usize, char)> = value.char_indices().collect();
    let mut result = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let (start, ch) = chars[index];
        let mut end = start + ch.len_utf8();
        let mut width = char_width(ch);
        index += 1;
        while index < chars.len() && is_zero_width(chars[index].1) {
            end = chars[index].0 + chars[index].1.len_utf8();
            index += 1;
        }
        while index < chars.len() && chars[index].1 == '\u{200D}' {
            end = chars[index].0 + chars[index].1.len_utf8();
            index += 1;
            if index < chars.len() {
                let (next_start, next_ch) = chars[index];
                end = next_start + next_ch.len_utf8();
                width = width.max(2);
                index += 1;
                while index < chars.len() && is_zero_width(chars[index].1) {
                    end = chars[index].0 + chars[index].1.len_utf8();
                    index += 1;
                }
            }
        }
        result.push(Grapheme { start, end, width });
    }
    result
}

fn char_width(ch: char) -> usize {
    if ch == '\n' {
        return 1;
    }
    if is_wide(ch) {
        2
    } else {
        1
    }
}

fn is_zero_width(ch: char) -> bool {
    matches!(
        ch as u32,
        0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE00..=0xFE0F | 0xFE20..=0xFE2F
    )
}

fn is_wide(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1100..=0x115F
            | 0x2E80..=0x303E
            | 0x3041..=0x33FF
            | 0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xA000..=0xA4CF
            | 0xAC00..=0xD7A3
            | 0xF900..=0xFAFF
            | 0xFE30..=0xFE4F
            | 0xFF00..=0xFF60
            | 0xFFE0..=0xFFE6
            | 0x1F000..=0x1FAFF
            | 0x2600..=0x27BF
    )
}
