//! Port of packages/schema/test/contract-hygiene.test.ts (upstream 18ef3cc).

use opencode_schema::schema::FiniteFromString;
use opencode_schema::session_todo::Info;
use opencode_schema::{identifiers, PtyId, QuestionId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct OptionalValue {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    value: Option<FiniteFromString>,
}

#[test]
fn optional_properties_preserve_transformations_and_omit_undefined_while_encoding() {
    let decoded: OptionalValue = serde_json::from_str(r#"{"value":"1"}"#).unwrap();
    assert_eq!(decoded.value, Some(FiniteFromString(1.0)));

    let encoded = serde_json::to_value(OptionalValue {
        value: Some(FiniteFromString(1.0)),
    })
    .unwrap();
    assert_eq!(encoded, serde_json::json!({ "value": "1" }));

    let omitted = serde_json::to_value(OptionalValue { value: None }).unwrap();
    assert_eq!(omitted, serde_json::json!({}));
}

#[test]
fn todo_status_and_priority_preserve_arbitrary_strings() {
    let decoded: Info =
        serde_json::from_str(r#"{"content":"ship","status":"waiting","priority":"urgent"}"#)
            .unwrap();
    assert_eq!(
        decoded,
        Info {
            content: "ship".into(),
            status: "waiting".into(),
            priority: "urgent".into(),
        }
    );
}

#[test]
fn current_id_constructors_expose_create() {
    assert!(QuestionId::create().as_str().starts_with("que_"));
    assert!(PtyId::create().as_str().starts_with("pty_"));
}

#[test]
fn reusable_public_identifiers_are_stable_and_unique() {
    let identifiers = identifiers::REUSABLE;
    assert!(identifiers.iter().all(|identifier| !identifier.is_empty()));
    let unique: BTreeSet<&&str> = identifiers.iter().collect();
    assert_eq!(unique.len(), identifiers.len());
}

#[test]
fn current_source_avoids_any_and_mutable_contract_wrappers() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
    for entry in std::fs::read_dir(dir).expect("read src dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("read source");
        assert!(
            !source.contains("Schema.Any"),
            "{} uses Schema.Any",
            path.display()
        );
        assert!(
            !source.contains("Schema.mutable"),
            "{} uses Schema.mutable",
            path.display()
        );
    }
}
