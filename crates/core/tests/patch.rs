//! Port of packages/core/test/patch.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: add/update/delete hunk parsing, heredoc stripping, fuzzy
//! derivation with BOM preservation, EOF-anchored matching, and malformed-body
//! rejection.

use opencode_core::patch::{Patch, PatchChunk, PatchHunk, PatchUpdate};
use opencode_core::CoreError;

fn chunk(
    old: &[&str],
    new: &[&str],
    context: Option<&str>,
    end_of_file: Option<bool>,
) -> PatchChunk {
    PatchChunk {
        old_lines: old.iter().map(|line| (*line).to_string()).collect(),
        new_lines: new.iter().map(|line| (*line).to_string()).collect(),
        change_context: context.map(str::to_string),
        end_of_file,
    }
}

#[test]
fn parses_add_update_and_delete_hunks() {
    let parsed = Patch::parse(
        "*** Begin Patch\n*** Add File: add.txt\n+added\n*** Update File: update.txt\n@@ section\n-old\n+new\n*** Delete File: delete.txt\n*** End Patch",
    )
    .unwrap();

    assert_eq!(
        parsed,
        vec![
            PatchHunk::Add {
                path: "add.txt".into(),
                contents: "added".into(),
            },
            PatchHunk::Update {
                path: "update.txt".into(),
                chunks: vec![chunk(&["old"], &["new"], Some("section"), None)],
                move_path: None,
            },
            PatchHunk::Delete {
                path: "delete.txt".into(),
            },
        ]
    );
}

#[test]
fn strips_a_heredoc_wrapper() {
    let parsed = Patch::parse(
        "cat <<'EOF'\n*** Begin Patch\n*** Add File: add.txt\n+added\n*** End Patch\nEOF",
    )
    .unwrap();

    assert_eq!(
        parsed,
        vec![PatchHunk::Add {
            path: "add.txt".into(),
            contents: "added".into(),
        }]
    );
}

#[test]
fn derives_fuzzy_line_updates_while_preserving_bom() {
    let update = Patch::derive(
        "update.txt",
        vec![chunk(&["  old   "], &["new"], None, None)],
        "\u{feff}old\n",
    )
    .unwrap();

    assert_eq!(
        update,
        PatchUpdate {
            content: "new\n".into(),
            bom: true,
        }
    );
    assert_eq!(
        Patch::join_bom(&update.content, update.bom).unwrap(),
        "\u{feff}new\n"
    );
}

#[test]
fn matches_eof_anchored_chunks_from_the_end() {
    let derived = Patch::derive(
        "update.txt",
        vec![chunk(
            &["marker", "end"],
            &["marker changed", "end"],
            None,
            Some(true),
        )],
        "marker\nmiddle\nmarker\nend\n",
    )
    .unwrap();

    assert_eq!(derived.content, "marker\nmiddle\nmarker changed\nend\n");
}

#[test]
fn parses_the_eof_marker_inside_update_chunks() {
    let parsed = Patch::parse(
        "*** Begin Patch\n*** Update File: update.txt\n@@\n-last\n+end\n*** End of File\n*** End Patch",
    )
    .unwrap();

    assert_eq!(
        parsed,
        vec![PatchHunk::Update {
            path: "update.txt".into(),
            chunks: vec![chunk(&["last"], &["end"], None, Some(true))],
            move_path: None,
        }]
    );
}

#[test]
fn rejects_malformed_hunk_bodies() {
    let invalid_add =
        Patch::parse("*** Begin Patch\n*** Add File: add.txt\nmissing plus\n*** End Patch")
            .unwrap_err();
    assert!(
        matches!(invalid_add, CoreError::Message(message) if message.contains("Invalid add file line"))
    );

    let empty_update =
        Patch::parse("*** Begin Patch\n*** Update File: update.txt\n*** End Patch").unwrap_err();
    assert!(
        matches!(empty_update, CoreError::Message(message) if message.contains("expected at least one @@ chunk"))
    );

    let invalid_delete = Patch::parse(
        "*** Begin Patch\n*** Delete File: delete.txt\nunexpected body\n*** End Patch",
    )
    .unwrap_err();
    assert!(
        matches!(invalid_delete, CoreError::Message(message) if message.contains("Invalid patch line"))
    );
}
