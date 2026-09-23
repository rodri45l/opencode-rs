//! Prompt-input v2 store.
//!
//! Port of packages/session-ui/src/v2/components/prompt-input/store.ts
//! behaviour (upstream 18ef3cc), re-derived without the Solid store binding.

use crate::prompt_types::{PersistedState, PromptComment, PromptModel, PromptPart};

/// A host-neutral prompt-input store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptStore {
    state: PersistedState,
}

impl PromptStore {
    /// Create a store over the given persisted state.
    pub fn new(state: PersistedState) -> Self {
        PromptStore { state }
    }

    /// The current persisted state.
    pub fn state(&self) -> &PersistedState {
        &self.state
    }

    /// Replace the prompt and optionally the cursor.
    pub fn set_prompt(&mut self, prompt: Vec<PromptPart>, cursor: Option<usize>) {
        self.state.prompt = prompt;
        if let Some(cursor) = cursor {
            self.state.cursor = Some(cursor);
        }
    }

    /// Move the cursor.
    pub fn set_cursor(&mut self, cursor: usize) {
        self.state.cursor = Some(cursor);
    }

    /// Replace all text with a single text part, preserving attachments.
    pub fn set_text(&mut self, content: &str) {
        let mut prompt = vec![PromptPart::Text {
            content: content.to_string(),
            start: 0,
            end: content.chars().count(),
        }];
        prompt.extend(
            self.state
                .prompt
                .iter()
                .filter(|part| !matches!(part, PromptPart::Text { .. }))
                .cloned(),
        );
        self.state.prompt = prompt;
        self.state.cursor = Some(content.chars().count());
    }

    /// Insert text at the cursor without flattening structured mentions.
    pub fn add_text(&mut self, content: &str) {
        let cursor = self
            .state
            .cursor
            .unwrap_or_else(|| prompt_length(&self.state.prompt));
        self.state.prompt = insert_text(&self.state.prompt, cursor, content);
        self.state.cursor = Some(cursor + content.chars().count());
    }

    /// Reset the prompt to an empty text part.
    pub fn reset(&mut self) {
        self.state.prompt = vec![PromptPart::Text {
            content: String::new(),
            start: 0,
            end: 0,
        }];
        self.state.cursor = Some(0);
    }

    /// Set or clear the model.
    pub fn set_model(&mut self, model: Option<PromptModel>) {
        self.state.model = model;
    }

    /// Set the model variant when a model is selected.
    pub fn set_variant(&mut self, variant: Option<String>) {
        if let Some(model) = &mut self.state.model {
            model.variant = variant;
        }
    }

    /// Add a context item, ignoring duplicate keys.
    pub fn add_context(&mut self, item: PromptComment) {
        if self
            .state
            .context
            .items
            .iter()
            .any(|entry| entry.key == item.key)
        {
            return;
        }
        self.state.context.items.push(item);
    }

    /// Remove a context item by key.
    pub fn remove_context(&mut self, key: &str) {
        self.state
            .context
            .items
            .retain(|item| item.key.as_str() != key);
    }

    /// Insert a file or agent mention at the cursor.
    pub fn add_mention(&mut self, mention: PromptPart) {
        let text = prompt_text(&self.state.prompt);
        let chars = text.chars().collect::<Vec<_>>();
        let end = self.state.cursor.unwrap_or(chars.len());
        let start = chars
            .get(..end)
            .and_then(|prefix| prefix.iter().collect::<String>().rfind('@'))
            .unwrap_or(end);
        let insert_at = start;
        self.state.prompt = insert_mention(&self.state.prompt, insert_at, end, mention.clone());
        self.state.cursor = Some(insert_at + mention.content().chars().count() + 1);
    }

    /// Append an attachment part.
    pub fn add_attachment(&mut self, attachment: PromptPart) {
        self.state.prompt.push(attachment);
    }

    /// Remove an image attachment by id.
    pub fn remove_attachment(&mut self, id: &str) {
        self.state.prompt.retain(|part| match part {
            PromptPart::Image { id: part_id, .. } => part_id.as_str() != id,
            _ => true,
        });
    }
}

fn prompt_text(prompt: &[PromptPart]) -> String {
    prompt.iter().map(PromptPart::content).collect()
}

fn prompt_length(prompt: &[PromptPart]) -> usize {
    prompt
        .iter()
        .map(|part| part.content().chars().count())
        .sum()
}

fn insert_text(prompt: &[PromptPart], cursor: usize, content: &str) -> Vec<PromptPart> {
    let mut position = 0usize;
    let mut inserted = false;
    let mut parts = Vec::new();
    for part in prompt {
        if part.is_image() {
            parts.push(part.clone());
            continue;
        }
        let start = position;
        let length = part.content().chars().count();
        position += length;
        if inserted {
            parts.push(part.clone());
            continue;
        }
        if matches!(part, PromptPart::Text { .. }) && cursor >= start && cursor <= position {
            inserted = true;
            let offset = cursor - start;
            let original = part.content().chars().collect::<Vec<_>>();
            let mut updated = String::new();
            updated.extend(&original[..offset]);
            updated.push_str(content);
            updated.extend(&original[offset..]);
            parts.push(PromptPart::Text {
                content: updated,
                start: 0,
                end: 0,
            });
            continue;
        }
        if cursor > start {
            parts.push(part.clone());
            continue;
        }
        inserted = true;
        parts.push(PromptPart::Text {
            content: content.to_string(),
            start: 0,
            end: 0,
        });
        parts.push(part.clone());
    }
    if !inserted {
        parts.push(PromptPart::Text {
            content: content.to_string(),
            start: 0,
            end: 0,
        });
    }
    with_offsets(parts)
}

fn insert_mention(
    prompt: &[PromptPart],
    start: usize,
    end: usize,
    mention: PromptPart,
) -> Vec<PromptPart> {
    let mut position = 0usize;
    let mut parts = Vec::new();
    for part in prompt {
        if part.is_image() {
            parts.push(part.clone());
            continue;
        }
        let part_start = position;
        let length = part.content().chars().count();
        position += length;
        if !matches!(part, PromptPart::Text { .. }) || start < part_start || end > position {
            parts.push(part.clone());
            continue;
        }
        let chars = part.content().chars().collect::<Vec<_>>();
        let before: String = chars[..start - part_start].iter().collect();
        let after: String = chars[end - part_start..].iter().collect();
        if !before.is_empty() {
            parts.push(PromptPart::Text {
                content: before,
                start: 0,
                end: 0,
            });
        }
        parts.push(mention.clone());
        parts.push(PromptPart::Text {
            content: format!(" {after}"),
            start: 0,
            end: 0,
        });
    }
    with_offsets(parts)
}

fn with_offsets(prompt: Vec<PromptPart>) -> Vec<PromptPart> {
    let mut offset = 0usize;
    prompt
        .into_iter()
        .map(|part| {
            if part.is_image() {
                return part;
            }
            let length = part.content().chars().count();
            let next = match part {
                PromptPart::Text { content, .. } => PromptPart::Text {
                    content,
                    start: offset,
                    end: offset + length,
                },
                PromptPart::File { path, content, .. } => PromptPart::File {
                    path,
                    content,
                    start: offset,
                    end: offset + length,
                },
                PromptPart::Agent { name, content, .. } => PromptPart::Agent {
                    name,
                    content,
                    start: offset,
                    end: offset + length,
                },
                other => other,
            };
            offset += length;
            next
        })
        .collect()
}
