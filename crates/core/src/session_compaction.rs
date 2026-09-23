//! Session compaction prompt and tool-content serialization.
//!
//! Ports the observable behaviour of `packages/core/src/session/compaction.ts`:
//! the compaction prompt embeds the conversation (and any prior summary) and
//! asks for an anchored summary with a fixed work-state shape; tool media is
//! described without embedding base64.

use serde_json::Value;

use crate::CoreResult;

const SUMMARY_TEMPLATE: &str = r#"Output exactly the Markdown structure shown inside <template> and keep the section order unchanged. Do not include the <template> tags in your response.
<template>
## Objective
- [one or two brief sentences describing what the user is trying to accomplish]

## Important Details
- [constraints/preferences, decisions and why, important facts/assumptions, exact context needed to continue, or "(none)"]

## Work State
### Completed
- [finished work, verified facts, or changes made; otherwise "(none)"]

### Active
- [current work, partial changes, or investigation state; otherwise "(none)"]

### Blocked
- [blockers, failing commands, or unknowns; otherwise "(none)"]

## Next Move
1. [immediate concrete action, or "(none)"]
2. [next action if known, or "(none)"]

## Relevant Files
- [file or directory path: why it matters, or "(none)"]
</template>

Rules:
- Keep every section, even when empty.
- Use terse bullets, not prose paragraphs.
- Preserve exact file paths, symbols, commands, error strings, URLs, and identifiers when known.
- Do not mention the summary process or that context was compacted."#;

const SUMMARY_UPDATE_INSTRUCTIONS: &str = r#"The <prior-summary> summarizes everything that happened before the <conversation>. Construct a new summary that combines both. The <prior-summary> is discarded after this: anything you do not carry into the new summary is lost.

When combining:
- Carry forward objectives, constraints, user directives, decisions, and parallel workstreams from the <prior-summary> even when the <conversation> does not mention them. Drop only what is finished and no longer needed.
- The <conversation> is more recent than the <prior-summary>. Where they conflict, the conversation wins: state the corrected fact and drop the old claim.
- Add new progress, decisions, constraints, and context from the conversation.
- Move completed work from "Active" to "Completed".
- If a blocker has been resolved, update the summary to reflect that while keeping any details still needed to continue the work.
- Update "Objective" and "Next Move" to reflect the current work state."#;

/// Session compaction helpers.
#[derive(Debug, Default)]
pub struct SessionCompaction;

impl SessionCompaction {
    /// Build the compaction prompt.
    pub fn build_prompt(context: &[String], previous_summary: Option<&str>) -> CoreResult<String> {
        let conversation = format!(
            "Here is the conversation so far:\n\n<conversation>\n{}\n</conversation>",
            context.join("\n\n")
        );
        let prompt = match previous_summary {
            None => format!(
                "{conversation}\n\nCreate a new anchored summary from the conversation history in the <conversation> tags above so another coding agent can continue the work.\n\n{SUMMARY_TEMPLATE}"
            ),
            Some(summary) => format!(
                "{conversation}\n\nHere is the summary of the conversation before the <conversation> above:\n\n<prior-summary>\n{summary}\n</prior-summary>\n\n{SUMMARY_UPDATE_INSTRUCTIONS}\n\n{SUMMARY_TEMPLATE}"
            ),
        };
        Ok(prompt)
    }

    /// Serialize tool content parts for compaction, describing media.
    pub fn serialize_tool_content(parts: &[Value]) -> CoreResult<String> {
        let mut lines = Vec::new();
        for part in parts {
            match part.get("type").and_then(Value::as_str) {
                Some("text") => {
                    if let Some(text) = part.get("text").and_then(Value::as_str) {
                        lines.push(text.to_string());
                    }
                }
                Some("file") => {
                    let mime = part.get("mime").and_then(Value::as_str).unwrap_or("");
                    let name = part.get("name").and_then(Value::as_str);
                    lines.push(match name {
                        Some(name) => format!("[Attached {mime}: {name}]"),
                        None => format!("[Attached {mime}]"),
                    });
                }
                _ => {}
            }
        }
        Ok(lines.join("\n"))
    }
}
