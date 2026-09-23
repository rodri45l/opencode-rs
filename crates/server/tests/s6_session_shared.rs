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

fn create_session(_messages: &[Message]) -> RunSession {
    RunSession::default()
}

fn session_history(session: &RunSession) -> Vec<RunPrompt> {
    session.turns.iter().map(|t| t.prompt.clone()).collect()
}

fn session_variant(_session: &RunSession, _model: &str) -> Option<String> {
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
#[ignore = "porting: cli run session shared not implemented"]
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
#[ignore = "porting: cli run session shared not implemented"]
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
#[ignore = "porting: cli run session shared not implemented"]
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
#[ignore = "porting: cli run session shared not implemented"]
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
