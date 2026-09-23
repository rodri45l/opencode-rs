//! Port of packages/core/test/reference-guidance.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: available references are listed in the system context with
//! their name, path and description; the block is omitted when no references are
//! available or when none carry a description. Re-derived as a pure renderer;
//! the `Layer`/`SystemContext` wiring is dropped.

use opencode_core::path::AbsolutePath;
use opencode_core::reference_guidance::{ReferenceGuidance, ReferenceInfo, ReferenceSource};

fn reference(name: &str, description: Option<&str>) -> ReferenceInfo {
    let path = AbsolutePath::new("/docs");
    ReferenceInfo {
        name: name.into(),
        path: path.clone(),
        description: description.map(str::to_string),
        hidden: false,
        source: ReferenceSource::Local {
            path,
            description: description.map(str::to_string),
        },
    }
}

#[test]
fn lists_available_references_in_the_system_context() {
    let baseline =
        ReferenceGuidance::render(&[reference("docs", Some("Use for product documentation"))])
            .unwrap();

    assert!(baseline.contains("<available_references>"));
    assert!(baseline.contains("<name>docs</name>"));
    assert!(baseline.contains("<path>/docs</path>"));
    assert!(baseline.contains("<description>Use for product documentation</description>"));
}

#[test]
fn omits_guidance_when_no_references_are_available() {
    assert_eq!(ReferenceGuidance::render(&[]).unwrap(), "");
}

#[test]
fn omits_references_without_descriptions() {
    assert_eq!(
        ReferenceGuidance::render(&[reference("docs", None)]).unwrap(),
        ""
    );
}
