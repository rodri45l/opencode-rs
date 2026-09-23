//! Port of packages/app/src/components/prompt-input/build-request-parts.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct Selection {
    start_line: i64,
    start_char: i64,
    end_line: i64,
    end_char: i64,
}

#[derive(Clone, Debug, PartialEq)]
enum Prompt {
    Text {
        content: String,
        start: i64,
        end: i64,
    },
    File {
        path: String,
        content: String,
        start: i64,
        end: i64,
        selection: Option<Selection>,
        mime: Option<String>,
        filename: Option<String>,
    },
    Agent {
        name: String,
        content: String,
        start: i64,
        end: i64,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct ContextItem {
    key: String,
    kind: String,
    path: String,
    comment: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct ImageAttachment {
    id: String,
    filename: String,
    mime: String,
    data_url: String,
    source_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
enum RequestPart {
    Text {
        text: String,
        synthetic: bool,
        comment: Option<String>,
    },
    File {
        url: String,
        filename: Option<String>,
        mime: Option<String>,
        source_path: Option<String>,
        source_text: Option<String>,
    },
    Agent {
        name: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct OptimisticPart {
    session_id: String,
    message_id: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct BuildRequestPartsResult {
    request_parts: Vec<RequestPart>,
    optimistic_parts: Vec<OptimisticPart>,
}

#[derive(Clone, Debug, Default)]
struct BuildRequestPartsInput {
    prompt: Vec<Prompt>,
    context: Vec<ContextItem>,
    images: Vec<ImageAttachment>,
    text: String,
    message_id: String,
    session_id: String,
    session_directory: String,
}

// Local stub (fast wave): real module lands later.
fn build_request_parts(_input: BuildRequestPartsInput) -> BuildRequestPartsResult {
    BuildRequestPartsResult::default()
}

fn text(content: &str, start: i64, end: i64) -> Prompt {
    Prompt::Text {
        content: content.into(),
        start,
        end,
    }
}

#[test]
#[ignore = "porting: components/prompt-input/build-request-parts not implemented"]
fn builds_typed_request_and_optimistic_parts_without_cast_path() {
    let input = BuildRequestPartsInput {
        prompt: vec![
            text("hello", 0, 5),
            Prompt::File {
                path: "src/foo.ts".into(),
                content: "@src/foo.ts".into(),
                start: 5,
                end: 16,
                selection: Some(Selection {
                    start_line: 4,
                    start_char: 1,
                    end_line: 6,
                    end_char: 1,
                }),
                mime: None,
                filename: None,
            },
            Prompt::Agent {
                name: "planner".into(),
                content: "@planner".into(),
                start: 16,
                end: 24,
            },
        ],
        context: vec![ContextItem {
            key: "ctx:1".into(),
            kind: "file".into(),
            path: "src/bar.ts".into(),
            comment: Some("check this".into()),
        }],
        images: vec![ImageAttachment {
            id: "img_1".into(),
            filename: "a.png".into(),
            mime: "image/png".into(),
            data_url: "data:image/png;base64,AAA".into(),
            source_path: None,
        }],
        text: "hello @src/foo.ts @planner".into(),
        message_id: "msg_1".into(),
        session_id: "ses_1".into(),
        session_directory: "/repo".into(),
    };

    let result = build_request_parts(input);

    assert!(matches!(
        result.request_parts.first(),
        Some(RequestPart::Text { .. })
    ));
    assert!(result
        .request_parts
        .iter()
        .any(|part| matches!(part, RequestPart::Agent { .. })));
    assert!(result.request_parts.iter().any(|part| match part {
        RequestPart::File { url, .. } => url.starts_with("file:///repo/src/foo.ts"),
        _ => false,
    }));
    assert!(result.request_parts.iter().any(|part| matches!(
        part,
        RequestPart::Text {
            synthetic: true,
            ..
        }
    )));
    assert!(result.request_parts.iter().any(|part| matches!(
        part,
        RequestPart::Text {
            synthetic: true,
            comment: Some(comment),
            ..
        } if comment == "check this"
    )));

    assert_eq!(result.optimistic_parts.len(), result.request_parts.len());
    assert!(result
        .optimistic_parts
        .iter()
        .all(|part| part.session_id == "ses_1" && part.message_id == "msg_1"));
}

#[test]
#[ignore = "porting: components/prompt-input/build-request-parts not implemented"]
fn keeps_multiple_uploaded_attachments_in_order() {
    let input = BuildRequestPartsInput {
        prompt: vec![text("check these", 0, 11)],
        context: Vec::new(),
        images: vec![
            ImageAttachment {
                id: "img_1".into(),
                filename: "a.png".into(),
                mime: "image/png".into(),
                data_url: "data:image/png;base64,AAA".into(),
                source_path: None,
            },
            ImageAttachment {
                id: "img_2".into(),
                filename: "b.pdf".into(),
                mime: "application/pdf".into(),
                data_url: "data:application/pdf;base64,BBB".into(),
                source_path: None,
            },
        ],
        text: "check these".into(),
        message_id: "msg_multi".into(),
        session_id: "ses_multi".into(),
        session_directory: "/repo".into(),
    };

    let result = build_request_parts(input);
    let files: Vec<Option<String>> = result
        .request_parts
        .iter()
        .filter_map(|part| match part {
            RequestPart::File { url, filename, .. } if url.starts_with("data:") => {
                Some(filename.clone())
            }
            _ => None,
        })
        .collect();

    assert_eq!(files.len(), 2);
    assert_eq!(
        files,
        vec![Some("a.png".to_string()), Some("b.pdf".to_string())]
    );
}

#[test]
#[ignore = "porting: components/prompt-input/build-request-parts not implemented"]
fn preserves_an_external_attachment_source_path_for_the_model() {
    let input = BuildRequestPartsInput {
        prompt: Vec::new(),
        context: Vec::new(),
        images: vec![ImageAttachment {
            id: "img_external".into(),
            filename: "opencode.global.dat".into(),
            source_path: Some(
                "C:\\Users\\Luke\\AppData\\Roaming\\ai.opencode.desktop.beta\\opencode.global.dat"
                    .into(),
            ),
            mime: "text/plain".into(),
            data_url: "data:text/plain;base64,AAA".into(),
        }],
        text: "inspect this".into(),
        message_id: "msg_external".into(),
        session_id: "ses_external".into(),
        session_directory: "C:\\Repos\\sst\\opencode".into(),
    };

    let result = build_request_parts(input);
    let filename = result.request_parts.iter().find_map(|part| match part {
        RequestPart::File { filename, .. } => filename.clone(),
        _ => None,
    });

    assert_eq!(
        filename,
        Some(
            "C:\\Users\\Luke\\AppData\\Roaming\\ai.opencode.desktop.beta\\opencode.global.dat"
                .to_string()
        )
    );
}

#[test]
#[ignore = "porting: components/prompt-input/build-request-parts not implemented"]
fn preserves_reference_aliases_as_directory_file_parts() {
    let input = BuildRequestPartsInput {
        prompt: vec![Prompt::File {
            path: "/repo/../docs".into(),
            content: "@docs".into(),
            start: 0,
            end: 5,
            selection: None,
            mime: Some("application/x-directory".into()),
            filename: Some("docs".into()),
        }],
        context: Vec::new(),
        images: Vec::new(),
        text: "@docs".into(),
        message_id: "msg_reference".into(),
        session_id: "ses_reference".into(),
        session_directory: "/repo/app".into(),
    };

    let result = build_request_parts(input);
    let file = result.request_parts.iter().find_map(|part| match part {
        RequestPart::File {
            url,
            filename,
            mime,
            source_path,
            source_text,
        } => Some((
            url.clone(),
            filename.clone(),
            mime.clone(),
            source_path.clone(),
            source_text.clone(),
        )),
        _ => None,
    });

    let (url, filename, mime, source_path, source_text) = file.expect("file part");
    assert_eq!(mime, Some("application/x-directory".to_string()));
    assert_eq!(filename, Some("docs".to_string()));
    assert_eq!(url, "file:///repo/../docs");
    assert_eq!(source_path, Some("/repo/../docs".to_string()));
    assert_eq!(source_text, Some("@docs".to_string()));
}

#[test]
#[ignore = "porting: components/prompt-input/build-request-parts not implemented"]
fn deduplicates_context_files_when_prompt_already_includes_same_path() {
    let input = BuildRequestPartsInput {
        prompt: vec![Prompt::File {
            path: "src/foo.ts".into(),
            content: "@src/foo.ts".into(),
            start: 0,
            end: 11,
            selection: None,
            mime: None,
            filename: None,
        }],
        context: vec![
            ContextItem {
                key: "ctx:dup".into(),
                kind: "file".into(),
                path: "src/foo.ts".into(),
                comment: None,
            },
            ContextItem {
                key: "ctx:comment".into(),
                kind: "file".into(),
                path: "src/foo.ts".into(),
                comment: Some("focus here".into()),
            },
        ],
        images: Vec::new(),
        text: "@src/foo.ts".into(),
        message_id: "msg_2".into(),
        session_id: "ses_2".into(),
        session_directory: "/repo".into(),
    };

    let result = build_request_parts(input);
    let foo_files = result
        .request_parts
        .iter()
        .filter(|part| match part {
            RequestPart::File { url, .. } => url.starts_with("file:///repo/src/foo.ts"),
            _ => false,
        })
        .count();
    let synthetic = result
        .request_parts
        .iter()
        .filter(|part| {
            matches!(
                part,
                RequestPart::Text {
                    synthetic: true,
                    ..
                }
            )
        })
        .count();

    assert_eq!(foo_files, 2);
    assert_eq!(synthetic, 1);
}

#[test]
#[ignore = "porting: components/prompt-input/build-request-parts not implemented"]
fn adds_file_parts_for_mentions_inside_comment_text() {
    let input = BuildRequestPartsInput {
        prompt: vec![text("look", 0, 4)],
        context: vec![ContextItem {
            key: "ctx:comment-mention".into(),
            kind: "file".into(),
            path: "src/review.ts".into(),
            comment: Some("Compare with @src/shared.ts and @src/review.ts.".into()),
        }],
        images: Vec::new(),
        text: "look".into(),
        message_id: "msg_comment_mentions".into(),
        session_id: "ses_comment_mentions".into(),
        session_directory: "/repo".into(),
    };

    let result = build_request_parts(input);
    let urls: Vec<String> = result
        .request_parts
        .iter()
        .filter_map(|part| match part {
            RequestPart::File { url, .. } => Some(url.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(urls.len(), 2);
    assert!(urls.contains(&"file:///repo/src/review.ts".to_string()));
    assert!(urls.contains(&"file:///repo/src/shared.ts".to_string()));
}

#[test]
#[ignore = "porting: components/prompt-input/build-request-parts not implemented"]
fn handles_linux_and_macos_absolute_paths() {
    for (path, directory, expected) in [
        (
            "src/app.ts",
            "/home/user/project",
            "file:///home/user/project/src/app.ts",
        ),
        (
            "README.md",
            "/Users/kelvin/Projects/opencode",
            "file:///Users/kelvin/Projects/opencode/README.md",
        ),
    ] {
        let input = BuildRequestPartsInput {
            prompt: vec![Prompt::File {
                path: path.into(),
                content: format!("@{path}"),
                start: 0,
                end: 10,
                selection: None,
                mime: None,
                filename: None,
            }],
            context: Vec::new(),
            images: Vec::new(),
            text: format!("@{path}"),
            message_id: "msg".into(),
            session_id: "ses".into(),
            session_directory: directory.into(),
        };
        let result = build_request_parts(input);
        let url = result.request_parts.iter().find_map(|part| match part {
            RequestPart::File { url, .. } => Some(url.clone()),
            _ => None,
        });
        assert_eq!(url, Some(expected.to_string()));
    }
}

#[test]
#[ignore = "porting: components/prompt-input/build-request-parts not implemented"]
fn handles_selection_with_query_parameters_on_windows() {
    let input = BuildRequestPartsInput {
        prompt: vec![Prompt::File {
            path: "src\\App.tsx".into(),
            content: "@src\\App.tsx".into(),
            start: 0,
            end: 11,
            selection: Some(Selection {
                start_line: 10,
                start_char: 0,
                end_line: 20,
                end_char: 5,
            }),
            mime: None,
            filename: None,
        }],
        context: Vec::new(),
        images: Vec::new(),
        text: "@src\\App.tsx".into(),
        message_id: "msg_sel".into(),
        session_id: "ses_sel".into(),
        session_directory: "C:\\project".into(),
    };

    let result = build_request_parts(input);
    let url = result.request_parts.iter().find_map(|part| match part {
        RequestPart::File { url, .. } => Some(url.clone()),
        _ => None,
    });
    let url = url.expect("file part");

    assert!(url.contains("?start=10&end=20"));
    assert!(url.contains("start=10"));
    assert!(url.contains("end=20"));
}
