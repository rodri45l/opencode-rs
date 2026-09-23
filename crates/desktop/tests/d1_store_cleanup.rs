//! Port of packages/desktop/src/main/store-cleanup.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/store-cleanup.ts: empty scoped
//! stores (`opencode.draft.*`, `opencode.workspace.*`) are deleted while
//! `opencode.global.dat` and `.json` sidecars survive; stale drafts are removed
//! by age; scoped stores are capped by recency; and an emptied scoped store is
//! removed immediately.
//! Re-derived: the filesystem is abstracted into an owned file list so age and
//! recency are explicit inputs rather than `stat` calls.

#[allow(dead_code)]
mod store_cleanup {
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: desktop store cleanup not implemented";

    fn stub<T>() -> PortResult<T> {
        Err(NotImplemented(NOTE))
    }

    #[derive(Debug, Clone)]
    pub struct StoreFile {
        pub name: String,
        pub contents: String,
        pub modified_ms: i64,
    }

    impl StoreFile {
        pub fn new(name: &str, contents: &str, modified_ms: i64) -> Self {
            Self {
                name: name.to_string(),
                contents: contents.to_string(),
                modified_ms,
            }
        }
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct CleanupResult {
        pub deleted: Vec<String>,
    }

    pub fn cleanup_store_files(_files: &[StoreFile], _now_ms: i64) -> PortResult<CleanupResult> {
        stub()
    }

    pub fn delete_store_file_if_empty(_files: &[StoreFile], _name: &str) -> PortResult<bool> {
        stub()
    }
}

use store_cleanup::{cleanup_store_files, delete_store_file_if_empty, StoreFile, NOTE};

const NOW: i64 = 1_782_000_000_000; // 2026-07-01T00:00:00Z

#[test]
#[ignore = "porting: desktop store cleanup not implemented"]
fn removes_empty_scoped_stores_and_leaves_global_stores_alone() {
    let files = vec![
        StoreFile::new("opencode.draft.empty.dat", "{}", NOW),
        StoreFile::new("opencode.workspace.empty.dat", "{\n}", NOW),
        StoreFile::new("opencode.global.dat", "{}", NOW),
        StoreFile::new("opencode.workspace.empty.dat.json", "{}", NOW),
    ];

    let result = cleanup_store_files(&files, NOW).expect(NOTE);
    let mut deleted = result.deleted.clone();
    deleted.sort();
    assert_eq!(
        deleted,
        vec![
            "opencode.draft.empty.dat".to_string(),
            "opencode.workspace.empty.dat".to_string()
        ]
    );

    let mut remaining: Vec<String> = files
        .iter()
        .filter(|file| !result.deleted.contains(&file.name))
        .map(|file| file.name.clone())
        .collect();
    remaining.sort();
    assert_eq!(
        remaining,
        vec![
            "opencode.global.dat".to_string(),
            "opencode.workspace.empty.dat.json".to_string()
        ]
    );
}

#[test]
#[ignore = "porting: desktop store cleanup not implemented"]
fn removes_stale_drafts_by_age_without_removing_non_empty_workspace_stores() {
    let old = NOW - 61 * 24 * 60 * 60 * 1000;
    let files = vec![
        StoreFile::new(
            "opencode.draft.old.dat",
            "{\"draft:prompt\":\"hello\"}",
            old,
        ),
        StoreFile::new(
            "opencode.draft.recent.dat",
            "{\"draft:prompt\":\"hello\"}",
            NOW,
        ),
        StoreFile::new(
            "opencode.workspace.old.dat",
            "{\"workspace:layout\":\"wide\"}",
            NOW - 500 * 24 * 60 * 60 * 1000,
        ),
        StoreFile::new(
            "opencode.workspace.recent.dat",
            "{\"workspace:layout\":\"wide\"}",
            NOW,
        ),
    ];

    let result = cleanup_store_files(&files, NOW).expect(NOTE);
    assert_eq!(result.deleted, vec!["opencode.draft.old.dat".to_string()]);

    let mut remaining: Vec<String> = files
        .iter()
        .filter(|file| !result.deleted.contains(&file.name))
        .map(|file| file.name.clone())
        .collect();
    remaining.sort();
    assert_eq!(
        remaining,
        vec![
            "opencode.draft.recent.dat".to_string(),
            "opencode.workspace.old.dat".to_string(),
            "opencode.workspace.recent.dat".to_string()
        ]
    );
}

#[test]
#[ignore = "porting: desktop store cleanup not implemented"]
fn caps_scoped_stores_by_recency() {
    let files: Vec<StoreFile> = (0..102)
        .map(|index| {
            StoreFile::new(
                &format!("opencode.draft.{index}.dat"),
                "{\"draft:prompt\":\"hello\"}",
                NOW - index * 1000,
            )
        })
        .collect();

    let result = cleanup_store_files(&files, NOW).expect(NOTE);
    let mut deleted = result.deleted.clone();
    deleted.sort();
    assert_eq!(
        deleted,
        vec![
            "opencode.draft.100.dat".to_string(),
            "opencode.draft.101.dat".to_string()
        ]
    );
    assert_eq!(files.len() - result.deleted.len(), 100);
}

#[test]
#[ignore = "porting: desktop store cleanup not implemented"]
fn removes_a_scoped_store_immediately_when_it_becomes_empty() {
    let files = vec![
        StoreFile::new("opencode.draft.empty.dat", "{}", NOW),
        StoreFile::new("opencode.global.dat", "{}", NOW),
    ];

    assert!(delete_store_file_if_empty(&files, "opencode.draft.empty.dat").expect(NOTE));
    assert!(!delete_store_file_if_empty(&files, "opencode.global.dat").expect(NOTE));
}
