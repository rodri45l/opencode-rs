//! Port of packages/core/test/config/skill.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: configured skill directories and URLs resolve in order —
//! config directories contribute `skill` and `skills`, then document entries are
//! resolved (relative against the location, `~/` against home, absolute paths
//! unchanged, URLs passed through). Re-derived as a pure mapping; the plugin
//! `Effect`/`Layer` wiring is dropped.

use opencode_core::config_skill::{ConfigSkill, SkillSource};
use opencode_core::path::AbsolutePath;

const NOTE: &str = "porting: config skill not implemented";

#[test]
#[ignore = "porting: config skill not implemented"]
fn registers_configured_skill_directories_and_urls() {
    let config_directories = vec![AbsolutePath::new("/repo/.opencode")];
    let skills = vec![
        "./skills".to_string(),
        "~/shared-skills".to_string(),
        "/opt/skills".to_string(),
        "https://example.test/skills/".to_string(),
    ];
    let location_directory = AbsolutePath::new("/repo/packages/app");

    let sources = ConfigSkill::resolve(
        &config_directories,
        &skills,
        &location_directory,
        "/home/test",
    )
    .expect(NOTE);

    assert_eq!(
        sources,
        vec![
            SkillSource::Directory {
                path: AbsolutePath::new("/repo/.opencode/skill"),
            },
            SkillSource::Directory {
                path: AbsolutePath::new("/repo/.opencode/skills"),
            },
            SkillSource::Directory {
                path: AbsolutePath::new("/repo/packages/app/skills"),
            },
            SkillSource::Directory {
                path: AbsolutePath::new("/home/test/shared-skills"),
            },
            SkillSource::Directory {
                path: AbsolutePath::new("/opt/skills"),
            },
            SkillSource::Url {
                url: "https://example.test/skills/".into(),
            },
        ]
    );
}
