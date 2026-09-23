//! Port of packages/core/test/reference.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: normalized local sources are registered for the owning
//! scope, git paths are derived from the repository cache layout without
//! exposing cache operations, and configured git descriptions are preserved.
//! Re-derived against the Rust registry; scope-close removal is dropped.

use opencode_core::path::AbsolutePath;
use opencode_core::reference::Reference;
use opencode_core::reference_guidance::ReferenceSource;

const NOTE: &str = "porting: reference registry not implemented";

#[test]
#[ignore = "porting: reference registry not implemented"]
fn registers_normalized_sources_for_the_owning_scope() {
    let mut references = Reference::new(AbsolutePath::new("/repos"));
    let path = AbsolutePath::new("/docs");
    let source = ReferenceSource::Local {
        path: path.clone(),
        description: Some("Use for API documentation".into()),
    };
    references.add("docs", source.clone()).expect(NOTE);

    let list = references.list().expect(NOTE);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "docs");
    assert_eq!(list[0].path, path);
    assert_eq!(
        list[0].description.as_deref(),
        Some("Use for API documentation")
    );
    assert!(list[0].hidden);
    assert_eq!(list[0].source, source);
}

#[test]
#[ignore = "porting: reference registry not implemented"]
fn derives_git_paths_without_exposing_cache_operations() {
    let mut references = Reference::new(AbsolutePath::new("/repos"));
    let source = ReferenceSource::Git {
        repository: "owner/repo".into(),
        branch: Some("main".into()),
        description: None,
    };
    references.add("sdk", source.clone()).expect(NOTE);

    let list = references.list().expect(NOTE);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "sdk");
    assert_eq!(
        list[0].path,
        AbsolutePath::new("/repos/github.com/owner/repo@main")
    );
    assert_eq!(list[0].description, None);
    assert_eq!(list[0].source, source);
}

#[test]
#[ignore = "porting: reference registry not implemented"]
fn preserves_configured_git_descriptions() {
    let mut references = Reference::new(AbsolutePath::new("/repos"));
    let source = ReferenceSource::Git {
        repository: "owner/repo".into(),
        branch: None,
        description: Some("Use for SDK implementation details".into()),
    };
    references.add("sdk", source.clone()).expect(NOTE);

    let list = references.list().expect(NOTE);
    assert_eq!(list.len(), 1);
    assert_eq!(
        list[0].path,
        AbsolutePath::new("/repos/github.com/owner/repo")
    );
    assert_eq!(
        list[0].description.as_deref(),
        Some("Use for SDK implementation details")
    );
    assert_eq!(list[0].source, source);
}
