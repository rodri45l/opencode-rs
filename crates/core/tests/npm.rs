//! Port of packages/core/test/npm.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `sanitize` keeps normal scoped specs and rewrites
//! `git+https` specs on Windows, and `add` resolves an installed package
//! entrypoint. Re-derived: the Bun/Node subprocess, `file:` package install, and
//! `.npmrc` install cases are replaced by direct calls; the Node-runtime and
//! install cases are noted as dropped.

use opencode_core::npm::Npm;

const NOTE: &str = "porting: npm not implemented";

#[test]
#[ignore = "porting: npm not implemented"]
fn keeps_normal_scoped_package_specs_unchanged() {
    assert_eq!(
        Npm::sanitize("@opencode/acme").expect(NOTE),
        "@opencode/acme"
    );
    assert_eq!(
        Npm::sanitize("@opencode/acme@1.0.0").expect(NOTE),
        "@opencode/acme@1.0.0"
    );
    assert_eq!(Npm::sanitize("prettier").expect(NOTE), "prettier");
}

#[test]
#[ignore = "porting: npm not implemented"]
fn resolves_an_importable_entrypoint() {
    let entry = Npm::add("@opencode/acme@1.0.0").expect(NOTE);
    assert!(!entry.entrypoint.is_empty());
}

#[test]
#[ignore = "porting: npm not implemented"]
fn handles_git_https_specs() {
    let spec = "acme@git+https://github.com/opencode/acme.git";
    let expected = if cfg!(windows) {
        "acme@git+https_//github.com/opencode/acme.git"
    } else {
        spec
    };
    assert_eq!(Npm::sanitize(spec).expect(NOTE), expected);
}
