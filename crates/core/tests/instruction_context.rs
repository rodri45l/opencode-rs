//! Port of packages/core/test/instruction-context.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: global, package and project `AGENTS.md` files render as one
//! aggregate context in discovery order, and an empty `AGENTS.md` still counts as
//! available context. Re-derived as a pure renderer over discovered files; the
//! filesystem discovery and `SystemContext` reconcile cases are dropped.

use opencode_core::instruction_context::{InstructionContext, InstructionFile};
use opencode_core::path::AbsolutePath;

const NOTE: &str = "porting: instruction context not implemented";

#[test]
#[ignore = "porting: instruction context not implemented"]
fn renders_discovered_instruction_files_as_one_aggregate_context() {
    let baseline = InstructionContext::render(&[
        InstructionFile {
            path: AbsolutePath::new("/global/AGENTS.md"),
            content: "global".into(),
        },
        InstructionFile {
            path: AbsolutePath::new("/repo/packages/core/AGENTS.md"),
            content: "package".into(),
        },
        InstructionFile {
            path: AbsolutePath::new("/repo/AGENTS.md"),
            content: "project".into(),
        },
    ])
    .expect(NOTE);

    assert_eq!(
        baseline,
        [
            "Instructions from: /global/AGENTS.md\nglobal",
            "Instructions from: /repo/packages/core/AGENTS.md\npackage",
            "Instructions from: /repo/AGENTS.md\nproject",
        ]
        .join("\n\n")
    );
}

#[test]
#[ignore = "porting: instruction context not implemented"]
fn keeps_an_empty_agents_md_as_available_context() {
    let baseline = InstructionContext::render(&[InstructionFile {
        path: AbsolutePath::new("/tmp/AGENTS.md"),
        content: String::new(),
    }])
    .expect(NOTE);

    assert_eq!(baseline, "Instructions from: /tmp/AGENTS.md\n");
}
