//! Port of packages/app/src/context/global-sync/utils.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
struct Agent {
    name: String,
    description: Option<String>,
    mode: Option<String>,
    hidden: bool,
    temperature: Option<f64>,
    top_p: Option<f64>,
    color: Option<String>,
    permission: Vec<Permission>,
    model: Option<ModelRef>,
    variant: Option<String>,
    prompt: Option<String>,
    steps: Option<i64>,
}

#[derive(Clone, Debug, PartialEq)]
struct Permission {
    permission: String,
    pattern: String,
    action: String,
}

#[derive(Clone, Debug, PartialEq)]
struct ModelRef {
    provider_id: String,
    model_id: String,
}

#[derive(Clone, Debug, PartialEq)]
struct PermissionRequest {
    id: String,
    session_id: String,
    permission: String,
    patterns: Vec<String>,
    always: Vec<String>,
    metadata_path: Option<String>,
    tool_message_id: Option<String>,
    tool_call_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct Provider {
    id: String,
    name: String,
    models: BTreeMap<String, ModelInfo>,
}

#[derive(Clone, Debug, PartialEq)]
struct ModelInfo {
    id: String,
    provider_id: String,
    toolcall: bool,
    attachment: bool,
    cost_input: f64,
    cost_output: f64,
    variants: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct ProviderList {
    connected: Vec<String>,
    default_model: Option<ModelRef>,
    default: BTreeMap<String, String>,
    all: BTreeMap<String, Provider>,
}

// Local stubs (fast wave): real module lands later.
fn normalize_agent_list(_input: Vec<Agent>) -> Vec<Agent> {
    Vec::new()
}

fn normalize_permission_request(_input: PermissionRequest) -> PermissionRequest {
    PermissionRequest {
        id: String::new(),
        session_id: String::new(),
        permission: String::new(),
        patterns: Vec::new(),
        always: Vec::new(),
        metadata_path: None,
        tool_message_id: None,
        tool_call_id: None,
    }
}

fn normalize_provider_list(
    _providers: Vec<(String, String)>,
    _models: Vec<ModelInfo>,
    _default: Option<ModelRef>,
) -> ProviderList {
    ProviderList {
        connected: Vec::new(),
        default_model: None,
        default: BTreeMap::new(),
        all: BTreeMap::new(),
    }
}

fn directory_key(path: &str) -> String {
    let replaced = if path.contains('\\') && !path.starts_with('/') {
        path.replace('\\', "/")
    } else {
        path.to_string()
    };
    if replaced.len() > 1 {
        replaced.trim_end_matches('/').to_string()
    } else {
        replaced
    }
}

#[test]
#[ignore = "porting: context/global-sync/utils not implemented"]
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
#[ignore = "porting: context/global-sync/utils not implemented"]
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
#[ignore = "porting: context/global-sync/utils not implemented"]
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
#[ignore = "porting: context/global-sync/utils not implemented"]
fn preserves_an_empty_current_default() {
    assert_eq!(
        normalize_provider_list(Vec::new(), Vec::new(), None).default_model,
        None
    );
}

#[test]
#[ignore = "porting: context/global-sync/utils not implemented"]
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
#[ignore = "porting: context/global-sync/utils not implemented"]
fn directory_key_preserves_backslashes_in_posix_paths() {
    assert_eq!(directory_key("/tmp/foo\\bar"), "/tmp/foo\\bar");
}

#[test]
#[ignore = "porting: context/global-sync/utils not implemented"]
fn directory_key_trims_trailing_slashes_without_breaking_roots() {
    assert_eq!(
        directory_key("C:/Repos/sst/opencode/"),
        "C:/Repos/sst/opencode"
    );
    assert_eq!(directory_key("C:/"), "C:/");
    assert_eq!(directory_key("/"), "/");
}
