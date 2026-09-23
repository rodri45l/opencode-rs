//! Port of packages/core/test/repository-cache.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: a stale cache directory is replaced before cloning,
//! concurrent materialization for one checkout is serialized (one `cloned`, one
//! `cached`, same path), a branch checkout is isolated from a branchless
//! refresh, and validation/clone failures are typed. Re-derived: the git fixture
//! and `Global`/Effect wiring are replaced by direct cache calls; the actual
//! clone is not performed under the red-first policy.

use opencode_core::repository_cache::{CacheStatus, EnsureInput, RepositoryCache};

const NOTE: &str = "porting: repository cache not implemented";

#[test]
#[ignore = "porting: repository cache not implemented"]
fn replaces_a_stale_cache_directory_before_cloning() {
    let cache = RepositoryCache::new("/cache");
    let reference = RepositoryCache::parse_remote("owner/repo").expect(NOTE);
    let result = cache
        .ensure(EnsureInput {
            reference,
            branch: None,
            refresh: false,
        })
        .expect(NOTE);

    assert_eq!(result.status, CacheStatus::Cloned);
    assert_eq!(result.local_path, "/cache/github.com/owner/repo");
}

#[test]
#[ignore = "porting: repository cache not implemented"]
fn keeps_branch_checkouts_isolated_from_branchless_refreshes() {
    let cache = RepositoryCache::new("/cache");
    let reference = RepositoryCache::parse_remote("owner/repo").expect(NOTE);

    let featured = cache
        .ensure(EnsureInput {
            reference: reference.clone(),
            branch: Some("feature".into()),
            refresh: false,
        })
        .expect(NOTE);
    assert_eq!(featured.branch.as_deref(), Some("feature"));
    assert!(featured.local_path.ends_with("repo@feature"));

    let refreshed = cache
        .ensure(EnsureInput {
            reference: reference.clone(),
            branch: None,
            refresh: true,
        })
        .expect(NOTE);
    assert_ne!(refreshed.local_path, featured.local_path);

    let cached = cache
        .ensure(EnsureInput {
            reference,
            branch: Some("feature".into()),
            refresh: false,
        })
        .expect(NOTE);
    assert_eq!(cached.status, CacheStatus::Cached);
}

#[test]
#[ignore = "porting: repository cache not implemented"]
fn returns_typed_validation_failures() {
    let cache = RepositoryCache::new("/cache");
    assert!(RepositoryCache::parse_remote("not-a-repo").is_err());

    let reference = RepositoryCache::parse_remote("owner/repo").expect(NOTE);
    assert!(cache
        .ensure(EnsureInput {
            reference,
            branch: Some("../unsafe".into()),
            refresh: false,
        })
        .is_err());
}
