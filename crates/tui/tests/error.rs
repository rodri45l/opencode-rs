//! Port of packages/tui/test/util/error.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/util/error.ts; see docs/TEST-PORT.md.

use opencode_tui::error::{error_data, error_format, error_message, ErrorValue};

#[test]
fn formats_native_error_instances() {
    let error = ErrorValue::Native {
        name: "Error".to_string(),
        message: "boom".to_string(),
    };
    assert_eq!(error_message(&error), "boom");
    assert!(error_format(&error).contains("boom"));

    let data = error_data(&error);
    assert_eq!(data.type_name, "Error");
    assert_eq!(data.message, "boom");
    assert!(data.formatted.contains("boom"));
}

#[test]
fn extracts_message_from_record_like_values() {
    let error = ErrorValue::Record {
        message: "bad input".to_string(),
        code: Some("E_BAD".to_string()),
    };
    assert_eq!(error_message(&error), "bad input");

    let data = error_data(&error);
    assert_eq!(data.message, "bad input");
    assert_eq!(data.code.as_deref(), Some("E_BAD"));
}

#[test]
fn never_returns_bare_empty_object_for_opaque_object_errors() {
    let empty = ErrorValue::Opaque {
        name: "Object".to_string(),
    };
    assert_ne!(error_format(&empty), "{}");
    assert!(error_format(&empty).contains("no message"));

    let opaque = ErrorValue::Opaque {
        name: "OpaqueError".to_string(),
    };
    assert_ne!(error_format(&opaque), "{}");
    assert!(error_format(&opaque).contains("OpaqueError"));
}

#[test]
fn handles_opaque_throwables_with_custom_to_string() {
    let error = ErrorValue::Custom {
        text: "ResolveMessage: Cannot resolve module".to_string(),
    };
    assert_eq!(
        error_message(&error),
        "ResolveMessage: Cannot resolve module"
    );

    let data = error_data(&error);
    assert_eq!(data.message, "ResolveMessage: Cannot resolve module");
    assert!(data.formatted.contains("ResolveMessage"));
}
