//! Transcript formatting.
//!
//! Port of packages/tui/src/util/transcript.ts behaviour (upstream 18ef3cc).
//! Locale-aware timestamps are omitted; the behavioural formatting is ported.

use serde_json::Value;

/// A provider with its display model names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub models: Vec<(String, String)>,
}

impl ProviderInfo {
    fn model_name(&self, model_id: &str) -> Option<&str> {
        self.models
            .iter()
            .find(|(id, _)| id == model_id)
            .map(|(_, name)| name.as_str())
    }
}

/// An assistant message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistantInfo {
    pub agent: String,
    pub model_id: String,
    pub provider_id: String,
    pub created: i64,
    pub completed: Option<i64>,
}

/// A user message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserInfo {
    pub agent: String,
    pub provider_id: String,
    pub model_id: String,
}

/// The body of a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageBody {
    User(UserInfo),
    Assistant(AssistantInfo),
}

/// A message with its ordering metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageInfo {
    pub id: String,
    pub created: i64,
    pub body: MessageBody,
}

/// A tool part state.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolPart {
    pub tool: String,
    pub status: String,
    pub input: Value,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// A message part.
#[derive(Debug, Clone, PartialEq)]
pub enum Part {
    Text { text: String, synthetic: bool },
    Reasoning { text: String },
    Tool(ToolPart),
}

/// Transcript formatting options.
#[derive(Debug, Clone, Default)]
pub struct TranscriptOptions {
    pub thinking: bool,
    pub tool_details: bool,
    pub assistant_metadata: bool,
    pub providers: Vec<ProviderInfo>,
}

/// Format the assistant header line.
pub fn format_assistant_header(
    message: &AssistantInfo,
    include_metadata: bool,
    providers: &[ProviderInfo],
) -> String {
    if !include_metadata {
        return "## Assistant\n\n".to_string();
    }
    let duration = match message.completed {
        Some(completed) if message.created != 0 => {
            format!("{:.1}s", (completed - message.created) as f64 / 1000.0)
        }
        _ => String::new(),
    };
    let model_name = model_name(providers, &message.provider_id, &message.model_id);
    let suffix = if duration.is_empty() {
        String::new()
    } else {
        format!(" · {duration}")
    };
    format!(
        "## Assistant ({} · {model_name}{suffix})\n\n",
        titlecase(&message.agent)
    )
}

/// Format a single part.
pub fn format_part(part: &Part, options: &TranscriptOptions) -> String {
    match part {
        Part::Text { text, synthetic } => {
            if *synthetic {
                String::new()
            } else {
                format!("{text}\n\n")
            }
        }
        Part::Reasoning { text } => {
            if options.thinking {
                format!("_Thinking:_\n\n{text}\n\n")
            } else {
                String::new()
            }
        }
        Part::Tool(tool) => {
            let mut result = format!("**Tool: {}**\n", tool.tool);
            if options.tool_details && !tool.input.is_null() {
                let pretty = serde_json::to_string_pretty(&tool.input).unwrap_or_default();
                result += &format!("\n**Input:**\n```json\n{pretty}\n```\n");
            }
            if let Some(output) = tool
                .output
                .as_deref()
                .filter(|_| options.tool_details && tool.status == "completed")
            {
                result += &format!("\n**Output:**\n```\n{output}\n```\n");
            }
            if let Some(error) = tool
                .error
                .as_deref()
                .filter(|_| options.tool_details && tool.status == "error")
            {
                result += &format!("\n**Error:**\n```\n{error}\n```\n");
            }
            result += "\n";
            result
        }
    }
}

/// Format a message and its parts.
pub fn format_message(
    message: &MessageInfo,
    parts: &[Part],
    options: &TranscriptOptions,
) -> String {
    let mut result = match &message.body {
        MessageBody::User(_) => "## User\n\n".to_string(),
        MessageBody::Assistant(assistant) => {
            format_assistant_header(assistant, options.assistant_metadata, &options.providers)
        }
    };
    for part in parts {
        result += &format_part(part, options);
    }
    result
}

/// Format a complete transcript, ordering messages by creation time.
pub fn format_transcript(
    title: &str,
    session_id: &str,
    messages: &[(MessageInfo, Vec<Part>)],
    options: &TranscriptOptions,
) -> String {
    let mut sorted: Vec<&(MessageInfo, Vec<Part>)> = messages.iter().collect();
    sorted.sort_by(|left, right| {
        left.0
            .created
            .cmp(&right.0.created)
            .then_with(|| left.0.id.cmp(&right.0.id))
    });
    let mut transcript = format!("# {title}\n\n**Session ID:** {session_id}\n\n---\n\n");
    for (message, parts) in sorted {
        transcript += &format_message(message, parts, options);
        transcript += "---\n\n";
    }
    transcript
}

fn model_name(providers: &[ProviderInfo], provider_id: &str, model_id: &str) -> String {
    providers
        .iter()
        .find(|provider| provider.id == provider_id)
        .and_then(|provider| provider.model_name(model_id))
        .unwrap_or(model_id)
        .to_string()
}

fn titlecase(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
