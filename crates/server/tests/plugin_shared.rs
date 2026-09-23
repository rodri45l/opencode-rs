//! Port of packages/opencode/test/plugin/shared.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `parsePluginSpecifier` resolves npm names, scoped names,
//! explicit versions, git protocols, aliases, and bare `npm:` protocol
//! specifiers. The loader/target-resolution exports of `shared.ts` are not
//! pure and are covered by the plugin-loader suite.

use opencode_server::port::plugin::{parse_plugin_specifier, PluginSpecifier};

fn spec(pkg: &str, version: &str) -> PluginSpecifier {
    PluginSpecifier {
        pkg: pkg.to_string(),
        version: version.to_string(),
    }
}

#[test]
fn parses_standard_npm_package_without_version() {
    assert_eq!(parse_plugin_specifier("acme"), spec("acme", "latest"));
}

#[test]
fn parses_standard_npm_package_with_version() {
    assert_eq!(parse_plugin_specifier("acme@1.0.0"), spec("acme", "1.0.0"));
}

#[test]
fn parses_scoped_npm_package_without_version() {
    assert_eq!(
        parse_plugin_specifier("@opencode/acme"),
        spec("@opencode/acme", "latest")
    );
}

#[test]
fn parses_scoped_npm_package_with_version() {
    assert_eq!(
        parse_plugin_specifier("@opencode/acme@1.0.0"),
        spec("@opencode/acme", "1.0.0")
    );
}

#[test]
fn parses_package_with_git_https_url() {
    assert_eq!(
        parse_plugin_specifier("acme@git+https://github.com/opencode/acme.git"),
        spec("acme", "git+https://github.com/opencode/acme.git")
    );
}

#[test]
fn parses_scoped_package_with_git_https_url() {
    assert_eq!(
        parse_plugin_specifier("@opencode/acme@git+https://github.com/opencode/acme.git"),
        spec("@opencode/acme", "git+https://github.com/opencode/acme.git")
    );
}

#[test]
fn parses_package_with_git_ssh_url_containing_another_at() {
    assert_eq!(
        parse_plugin_specifier("acme@git+ssh://git@github.com/opencode/acme.git"),
        spec("acme", "git+ssh://git@github.com/opencode/acme.git")
    );
}

#[test]
fn parses_scoped_package_with_git_ssh_url_containing_another_at() {
    assert_eq!(
        parse_plugin_specifier("@opencode/acme@git+ssh://git@github.com/opencode/acme.git"),
        spec(
            "@opencode/acme",
            "git+ssh://git@github.com/opencode/acme.git"
        )
    );
}

#[test]
fn parses_unaliased_git_ssh_url() {
    assert_eq!(
        parse_plugin_specifier("git+ssh://git@github.com/opencode/acme.git"),
        spec("git+ssh://git@github.com/opencode/acme.git", "")
    );
}

#[test]
fn parses_npm_alias_using_the_alias_name() {
    assert_eq!(
        parse_plugin_specifier("acme@npm:@opencode/acme@1.0.0"),
        spec("acme", "npm:@opencode/acme@1.0.0")
    );
}

#[test]
fn parses_bare_npm_protocol_specifier_using_the_target_package() {
    assert_eq!(
        parse_plugin_specifier("npm:@opencode/acme@1.0.0"),
        spec("@opencode/acme", "1.0.0")
    );
}

#[test]
fn parses_unversioned_npm_protocol_specifier() {
    assert_eq!(
        parse_plugin_specifier("npm:@opencode/acme"),
        spec("@opencode/acme", "latest")
    );
}
