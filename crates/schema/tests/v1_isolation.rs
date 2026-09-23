//! Port of packages/schema/test/v1-isolation.test.ts (upstream 18ef3cc).

use std::any::TypeId;
use std::collections::BTreeSet;

#[test]
fn compatibility_entrypoints_preserve_isolated_v1_schema_identity() {
    assert_eq!(
        TypeId::of::<opencode_schema::legacy_event::LegacyEvent>(),
        TypeId::of::<opencode_schema::v1::legacy_event::LegacyEvent>()
    );
    assert_eq!(
        TypeId::of::<opencode_schema::permission_v1::PermissionV1>(),
        TypeId::of::<opencode_schema::v1::permission::PermissionV1>()
    );
    assert_eq!(
        TypeId::of::<opencode_schema::question_v1::QuestionV1>(),
        TypeId::of::<opencode_schema::v1::question::QuestionV1>()
    );
    assert_eq!(
        TypeId::of::<opencode_schema::session_v1::SessionV1>(),
        TypeId::of::<opencode_schema::v1::session::SessionV1>()
    );
}

#[test]
fn current_source_does_not_import_the_v1_subtree_directly() {
    let allowed: BTreeSet<&str> = [
        "legacy_event.rs",
        "permission_v1.rs",
        "question_v1.rs",
        "session_v1.rs",
    ]
    .into_iter()
    .collect();

    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
    let mut direct_imports = Vec::new();
    for entry in std::fs::read_dir(dir).expect("read src dir") {
        let path = entry.expect("dir entry").path();
        let Some(file) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        if allowed.contains(file) {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("read source");
        if source.contains("crate::v1::") {
            direct_imports.push(file.to_string());
        }
    }

    assert!(
        direct_imports.is_empty(),
        "direct v1 imports: {direct_imports:?}"
    );
}
