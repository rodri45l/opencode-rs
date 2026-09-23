//! Port of packages/core/test/command.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: two updates to the same command merge, with the later
//! update overriding earlier fields while preserving untouched ones.

use opencode_core::command::{CommandInfo, CommandRegistry};
use opencode_core::model::{ModelId, ProviderId, VariantId};

fn expected_model() -> opencode_core::model::ModelRef {
    opencode_core::model::ModelRef {
        id: ModelId::make("claude"),
        provider_id: ProviderId::make("anthropic"),
        variant: Some(VariantId::make("high")),
    }
}

#[test]
fn applies_command_transforms_and_preserves_later_overrides() {
    let command = CommandRegistry::new().unwrap();
    command
        .transform(|editor| {
            editor.update("review", |command| {
                command.template = Some("First".into());
                command.description = Some("Review code".into());
            });
            editor.update("review", |command| {
                command.template = Some("Second".into());
                command.model = Some(expected_model());
            });
        })
        .unwrap();

    let expected = CommandInfo {
        name: "review".into(),
        template: "Second".into(),
        description: Some("Review code".into()),
        model: Some(expected_model()),
    };
    assert_eq!(command.get("review").unwrap(), Some(expected.clone()));
    assert_eq!(command.list().unwrap(), vec![expected]);
}
