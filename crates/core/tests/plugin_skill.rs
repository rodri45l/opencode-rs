//! Port of packages/core/test/plugin/skill.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: the built-in skill plugin registers a `customize-opencode`
//! skill whose description mentions opencode's own configuration. Re-derived:
//! the `SkillV2`/`PluginHost` service wiring is dropped.
//!
//! Renamed target: recorded in `crates/core/port-map.json`.

use opencode_core::plugin_skill::PluginSkill;

const NOTE: &str = "porting: skill plugin not implemented";

#[test]
#[ignore = "porting: skill plugin not implemented"]
fn registers_the_builtin_customize_opencode_skill() {
    let skill = PluginSkill::builtin().expect(NOTE);

    assert_eq!(skill.name, "customize-opencode");
    assert!(skill.description.contains("opencode's own configuration"));
}
