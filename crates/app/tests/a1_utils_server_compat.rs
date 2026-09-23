//! Port of packages/app/src/utils/server-compat.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct PromptFile {
    uri: String,
    name: String,
    mention: Option<Mention>,
}

#[derive(Clone, Debug, PartialEq)]
struct Mention {
    text: String,
    start: i64,
    end: i64,
}

#[derive(Clone, Debug, PartialEq)]
struct PromptRequest {
    session_id: String,
    id: String,
    text: String,
    agent: Option<String>,
    model: Option<Model>,
    files: Vec<PromptFile>,
    legacy_parts: Option<Vec<LegacyPart>>,
}

#[derive(Clone, Debug, PartialEq)]
struct Model {
    provider_id: String,
    model_id: String,
}

#[derive(Clone, Debug, PartialEq)]
enum LegacyPart {
    Text {
        id: String,
        text: String,
    },
    File {
        id: String,
        mime: String,
        url: String,
        filename: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
enum PromptPart {
    Text {
        text: String,
    },
    File {
        mime: String,
        url: String,
        filename: String,
        source: Option<FileSource>,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct FileSource {
    source_type: String,
    text_value: String,
    start: i64,
    end: i64,
    path: String,
}

#[derive(Clone, Debug, PartialEq)]
struct PromptBody {
    message_id: String,
    agent: Option<String>,
    model: Option<Model>,
    parts: Vec<PromptPart>,
}

// Local stub (fast wave): real module lands later.
fn convert_v1_prompt(_request: &PromptRequest) -> PromptBody {
    PromptBody {
        message_id: String::new(),
        agent: None,
        model: None,
        parts: Vec::new(),
    }
}

fn v1_list_path() -> String {
    String::new()
}

fn v1_file_find_query(_limit: i64) -> (String, String) {
    (String::new(), String::new())
}

fn v1_permission_reply_path(session_id: &str, request_id: &str) -> String {
    let _ = (session_id, request_id);
    String::new()
}

fn v1_connect_paths(integration_id: &str) -> Vec<String> {
    let _ = integration_id;
    Vec::new()
}

fn v1_oauth_paths(integration_id: &str) -> Vec<String> {
    let _ = integration_id;
    Vec::new()
}

#[test]
#[ignore = "porting: utils/server-compat not implemented"]
fn converts_current_prompts_to_the_v1_prompt_contract() {
    let request = PromptRequest {
        session_id: "ses_1".into(),
        id: "msg_1".into(),
        text: "hello @src/index.ts".into(),
        agent: Some("build".into()),
        model: Some(Model {
            provider_id: "provider".into(),
            model_id: "model".into(),
        }),
        files: vec![
            PromptFile {
                uri: "file:///repo/src/index.ts".into(),
                name: "index.ts".into(),
                mention: Some(Mention {
                    text: "@src/index.ts".into(),
                    start: 6,
                    end: 19,
                }),
            },
            PromptFile {
                uri: "data:text/plain;base64,aGVsbG8=".into(),
                name: "notes.txt".into(),
                mention: None,
            },
        ],
        legacy_parts: None,
    };

    let body = convert_v1_prompt(&request);
    assert_eq!(body.message_id, "msg_1");
    assert_eq!(body.agent.as_deref(), Some("build"));
    assert_eq!(
        body.model,
        Some(Model {
            provider_id: "provider".into(),
            model_id: "model".into()
        })
    );
    assert_eq!(
        body.parts,
        vec![
            PromptPart::Text {
                text: "hello @src/index.ts".into()
            },
            PromptPart::File {
                mime: "text/plain".into(),
                url: "file:///repo/src/index.ts".into(),
                filename: "index.ts".into(),
                source: Some(FileSource {
                    source_type: "file".into(),
                    text_value: "@src/index.ts".into(),
                    start: 6,
                    end: 19,
                    path: "file:///repo/src/index.ts".into()
                })
            },
            PromptPart::File {
                mime: "text/plain".into(),
                url: "data:text/plain;base64,aGVsbG8=".into(),
                filename: "notes.txt".into(),
                source: None
            },
        ]
    );
}

#[test]
#[ignore = "porting: utils/server-compat not implemented"]
fn preserves_original_parts_for_v1_optimistic_reconciliation() {
    let request = PromptRequest {
        session_id: "ses_1".into(),
        id: "msg_1".into(),
        text: "look".into(),
        agent: None,
        model: None,
        files: vec![PromptFile {
            uri: "data:image/png;base64,AAAA".into(),
            name: "image.png".into(),
            mention: None,
        }],
        legacy_parts: Some(vec![
            LegacyPart::Text {
                id: "prt_text".into(),
                text: "look".into(),
            },
            LegacyPart::File {
                id: "prt_image".into(),
                mime: "image/png".into(),
                url: "data:image/png;base64,AAAA".into(),
                filename: "image.png".into(),
            },
        ]),
    };
    let body = convert_v1_prompt(&request);
    assert_eq!(body.parts.len(), 2);
}

#[test]
#[ignore = "porting: utils/server-compat not implemented"]
fn uses_the_global_v1_session_search_endpoint() {
    assert_eq!(v1_list_path(), "/experimental/session");
}

#[test]
#[ignore = "porting: utils/server-compat not implemented"]
fn translates_current_file_searches_to_the_v1_dirs_parameter() {
    assert_eq!(
        v1_file_find_query(20),
        ("/find/file".into(), "false".into())
    );
}

#[test]
#[ignore = "porting: utils/server-compat not implemented"]
fn routes_v1_permission_replies_through_the_requested_directory() {
    assert_eq!(
        v1_permission_reply_path("ses_1", "permission_1"),
        "/session/ses_1/permissions/permission_1"
    );
}

#[test]
#[ignore = "porting: utils/server-compat not implemented"]
fn disposes_the_v1_instance_after_connecting_a_provider() {
    assert_eq!(
        v1_connect_paths("openrouter"),
        vec![
            "/auth/openrouter".to_string(),
            "/instance/dispose".to_string(),
            "/instance/dispose".to_string()
        ]
    );
}

#[test]
#[ignore = "porting: utils/server-compat not implemented"]
fn disposes_the_v1_instance_after_completing_provider_oauth() {
    assert_eq!(
        v1_oauth_paths("openrouter"),
        vec![
            "/provider/openrouter/oauth/callback".to_string(),
            "/instance/dispose".to_string(),
            "/instance/dispose".to_string()
        ]
    );
}
