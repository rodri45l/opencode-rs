//! Port of packages/core/test/config/plugin.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: config plugin entries may be bare package strings or
//! objects with `package` and `options`; relative file references and npm
//! specifiers are distinguished; plugin files are discovered by extension; and
//! invalid entries are skipped while loading continues. Re-derived: the
//! `Config`/`Npm`/`FSUtil`/`PluginHost` service wiring, the async module load,
//! and the live fixtures are dropped.

use opencode_core::config_plugin::{ConfigExternalPlugin, PluginSpec};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn parses_string_and_object_plugin_entries_with_options() {
    let specs = ConfigExternalPlugin::parse_specs(&json!([
        "../plugin/fixtures/missing-plugin.ts",
        {
            "package": "../plugin/fixtures/config-promise-plugin.ts",
            "options": { "description": "Loaded from config" }
        }
    ]))
    .unwrap();

    assert_eq!(specs.len(), 2);
    assert_eq!(specs[0].package, "../plugin/fixtures/missing-plugin.ts");
    assert!(specs[0].options.is_empty());
    assert_eq!(
        specs[1].package,
        "../plugin/fixtures/config-promise-plugin.ts"
    );
    assert_eq!(
        specs[1].options.get("description").map(String::as_str),
        Some("Loaded from config")
    );
}

#[test]
fn distinguishes_relative_references_from_npm_specifiers() {
    assert!(ConfigExternalPlugin::is_relative_reference("../plugin/x.ts").unwrap());
    assert!(ConfigExternalPlugin::is_relative_reference("./x.ts").unwrap());
    assert!(!ConfigExternalPlugin::is_relative_reference("example-plugin@1.0.0").unwrap());
    assert!(ConfigExternalPlugin::is_npm_spec("example-plugin@1.0.0").unwrap());
    assert!(!ConfigExternalPlugin::is_npm_spec("../plugin/x.ts").unwrap());
}

#[test]
fn detects_plugin_files_by_extension() {
    assert!(ConfigExternalPlugin::is_plugin_file("config-promise-plugin.ts").unwrap());
    assert!(ConfigExternalPlugin::is_plugin_file("plugin.mts").unwrap());
    assert!(!ConfigExternalPlugin::is_plugin_file("opencode.json").unwrap());
}

#[test]
fn ignores_invalid_plugins_and_continues_loading() {
    let specs = vec![
        PluginSpec {
            package: String::new(),
            options: BTreeMap::new(),
        },
        PluginSpec {
            package: "../plugin/fixtures/config-promise-plugin.ts".into(),
            options: [(
                "description".to_string(),
                "Loaded after invalid plugins".to_string(),
            )]
            .into_iter()
            .collect(),
        },
    ];

    let loadable = ConfigExternalPlugin::loadable_specs(specs).unwrap();
    assert_eq!(loadable.len(), 1);
    assert_eq!(
        loadable[0].options.get("description").map(String::as_str),
        Some("Loaded after invalid plugins")
    );
}
