//! Port of packages/core/test/plugin/skill.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the built-in skill plugin registers a `customize-opencode`
//! skill whose description mentions opencode's own configuration. Re-derived:
//! the `SkillV2`/`PluginHost` service wiring is dropped.
//!
//! Renamed target: recorded in `crates/core/port-map.json`.

use opencode_core::plugin_skill::PluginSkill;

#[test]
fn registers_the_builtin_customize_opencode_skill() {
    let skill = PluginSkill::builtin().unwrap();

    assert_eq!(skill.name, "customize-opencode");
    assert!(skill.description.contains("opencode's own configuration"));
}
