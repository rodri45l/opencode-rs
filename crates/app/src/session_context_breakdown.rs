//! Session context breakdown estimate
//! (port of packages/app/src/components/session/session-context-breakdown.ts).

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub key: String,
    pub tokens: i64,
    pub width: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub id: String,
    pub role: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextPart {
    pub text: String,
}

fn estimate_tokens(chars: usize) -> i64 {
    ((chars as f64) / 4.0).ceil() as i64
}

fn build(
    system: i64,
    user: i64,
    assistant: i64,
    tool: i64,
    other: i64,
    input: i64,
) -> Vec<Segment> {
    let entries = [
        ("system", system),
        ("user", user),
        ("assistant", assistant),
        ("tool", tool),
        ("other", other),
    ];
    entries
        .into_iter()
        .filter(|(_, tokens)| *tokens > 0)
        .map(|(key, tokens)| Segment {
            key: key.to_string(),
            tokens,
            width: tokens as f64 / input as f64 * 100.0,
        })
        .collect()
}

pub fn estimate_session_context_breakdown(
    messages: &[Message],
    parts: &BTreeMap<String, Vec<TextPart>>,
    input: i64,
    system_prompt: &str,
) -> Vec<Segment> {
    if input == 0 {
        return Vec::new();
    }

    let mut user_chars = 0usize;
    let mut assistant_chars = 0usize;

    for message in messages {
        let message_parts = parts.get(&message.id);
        if message.role == "user" {
            if let Some(message_parts) = message_parts {
                user_chars += message_parts
                    .iter()
                    .map(|part| part.text.len())
                    .sum::<usize>();
            }
        } else if message.role == "assistant" {
            if let Some(message_parts) = message_parts {
                assistant_chars += message_parts
                    .iter()
                    .map(|part| part.text.len())
                    .sum::<usize>();
            }
        }
    }

    let system = estimate_tokens(system_prompt.len());
    let user = estimate_tokens(user_chars);
    let assistant = estimate_tokens(assistant_chars);
    let tool = 0i64;
    let estimated = system + user + assistant + tool;

    if estimated <= input {
        return build(system, user, assistant, tool, input - estimated, input);
    }

    let scale = input as f64 / estimated as f64;
    let scaled_system = (system as f64 * scale).floor() as i64;
    let scaled_user = (user as f64 * scale).floor() as i64;
    let scaled_assistant = (assistant as f64 * scale).floor() as i64;
    let scaled_tool = (tool as f64 * scale).floor() as i64;
    let total = scaled_system + scaled_user + scaled_assistant + scaled_tool;
    build(
        scaled_system,
        scaled_user,
        scaled_assistant,
        scaled_tool,
        std::cmp::max(0, input - total),
        input,
    )
}
