//! Contract tests for the error taxonomy.
//!
//! The fixture is parsed from the reference `errors.ts`, so this pins every
//! tag, status code, and payload field to the reference implementation.

use opencode_protocol::{ApiError, ERROR_SPECS};
use serde_json::Value;

fn fixture() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/errors.json"
    );
    let raw = std::fs::read_to_string(path).expect("read errors fixture");
    serde_json::from_str(&raw).expect("parse errors fixture")
}

/// One constructed instance of every variant.
fn sample_errors() -> Vec<ApiError> {
    vec![
        ApiError::InvalidRequest {
            message: "bad".into(),
            kind: None,
            field: None,
        },
        ApiError::Unauthorized {
            message: "no".into(),
        },
        ApiError::Forbidden {
            message: "nope".into(),
        },
        ApiError::Conflict {
            message: "busy".into(),
            resource: None,
        },
        ApiError::ServiceUnavailable {
            message: "down".into(),
            service: None,
        },
        ApiError::Unknown {
            message: "oops".into(),
            reference: None,
        },
        ApiError::ProviderNotFound {
            provider_id: "openai".into(),
            message: "missing".into(),
        },
        ApiError::SessionNotFound {
            session_id: "ses_1".into(),
            message: "missing".into(),
        },
        ApiError::MessageNotFound {
            session_id: "ses_1".into(),
            message_id: "msg_1".into(),
            message: "missing".into(),
        },
        ApiError::PermissionNotFound {
            request_id: "per_1".into(),
            message: "missing".into(),
        },
        ApiError::QuestionNotFound {
            request_id: "que_1".into(),
            message: "missing".into(),
        },
        ApiError::PtyNotFound {
            pty_id: "pty_1".into(),
            message: "missing".into(),
        },
        ApiError::InvalidCursor {
            message: "bad cursor".into(),
        },
    ]
}

#[test]
fn variant_count_matches_the_contract() {
    let fx = fixture();
    let expected = fx["error_count"].as_u64().unwrap() as usize;
    assert_eq!(
        expected,
        ERROR_SPECS.len(),
        "spec table and fixture disagree"
    );
    assert_eq!(
        sample_errors().len(),
        expected,
        "a variant is not represented"
    );
}

#[test]
fn spec_table_matches_the_fixture() {
    let fx = fixture();
    for entry in fx["errors"].as_array().unwrap() {
        let tag = entry["tag"].as_str().unwrap();
        let spec = ERROR_SPECS
            .iter()
            .find(|s| s.tag == tag)
            .unwrap_or_else(|| panic!("no spec for {tag}"));

        assert_eq!(
            spec.status,
            entry["status"].as_u64().unwrap() as u16,
            "status for {tag}"
        );

        let expected: Vec<(String, bool)> = entry["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| {
                (
                    f["name"].as_str().unwrap().to_string(),
                    f["required"].as_bool().unwrap(),
                )
            })
            .collect();
        let actual: Vec<(String, bool)> = spec
            .fields
            .iter()
            .map(|f| (f.name.to_string(), f.required))
            .collect();
        assert_eq!(actual, expected, "fields for {tag}");
    }
}

#[test]
fn every_variant_reports_its_spec_tag_and_status() {
    for error in sample_errors() {
        let spec = ERROR_SPECS
            .iter()
            .find(|s| s.tag == error.tag())
            .unwrap_or_else(|| panic!("no spec for {}", error.tag()));
        assert_eq!(error.status(), spec.status, "status for {}", error.tag());
    }
}

#[test]
fn required_fields_are_present_and_optional_fields_omitted() {
    for error in sample_errors() {
        let value = serde_json::to_value(&error).unwrap();
        assert_eq!(value["_tag"], error.tag());

        let spec = ERROR_SPECS.iter().find(|s| s.tag == error.tag()).unwrap();
        for field in spec.fields.iter().filter(|f| f.required) {
            assert!(
                value.get(field.name).is_some(),
                "{} is missing required field {}",
                error.tag(),
                field.name
            );
        }
        // Optional fields are unset in every sample and must be omitted.
        for field in spec.fields.iter().filter(|f| !f.required) {
            assert!(
                value.get(field.name).is_none(),
                "{} unexpectedly serialized optional field {}",
                error.tag(),
                field.name
            );
        }
    }
}
