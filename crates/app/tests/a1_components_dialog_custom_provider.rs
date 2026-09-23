//! Port of packages/app/src/components/dialog-custom-provider.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
struct ModelRow {
    row: String,
    id: String,
    name: String,
    err: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq)]
struct HeaderRow {
    row: String,
    key: String,
    value: String,
    err: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq)]
struct Form {
    provider_id: String,
    name: String,
    base_url: String,
    api_key: String,
    models: Vec<ModelRow>,
    headers: Vec<HeaderRow>,
}

#[derive(Clone, Debug, PartialEq)]
struct ProviderConfig {
    npm: String,
    name: String,
    env: Vec<String>,
    base_url: String,
    headers: BTreeMap<String, String>,
    models: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq)]
struct ProviderResult {
    provider_id: String,
    name: String,
    key: Option<String>,
    config: ProviderConfig,
}

#[derive(Clone, Debug, PartialEq)]
struct Validation {
    result: Option<ProviderResult>,
    provider_id_err: Option<String>,
    models: Vec<BTreeMap<String, Option<String>>>,
    headers: Vec<BTreeMap<String, Option<String>>>,
}

// Local stub (fast wave): real module lands later.
fn validate_custom_provider(
    _form: &Form,
    _disabled_providers: &[&str],
    _existing_provider_ids: &[&str],
) -> Validation {
    Validation {
        result: None,
        provider_id_err: None,
        models: Vec::new(),
        headers: Vec::new(),
    }
}

#[test]
#[ignore = "porting: components/dialog-custom-provider not implemented"]
fn builds_trimmed_config_payload() {
    let form = Form {
        provider_id: "custom-provider".into(),
        name: " Custom Provider ".into(),
        base_url: "https://api.example.com ".into(),
        api_key: " {env: CUSTOM_PROVIDER_KEY} ".into(),
        models: vec![ModelRow {
            row: "m0".into(),
            id: " model-a ".into(),
            name: " Model A ".into(),
            err: BTreeMap::new(),
        }],
        headers: vec![
            HeaderRow {
                row: "h0".into(),
                key: " X-Test ".into(),
                value: " enabled ".into(),
                err: BTreeMap::new(),
            },
            HeaderRow {
                row: "h1".into(),
                key: String::new(),
                value: String::new(),
                err: BTreeMap::new(),
            },
        ],
    };

    let mut headers = BTreeMap::new();
    headers.insert("X-Test".to_string(), "enabled".to_string());
    let mut models = BTreeMap::new();
    models.insert("model-a".to_string(), "Model A".to_string());

    assert_eq!(
        validate_custom_provider(&form, &[], &[]).result,
        Some(ProviderResult {
            provider_id: "custom-provider".into(),
            name: "Custom Provider".into(),
            key: None,
            config: ProviderConfig {
                npm: "@ai-sdk/openai-compatible".into(),
                name: "Custom Provider".into(),
                env: vec!["CUSTOM_PROVIDER_KEY".into()],
                base_url: "https://api.example.com".into(),
                headers,
                models,
            },
        })
    );
}

#[test]
#[ignore = "porting: components/dialog-custom-provider not implemented"]
fn flags_duplicate_rows_and_allows_reconnecting_disabled_providers() {
    let form = Form {
        provider_id: "custom-provider".into(),
        name: "Provider".into(),
        base_url: "https://api.example.com".into(),
        api_key: "secret".into(),
        models: vec![
            ModelRow {
                row: "m0".into(),
                id: "model-a".into(),
                name: "Model A".into(),
                err: BTreeMap::new(),
            },
            ModelRow {
                row: "m1".into(),
                id: "model-a".into(),
                name: "Model A 2".into(),
                err: BTreeMap::new(),
            },
        ],
        headers: vec![
            HeaderRow {
                row: "h0".into(),
                key: "Authorization".into(),
                value: "one".into(),
                err: BTreeMap::new(),
            },
            HeaderRow {
                row: "h1".into(),
                key: "authorization".into(),
                value: "two".into(),
                err: BTreeMap::new(),
            },
        ],
    };
    let validation = validate_custom_provider(&form, &["custom-provider"], &["custom-provider"]);
    assert_eq!(validation.result, None);
    assert_eq!(validation.provider_id_err, None);
    let mut expected_model = BTreeMap::new();
    expected_model.insert(
        "id".to_string(),
        Some("provider.custom.error.duplicate".to_string()),
    );
    expected_model.insert("name".to_string(), None);
    assert_eq!(validation.models.get(1), Some(&expected_model));
    let mut expected_header = BTreeMap::new();
    expected_header.insert(
        "key".to_string(),
        Some("provider.custom.error.duplicate".to_string()),
    );
    expected_header.insert("value".to_string(), None);
    assert_eq!(validation.headers.get(1), Some(&expected_header));
}
