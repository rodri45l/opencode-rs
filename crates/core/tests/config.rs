//! Port of packages/core/test/config/config.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: latest-scalar resolution across priority-ordered
//! documents, v1 detection, and v1 to v2 migration of provider setup options
//! and commands. Dropped (re-derived): the filesystem-backed `entries()` cases
//! and the `FastCheck` property test, which depend on service layers and a
//! property-testing harness.

use opencode_core::config::{Config, ConfigDocument, ConfigEntry, ConfigMigrateV1};
use opencode_core::path::AbsolutePath;
use serde_json::json;

#[test]
fn returns_the_latest_defined_scalar_from_priority_ordered_documents() {
    let entries = vec![
        ConfigEntry::Document(ConfigDocument::new(
            json!({ "model": "openrouter/openai/gpt-5" }),
        )),
        ConfigEntry::Directory(AbsolutePath::new("/skills")),
        ConfigEntry::Document(ConfigDocument::new(json!({}))),
        ConfigEntry::Document(ConfigDocument::new(
            json!({ "model": "openrouter/openai/gpt-5.5" }),
        )),
    ];

    assert_eq!(
        Config::latest(&entries, "model").unwrap(),
        Some(json!("openrouter/openai/gpt-5.5"))
    );
    assert_eq!(Config::latest(&entries, "default_agent").unwrap(), None);
}

#[test]
fn detects_v1_configuration_from_any_v1_only_top_level_key() {
    assert!(ConfigMigrateV1::is_v1(&json!({ "snapshot": false })).unwrap());
    assert!(ConfigMigrateV1::is_v1(&json!({ "snapshot": false, "agents": {} })).unwrap());
    assert!(ConfigMigrateV1::is_v1(&json!({ "reference": {} })).unwrap());
    assert!(
        !ConfigMigrateV1::is_v1(&json!({ "shell": "/bin/zsh", "model": "anthropic/claude" }))
            .unwrap()
    );
    assert!(!ConfigMigrateV1::is_v1(&json!({ "references": {} })).unwrap());
}

#[test]
fn migrates_v1_provider_setup_options_into_aisdk_settings() {
    let migrated = ConfigMigrateV1::migrate(&json!({
        "provider": {
            "bedrock": {
                "npm": "@ai-sdk/amazon-bedrock",
                "options": {
                    "headers": { "x-test": "1" },
                    "body": { "trace": true },
                    "region": "us-east-1",
                    "profile": "dev",
                },
            },
        },
    }))
    .unwrap();

    assert_eq!(
        migrated["providers"]["bedrock"]["api"],
        json!({
            "type": "aisdk",
            "package": "@ai-sdk/amazon-bedrock",
            "settings": { "region": "us-east-1", "profile": "dev" },
        })
    );
    assert_eq!(
        migrated["providers"]["bedrock"]["request"],
        json!({ "headers": { "x-test": "1" }, "body": { "trace": true } })
    );
}

#[test]
fn migrates_v1_command_configuration() {
    let migrated = ConfigMigrateV1::migrate(&json!({
        "command": {
            "review": {
                "template": "Review changes",
                "description": "Review code",
                "agent": "reviewer",
                "model": "anthropic/claude",
                "variant": "high",
                "subtask": true,
            },
        },
    }))
    .unwrap();

    assert_eq!(
        migrated["commands"],
        json!({
            "review": {
                "template": "Review changes",
                "description": "Review code",
                "agent": "reviewer",
                "model": "anthropic/claude",
                "variant": "high",
                "subtask": true,
            },
        })
    );
}
