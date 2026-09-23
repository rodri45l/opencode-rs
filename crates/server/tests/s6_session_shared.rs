//! Port of packages/opencode/test/cli/run/session.shared.test.ts (upstream 18ef3cc).
//!
//! RED-first: the run-session projection helpers in `cli/cmd/run/session.shared`
//! are not implemented in this crate. The reference projections are pinned
//! against local typed stubs.

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
struct Span {
    start: usize,
    end: usize,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileSource {
    path: String,
    text: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Part {
    Text {
        text: String,
        synthetic: bool,
    },
    Agent {
        name: String,
        source: Option<Span>,
    },
    File {
        mime: String,
        filename: Option<String>,
        url: String,
        source: Option<FileSource>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Message {
    role: String,
    parts: Vec<Part>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunPrompt {
    text: String,
    parts: Vec<Part>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Turn {
    prompt: RunPrompt,
    provider: String,
    model: String,
    variant: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct RunSession {
    first: bool,
    turns: Vec<Turn>,
}

fn display_width(text: &str) -> usize {
    text.chars().map(|c| if is_wide(c) { 2 } else { 1 }).sum()
}

fn is_wide(c: char) -> bool {
    matches!(
        c as u32,
        0x1100..=0x115F
            | 0x2E80..=0xA4CF
            | 0xAC00..=0xD7A3
            | 0xF900..=0xFAFF
            | 0xFE30..=0xFE4F
            | 0xFF00..=0xFF60
            | 0xFFE0..=0xFFE6
            | 0x20000..=0x3FFFD
    )
}

fn file_name(url: &str, filename: &Option<String>) -> String {
    if let Some(filename) = filename {
        return filename.clone();
    }
    if let Some(rest) = url.strip_prefix("file://") {
        if let Some(name) = rest.rsplit('/').next() {
            if !name.is_empty() {
                return name.to_string();
            }
        }
    }
    url.to_string()
}

fn message_prompt(msg: &Message) -> RunPrompt {
    let mut text: String = msg
        .parts
        .iter()
        .filter_map(|part| match part {
            Part::Text {
                text,
                synthetic: false,
            } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    let mut cursor = display_width(&text);
    let mut used: Vec<(usize, usize)> = Vec::new();
    let mut parts: Vec<Part> = Vec::new();

    let take = |text: &str, value: &str, used: &[(usize, usize)]| -> Option<Span> {
        let mut from = 0;
        while let Some(offset) = text[from..].find(value) {
            let idx = from + offset;
            let start = display_width(&text[..idx]);
            let end = start + display_width(value);
            if !used.iter().any(|(s, e)| *s < end && start < *e) {
                return Some(Span {
                    start,
                    end,
                    value: value.to_string(),
                });
            }
            from = idx + value.len();
        }
        None
    };

    for part in &msg.parts {
        match part {
            Part::File { filename, url, .. } => {
                let name = file_name(url, filename);
                let mention = format!("@{name}");
                let span = match take(&text, &mention, &used) {
                    Some(span) => span,
                    None => {
                        let gap = if text.is_empty() { "" } else { " " };
                        let start = cursor + display_width(gap);
                        text.push_str(gap);
                        text.push_str(&mention);
                        let end = start + display_width(&mention);
                        cursor = end;
                        Span {
                            start,
                            end,
                            value: mention.clone(),
                        }
                    }
                };
                used.push((span.start, span.end));
                parts.push(Part::File {
                    mime: "text/plain".to_string(),
                    filename: filename.clone(),
                    url: url.clone(),
                    source: Some(FileSource {
                        path: filename.clone().unwrap_or_else(|| url.clone()),
                        text: span,
                    }),
                });
            }
            Part::Agent { name, source } => {
                let span = match source {
                    Some(span) => span.clone(),
                    None => {
                        let mention = format!("@{name}");
                        take(&text, &mention, &used).unwrap_or_else(|| {
                            let gap = if text.is_empty() { "" } else { " " };
                            let start = cursor + display_width(gap);
                            text.push_str(gap);
                            text.push_str(&mention);
                            let end = start + display_width(&mention);
                            cursor = end;
                            Span {
                                start,
                                end,
                                value: mention.clone(),
                            }
                        })
                    }
                };
                used.push((span.start, span.end));
                parts.push(Part::Agent {
                    name: name.clone(),
                    source: Some(span),
                });
            }
            Part::Text { .. } => {}
        }
    }

    RunPrompt { text, parts }
}

fn create_session(messages: &[Message]) -> RunSession {
    RunSession {
        first: messages.is_empty(),
        turns: messages
            .iter()
            .filter(|message| message.role == "user")
            .map(|message| Turn {
                prompt: message_prompt(message),
                provider: String::new(),
                model: String::new(),
                variant: None,
            })
            .collect(),
    }
}

fn session_history(session: &RunSession) -> Vec<RunPrompt> {
    let mut out: Vec<RunPrompt> = Vec::new();
    for turn in &session.turns {
        if turn.prompt.text.trim().is_empty() {
            continue;
        }
        if out.last().map(|last| *last == turn.prompt) == Some(true) {
            continue;
        }
        out.push(turn.prompt.clone());
    }
    out
}

fn session_variant(session: &RunSession, model: &str) -> Option<String> {
    for turn in session.turns.iter().rev() {
        if turn.model == model {
            return turn.variant.clone();
        }
    }
    None
}

fn text(text: &str, synthetic: bool) -> Part {
    Part::Text {
        text: text.to_string(),
        synthetic,
    }
}

fn agent(name: &str, source: Option<(usize, usize, &str)>) -> Part {
    Part::Agent {
        name: name.to_string(),
        source: source.map(|(start, end, value)| Span {
            start,
            end,
            value: value.to_string(),
        }),
    }
}

fn file(url: &str, source: Option<FileSource>) -> Part {
    Part::File {
        mime: "text/plain".to_string(),
        filename: None,
        url: url.to_string(),
        source,
    }
}

fn user_message(parts: Vec<Part>) -> Message {
    Message {
        role: "user".to_string(),
        parts,
    }
}

fn assistant_message(parts: Vec<Part>) -> Message {
    Message {
        role: "assistant".to_string(),
        parts,
    }
}

fn turn(
    prompt_text: &str,
    parts: Vec<Part>,
    provider: &str,
    model: &str,
    variant: Option<&str>,
) -> Turn {
    Turn {
        prompt: RunPrompt {
            text: prompt_text.to_string(),
            parts,
        },
        provider: provider.to_string(),
        model: model.to_string(),
        variant: variant.map(|v| v.to_string()),
    }
}

#[test]
fn builds_user_prompt_text_from_text_file_and_agent_parts() {
    let msgs = vec![
        assistant_message(vec![text("ignore me", false)]),
        user_message(vec![
            text("look @scan", false),
            text("hidden", true),
            agent("scan", Some((5, 10, "@scan"))),
            file("file:///tmp/note.ts", None),
        ]),
    ];

    let out = create_session(&msgs);
    assert!(!out.first);
    assert_eq!(out.turns.len(), 1);
    assert_eq!(out.turns[0].prompt.text, "look @scan @note.ts");
    assert_eq!(
        out.turns[0].prompt.parts,
        vec![
            agent("scan", Some((5, 10, "@scan"))),
            file(
                "file:///tmp/note.ts",
                Some(FileSource {
                    path: "file:///tmp/note.ts".to_string(),
                    text: Span {
                        start: 11,
                        end: 19,
                        value: "@note.ts".to_string(),
                    },
                }),
            ),
        ]
    );
}

#[test]
fn reuses_existing_mentions_when_file_and_agent_parts_have_no_source() {
    let out = create_session(&[user_message(vec![
        text("look @scan @note.ts", false),
        agent("scan", None),
        file("file:///tmp/note.ts", None),
    ])]);

    assert_eq!(
        out.turns[0].prompt,
        RunPrompt {
            text: "look @scan @note.ts".to_string(),
            parts: vec![
                agent("scan", Some((5, 10, "@scan"))),
                file(
                    "file:///tmp/note.ts",
                    Some(FileSource {
                        path: "file:///tmp/note.ts".to_string(),
                        text: Span {
                            start: 11,
                            end: 19,
                            value: "@note.ts".to_string(),
                        },
                    }),
                ),
            ],
        }
    );
}

#[test]
fn dedupes_consecutive_history_entries_drops_blanks_and_copies_prompt_parts() {
    let parts = vec![agent("scan", Some((0, 5, "@scan")))];
    let session = RunSession {
        first: false,
        turns: vec![
            turn("one", parts.clone(), "openai", "gpt-5", Some("high")),
            turn("one", parts.clone(), "openai", "gpt-5", Some("high")),
            turn("   ", Vec::new(), "openai", "gpt-5", Some("high")),
            turn("two", Vec::new(), "openai", "gpt-5", None),
        ],
    };

    let out = session_history(&session);
    assert_eq!(
        out.iter().map(|i| i.text.clone()).collect::<Vec<_>>(),
        vec!["one", "two"]
    );
    assert_eq!(out[0].parts, parts);
}

#[test]
fn returns_the_latest_matching_variant_for_the_active_model() {
    let mut session = RunSession {
        first: false,
        turns: vec![
            turn("one", Vec::new(), "openai", "gpt-5", Some("high")),
            turn("two", Vec::new(), "anthropic", "sonnet", Some("max")),
            turn("three", Vec::new(), "openai", "gpt-5", None),
        ],
    };

    assert_eq!(session_variant(&session, "gpt-5"), None);

    session
        .turns
        .push(turn("four", Vec::new(), "openai", "gpt-5", Some("minimal")));
    assert_eq!(
        session_variant(&session, "gpt-5"),
        Some("minimal".to_string())
    );
}
