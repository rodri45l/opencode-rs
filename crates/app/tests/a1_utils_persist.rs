//! Port of packages/app/src/utils/persist.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Default)]
struct MemoryStorage {
    values: BTreeMap<String, String>,
    set_calls: usize,
    throw_on: Vec<String>,
    quota_on: Vec<String>,
}

impl MemoryStorage {
    fn get_item(&self, key: &str) -> Option<String> {
        self.values.get(key).cloned()
    }
    fn set_item(&mut self, key: &str, value: &str) {
        self.set_calls += 1;
        if self.quota_on.iter().any(|p| key.starts_with(p)) {
            return;
        }
        if self.throw_on.iter().any(|p| key.starts_with(p)) {
            return;
        }
        self.values.insert(key.to_string(), value.to_string());
    }
    fn remove_item(&mut self, key: &str) {
        self.values.remove(key);
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Target {
    storage: String,
    key: String,
    legacy: Option<Vec<String>>,
    legacy_storage_names: Option<Vec<String>>,
}

// Local stubs (fast wave): real module lands later.
fn local_storage_with_prefix(_storage: &MemoryStorage, _prefix: &str) -> ScopedStorage {
    ScopedStorage {
        prefix: String::new(),
    }
}

struct ScopedStorage {
    prefix: String,
}

impl ScopedStorage {
    fn set_item(&self, _key: &str, _value: &str) {}
    fn get_item(&self, _key: &str) -> Option<String> {
        None
    }
}

fn workspace_storage(_path: &str) -> String {
    String::new()
}

fn workspace(_path: &str, _key: &str) -> Target {
    Target {
        storage: String::new(),
        key: String::new(),
        legacy: None,
        legacy_storage_names: None,
    }
}

fn draft(_draft: &str, _key: &str) -> Target {
    Target {
        storage: String::new(),
        key: String::new(),
        legacy: None,
        legacy_storage_names: None,
    }
}

fn server_workspace(_scope: &str, _path: &str, _key: &str) -> Target {
    workspace(_path, _key)
}

fn server_global(_scope: &str, _key: &str) -> Target {
    Target {
        storage: "opencode.global.dat".into(),
        key: _key.to_string(),
        legacy: None,
        legacy_storage_names: None,
    }
}

fn normalize(_defaults: &str, _raw: &str) -> Option<String> {
    None
}

fn remove_persisted(_storage: &mut MemoryStorage, _target: &Target) {}

#[test]
#[ignore = "porting: utils/persist not implemented"]
fn does_not_cache_values_as_persisted_when_quota_write_and_eviction_fail() {
    let mut storage = MemoryStorage::default();
    storage.quota_on.push("opencode.quota".into());
    let scoped = local_storage_with_prefix(&storage, "opencode.quota.scope");
    scoped.set_item("value", r#"{"value":1}"#);
    assert_eq!(storage.get_item("opencode.quota.scope:value"), None);
    assert_eq!(scoped.get_item("value"), None);
}

#[test]
#[ignore = "porting: utils/persist not implemented"]
fn disables_only_the_failing_scope_when_storage_throws() {
    let mut storage = MemoryStorage::default();
    storage.throw_on.push("opencode.throw.scope".into());
    let bad = local_storage_with_prefix(&storage, "opencode.throw.scope");
    bad.set_item("value", r#"{"value":1}"#);
    let before = storage.set_calls;
    bad.set_item("value", r#"{"value":2}"#);
    assert_eq!(storage.set_calls, before);
    assert_eq!(bad.get_item("value"), None);

    let healthy = local_storage_with_prefix(&storage, "opencode.safe.scope");
    healthy.set_item("value", r#"{"value":3}"#);
    assert_eq!(
        storage.get_item("opencode.safe.scope:value"),
        Some(r#"{"value":3}"#.to_string())
    );
}

#[test]
#[ignore = "porting: utils/persist not implemented"]
fn normalizer_rejects_malformed_json_payloads() {
    assert_eq!(normalize(r#"{"value":"ok"}"#, r#"{"value":"\x"}"#), None);
}

#[test]
#[ignore = "porting: utils/persist not implemented"]
fn workspace_storage_sanitizes_windows_filename_characters() {
    let result = workspace_storage("C:\\Users\\foo");
    assert!(result.starts_with("opencode.workspace."));
    assert!(result.ends_with(".dat"));
    assert!(!result.contains(':') && !result.contains('\\') && !result.contains('/'));
}

#[test]
#[ignore = "porting: utils/persist not implemented"]
fn workspace_target_keeps_raw_path_storage_as_legacy_fallback() {
    let target = workspace("C:\\Users\\foo", "vcs");
    assert_eq!(target.storage, workspace_storage("C:/Users/foo"));
    assert_eq!(
        target.legacy_storage_names,
        Some(vec![workspace_storage("C:\\Users\\foo")])
    );
}

#[test]
#[ignore = "porting: utils/persist not implemented"]
fn workspace_target_keeps_backslash_storage_as_fallback_for_normalized_windows_paths() {
    let target = workspace("C:/Users/foo", "vcs");
    assert_eq!(target.storage, workspace_storage("C:/Users/foo"));
    assert_eq!(
        target.legacy_storage_names,
        Some(vec![workspace_storage("C:\\Users\\foo")])
    );
}

#[test]
#[ignore = "porting: utils/persist not implemented"]
fn draft_target_isolates_storage_per_draft_and_namespaces_keys() {
    let a = draft("draft-a", "prompt");
    let b = draft("draft-b", "prompt");
    assert_eq!(a.key, "draft:prompt");
    assert_ne!(a.storage, b.storage);
    assert_ne!(a.storage, workspace("/home/luke/repo", "prompt").storage);
}

#[test]
#[ignore = "porting: utils/persist not implemented"]
fn removes_draft_storage_when_removing_persisted_target() {
    let mut storage = MemoryStorage::default();
    let target = draft("draft-a", "prompt");
    storage.set_item(
        &format!("{}:{}", target.storage, target.key),
        r#"{"value":1}"#,
    );
    remove_persisted(&mut storage, &target);
    assert_eq!(
        storage.get_item(&format!("{}:{}", target.storage, target.key)),
        None
    );
}

#[test]
#[ignore = "porting: utils/persist not implemented"]
fn server_workspace_target_preserves_local_storage_and_isolates_remote_storage() {
    let local = server_workspace("local", "/home/luke/repo", "prompt");
    let windows = server_workspace("https://windows.example", "/home/luke/repo", "prompt");
    let debian = server_workspace("https://debian.example", "/home/luke/repo", "prompt");
    assert_eq!(local, workspace("/home/luke/repo", "prompt"));
    assert_ne!(windows.storage, local.storage);
    assert_ne!(debian.storage, local.storage);
    assert_ne!(debian.storage, windows.storage);
    assert_eq!(windows.legacy_storage_names, None);
    assert_eq!(debian.legacy_storage_names, None);
}

#[test]
#[ignore = "porting: utils/persist not implemented"]
fn server_global_target_preserves_local_key_and_isolates_remote_keys() {
    assert_eq!(
        server_global("local", "notification"),
        Target {
            storage: "opencode.global.dat".into(),
            key: "notification".into(),
            legacy: None,
            legacy_storage_names: None,
        }
    );
    assert_eq!(
        server_global("https://debian.example", "notification"),
        Target {
            storage: "opencode.global.dat".into(),
            key: "https://debian.example\0notification".into(),
            legacy: None,
            legacy_storage_names: None,
        }
    );
}

#[test]
#[ignore = "porting: utils/persist not implemented"]
fn server_global_target_cannot_collide_when_scope_and_key_contain_colons() {
    assert_ne!(server_global("a:b", "c"), server_global("a", "b:c"));
}
