//! Port of packages/app/src/context/global-sync/utils.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use opencode_app::global_sync_utils::{
    directory_key, normalize_agent_list, normalize_permission_request, normalize_provider_list,
    Agent, ModelInfo, ModelRef, Permission, PermissionRequest,
};

#[test]
fn adapts_current_agents_to_the_app_agent_shape() {
    let input = vec![Agent {
        name: "build".into(),
        description: None,
        mode: Some("primary".into()),
        hidden: false,
        temperature: Some(0.2),
        top_p: Some(0.9),
        color: Some("primary".into()),
        permission: vec![Permission {
            permission: "read".into(),
            pattern: "*".into(),
            action: "allow".into(),
        }],
        model: Some(ModelRef {
            provider_id: "openai".into(),
            model_id: "gpt-5".into(),
        }),
        variant: Some("high".into()),
        prompt: Some("Build software".into()),
        steps: None,
    }];
    assert_eq!(
        normalize_agent_list(input),
        vec![Agent {
            name: "build".into(),
            description: None,
            mode: Some("primary".into()),
            hidden: false,
            temperature: Some(0.2),
            top_p: Some(0.9),
            color: Some("primary".into()),
            permission: vec![Permission {
                permission: "read".into(),
                pattern: "*".into(),
                action: "allow".into()
            }],
            model: Some(ModelRef {
                provider_id: "openai".into(),
                model_id: "gpt-5".into()
            }),
            variant: Some("high".into()),
            prompt: Some("Build software".into()),
            steps: None,
        }]
    );
}

#[test]
fn adapts_the_current_permission_request_to_app_state() {
    let input = PermissionRequest {
        id: "permission-1".into(),
        session_id: "session-1".into(),
        permission: "read".into(),
        patterns: vec!["README.md".into()],
        always: vec!["*.md".into()],
        metadata_path: Some("README.md".into()),
        tool_message_id: Some("message-1".into()),
        tool_call_id: Some("call-1".into()),
    };
    assert_eq!(normalize_permission_request(input.clone()), input);
}

#[test]
fn groups_current_models_into_the_app_provider_catalog() {
    let providers = vec![("openai".to_string(), "OpenAI".to_string())];
    let models = vec![
        ModelInfo {
            id: "gpt-5".into(),
            provider_id: "openai".into(),
            toolcall: true,
            attachment: true,
            cost_input: 1.0,
            cost_output: 2.0,
            variants: vec!["high".into()],
        },
        ModelInfo {
            id: "gpt-old".into(),
            provider_id: "openai".into(),
            toolcall: false,
            attachment: false,
            cost_input: 1.0,
            cost_output: 1.0,
            variants: Vec::new(),
        },
    ];
    let default = Some(ModelRef {
        provider_id: "openai".into(),
        model_id: "gpt-5".into(),
    });
    let result = normalize_provider_list(providers, models, default);
    assert_eq!(result.connected, vec!["openai".to_string()]);
    assert_eq!(
        result.default_model,
        Some(ModelRef {
            provider_id: "openai".into(),
            model_id: "gpt-5".into()
        })
    );
    assert_eq!(result.default.get("openai"), Some(&"gpt-5".to_string()));
    assert!(!result.all["openai"].models.contains_key("gpt-old"));
    assert_eq!(
        result.all["openai"].models["gpt-5"],
        ModelInfo {
            id: "gpt-5".into(),
            provider_id: "openai".into(),
            toolcall: true,
            attachment: true,
            cost_input: 1.0,
            cost_output: 2.0,
            variants: vec!["high".into()]
        }
    );
}

#[test]
fn preserves_an_empty_current_default() {
    assert_eq!(
        normalize_provider_list(Vec::new(), Vec::new(), None).default_model,
        None
    );
}

#[test]
fn directory_key_normalizes_slashes() {
    assert_eq!(
        directory_key("C:\\Repos\\sst\\opencode"),
        "C:/Repos/sst/opencode"
    );
    assert_eq!(
        directory_key("C:/Repos/sst/opencode"),
        "C:/Repos/sst/opencode"
    );
}

#[test]
fn directory_key_preserves_backslashes_in_posix_paths() {
    assert_eq!(directory_key("/tmp/foo\\bar"), "/tmp/foo\\bar");
}

#[test]
fn directory_key_trims_trailing_slashes_without_breaking_roots() {
    assert_eq!(
        directory_key("C:/Repos/sst/opencode/"),
        "C:/Repos/sst/opencode"
    );
    assert_eq!(directory_key("C:/"), "C:/");
    assert_eq!(directory_key("/"), "/");
}
