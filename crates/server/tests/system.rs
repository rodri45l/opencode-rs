//! Port of packages/opencode/test/session/system.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: provider-specific prompt selection, the name-sorted skills
//! block, and the MCP instructions block filtered by permission.

use opencode_core::CoreError;
use opencode_server::agent::{AgentInfo, AgentMode};
use opencode_server::permission::{from_config, Ruleset};
use opencode_server::system_prompt::SystemPrompt;

fn build_agent() -> AgentInfo {
    AgentInfo {
        name: "build".to_string(),
        mode: AgentMode::Primary,
        description: None,
        permission: from_config(&serde_json::json!({ "*": "allow" })),
        options: serde_json::Value::Null,
    }
}

fn first_prompt(api_id: &str) -> String {
    SystemPrompt::provider(api_id)
        .into_iter()
        .next()
        .unwrap_or_default()
}

#[test]
fn selects_the_meta_prompt_for_muse_spark_model_ids() {
    for id in [
        "meta/muse-spark-preview",
        "muse-spark-1.1",
        "muse-spark-1.2",
    ] {
        let prompt = first_prompt(id);
        assert!(prompt.contains("powered by Muse Spark,"));
        assert!(prompt.contains("using Meta Muse Spark."));
        assert!(!prompt.contains("{{MODEL_NAME}}"));
    }
}

#[test]
fn selects_the_meta_prompt_for_muse_glimmer_model_ids() {
    for id in [
        "meta/muse-glimmer",
        "meta/muse-glimmer-30b",
        "muse-glimmer-30b",
    ] {
        let prompt = first_prompt(id);
        assert!(prompt.contains("powered by Muse Glimmer,"));
        assert!(prompt.contains("using Meta Muse Glimmer."));
        assert!(!prompt.contains("{{MODEL_NAME}}"));
    }
}

#[test]
fn selects_the_kimi_prompt_for_official_provider_model_ids() {
    for provider_id in ["kimi-for-coding", "moonshotai", "moonshotai-cn"] {
        let _ = provider_id;
        let prompt = first_prompt("k3");
        assert!(prompt.contains("# Prompt and Tool Use"));
    }
}

#[test]
fn skills_output_is_sorted_by_name_and_stable_across_calls() -> Result<(), CoreError> {
    let prompt = SystemPrompt::new();
    let agent = build_agent();
    let first = prompt.skills(&agent)?;
    let second = prompt.skills(&agent)?;

    assert_eq!(first, second);

    let output = first.unwrap_or_default();
    let alpha = output.find("<name>alpha-skill</name>");
    let middle = output.find("<name>middle-skill</name>");
    let zeta = output.find("<name>zeta-skill</name>");

    let alpha = alpha.expect("alpha present");
    let middle = middle.expect("middle present");
    let zeta = zeta.expect("zeta present");
    assert!(middle > alpha);
    assert!(zeta > middle);
    assert!(!output.contains("manual-skill"));
    Ok(())
}

#[test]
fn mcp_output_includes_connected_server_instructions() -> Result<(), CoreError> {
    let prompt = SystemPrompt::new();
    let output = prompt.mcp(&build_agent(), &Ruleset::new())?;

    assert_eq!(
        output,
        [
            "<mcp_instructions>",
            "  <server name=\"guide-server\">",
            "    Use lookup before mutate.",
            "  </server>",
            "  <server name=\"tool-server\">",
            "    Prefer search before update.",
            "  </server>",
            "</mcp_instructions>",
        ]
        .join("\n")
    );
    Ok(())
}

#[test]
fn mcp_output_omits_servers_when_all_advertised_tools_are_denied() -> Result<(), CoreError> {
    let prompt = SystemPrompt::new();
    let permission = from_config(&serde_json::json!({ "tool-server_*": "deny" }));
    let output = prompt.mcp(&build_agent(), &permission)?;

    assert_eq!(
        output,
        [
            "<mcp_instructions>",
            "  <server name=\"guide-server\">",
            "    Use lookup before mutate.",
            "  </server>",
            "</mcp_instructions>",
        ]
        .join("\n")
    );
    Ok(())
}
