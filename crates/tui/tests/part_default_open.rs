//! Port of packages/session-ui/src/components/part-default-open.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/part-default-open.ts:
//! edited files default open when enabled, deletion-only edits/patches collapse, mixed
//! patches stay open, and shell defaults are preserved.
//! Red-first: the part-default-open rule is not implemented.

#[allow(dead_code)]
mod part_default_open {
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

    pub const NOTE: &str = "porting: session-ui part-default-open not implemented";

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct PatchFile {
        pub file_path: String,
        pub kind: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PartMetadata {
        Edit { additions: i64, deletions: i64 },
        ApplyPatch { files: Vec<PatchFile> },
        None,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ToolPart {
        pub tool: String,
        pub metadata: PartMetadata,
    }

    pub fn part_default_open(
        _part: &ToolPart,
        _default_open: bool,
        _expand_edits: bool,
    ) -> PortResult<bool> {
        Err(NotImplemented(NOTE))
    }
}

use part_default_open::{part_default_open, PartMetadata, PatchFile, ToolPart, NOTE};

fn tool(name: &str, metadata: PartMetadata) -> ToolPart {
    ToolPart {
        tool: name.into(),
        metadata,
    }
}

fn patch(file_path: &str, kind: &str) -> PatchFile {
    PatchFile {
        file_path: file_path.into(),
        kind: kind.into(),
    }
}

#[test]
#[ignore = "porting: session-ui part-default-open not implemented"]
fn keeps_edited_files_expanded_when_enabled() {
    assert!(part_default_open(
        &tool(
            "edit",
            PartMetadata::Edit {
                additions: 1,
                deletions: 1
            }
        ),
        false,
        true
    )
    .expect(NOTE));
}

#[test]
#[ignore = "porting: session-ui part-default-open not implemented"]
fn collapses_deletion_only_edits_when_enabled() {
    assert!(!part_default_open(
        &tool(
            "edit",
            PartMetadata::Edit {
                additions: 0,
                deletions: 1_200
            }
        ),
        false,
        true
    )
    .expect(NOTE));
}

#[test]
#[ignore = "porting: session-ui part-default-open not implemented"]
fn collapses_patches_containing_only_deleted_files_when_enabled() {
    let part = tool(
        "apply_patch",
        PartMetadata::ApplyPatch {
            files: vec![patch("one.ts", "delete"), patch("two.ts", "delete")],
        },
    );
    assert!(!part_default_open(&part, false, true).expect(NOTE));
}

#[test]
#[ignore = "porting: session-ui part-default-open not implemented"]
fn keeps_mixed_patches_expanded_when_enabled() {
    let part = tool(
        "apply_patch",
        PartMetadata::ApplyPatch {
            files: vec![patch("one.ts", "delete"), patch("two.ts", "update")],
        },
    );
    assert!(part_default_open(&part, false, true).expect(NOTE));
}

#[test]
#[ignore = "porting: session-ui part-default-open not implemented"]
fn preserves_shell_defaults() {
    assert!(part_default_open(&tool("shell", PartMetadata::None), true, false).expect(NOTE));
}
