//! Port of packages/app/src/utils/server-compat.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::server_compat::{
    convert_v1_prompt, v1_connect_paths, v1_file_find_query, v1_list_path, v1_oauth_paths,
    v1_permission_reply_path, FileSource, LegacyPart, Mention, Model, PromptFile, PromptPart,
    PromptRequest,
};

#[test]
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
fn uses_the_global_v1_session_search_endpoint() {
    assert_eq!(v1_list_path(), "/experimental/session");
}

#[test]
fn translates_current_file_searches_to_the_v1_dirs_parameter() {
    assert_eq!(
        v1_file_find_query(20),
        ("/find/file".into(), "false".into())
    );
}

#[test]
fn routes_v1_permission_replies_through_the_requested_directory() {
    assert_eq!(
        v1_permission_reply_path("ses_1", "permission_1"),
        "/session/ses_1/permissions/permission_1"
    );
}

#[test]
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
