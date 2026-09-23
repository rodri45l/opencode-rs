//! Port of packages/core/test/plugin/provider-gitlab.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the GitLab plugin binds only to the exact
//! `gitlab-ai-provider` package, resolves the instance URL from the configured
//! option then `GITLAB_INSTANCE_URL` then `https://gitlab.com`, resolves the API
//! key from the configured option then `GITLAB_TOKEN`, merges AI gateway headers
//! and feature flags (configured values winning per key), and applies the
//! default feature flags. Re-derived: the `AISDK`/`Npm` service wiring, SDK
//! factory and workflow-model selection are dropped.

use std::collections::BTreeMap;

use opencode_core::provider_gitlab::{GitLabOptions, GitLabPlugin};

const NOTE: &str = "porting: gitlab provider plugin not implemented";

fn env(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

#[test]
#[ignore = "porting: gitlab provider plugin not implemented"]
fn binds_only_to_the_gitlab_provider_package() {
    assert!(GitLabPlugin::matches_package("gitlab-ai-provider").expect(NOTE));
    assert!(!GitLabPlugin::matches_package("@ai-sdk/openai").expect(NOTE));
}

#[test]
#[ignore = "porting: gitlab provider plugin not implemented"]
fn resolves_legacy_defaults_from_env() {
    let resolved = GitLabPlugin::resolve_options(
        &env(&[("GITLAB_TOKEN", "env-token")]),
        &GitLabOptions::default(),
    )
    .expect(NOTE);

    assert_eq!(resolved.instance_url, "https://gitlab.com");
    assert_eq!(resolved.api_key.as_deref(), Some("env-token"));
    assert_eq!(
        resolved
            .ai_gateway_headers
            .get("anthropic-beta")
            .map(String::as_str),
        Some("context-1m-2025-08-07")
    );
    assert!(resolved
        .ai_gateway_headers
        .get("User-Agent")
        .is_some_and(|value| value.contains("gitlab-ai-provider/")));
    assert_eq!(
        resolved
            .feature_flags
            .get("duo_agent_platform_agentic_chat"),
        Some(&true)
    );
    assert_eq!(
        resolved.feature_flags.get("duo_agent_platform"),
        Some(&true)
    );
}

#[test]
#[ignore = "porting: gitlab provider plugin not implemented"]
fn uses_the_instance_url_env_when_not_configured() {
    let resolved = GitLabPlugin::resolve_options(
        &env(&[("GITLAB_INSTANCE_URL", "https://env.gitlab.example")]),
        &GitLabOptions::default(),
    )
    .expect(NOTE);

    assert_eq!(resolved.instance_url, "https://env.gitlab.example");
}

#[test]
#[ignore = "porting: gitlab provider plugin not implemented"]
fn keeps_configured_options_over_env_and_defaults() {
    let configured = GitLabOptions {
        instance_url: Some("https://configured.gitlab.example".to_string()),
        api_key: Some("configured-token".to_string()),
        ai_gateway_headers: BTreeMap::from([
            ("anthropic-beta".to_string(), "configured-beta".to_string()),
            ("x-gitlab-test".to_string(), "1".to_string()),
        ]),
        feature_flags: BTreeMap::from([
            ("duo_agent_platform".to_string(), false),
            ("custom_flag".to_string(), true),
        ]),
    };
    let resolved = GitLabPlugin::resolve_options(
        &env(&[
            ("GITLAB_INSTANCE_URL", "https://env.gitlab.example"),
            ("GITLAB_TOKEN", "env-token"),
        ]),
        &configured,
    )
    .expect(NOTE);

    assert_eq!(resolved.instance_url, "https://configured.gitlab.example");
    assert_eq!(resolved.api_key.as_deref(), Some("configured-token"));
    assert_eq!(
        resolved
            .ai_gateway_headers
            .get("anthropic-beta")
            .map(String::as_str),
        Some("configured-beta")
    );
    assert_eq!(
        resolved
            .ai_gateway_headers
            .get("x-gitlab-test")
            .map(String::as_str),
        Some("1")
    );
    assert_eq!(
        resolved.feature_flags.get("duo_agent_platform"),
        Some(&false)
    );
    assert_eq!(resolved.feature_flags.get("custom_flag"), Some(&true));
    assert_eq!(
        resolved
            .feature_flags
            .get("duo_agent_platform_agentic_chat"),
        Some(&true)
    );
}
