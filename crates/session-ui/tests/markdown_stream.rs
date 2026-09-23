//! Port of packages/session-ui/src/components/markdown-stream.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/markdown-stream.ts; see docs/TEST-PORT.md.

use opencode_session_ui::markdown_stream::{
    can_reuse_pending_block, project, stream, Block, BlockMode,
};

fn full(raw: &str) -> Block {
    Block {
        raw: raw.to_string(),
        src: raw.to_string(),
        mode: BlockMode::Full,
        language: None,
        complete: None,
    }
}

fn live(raw: &str) -> Block {
    Block {
        raw: raw.to_string(),
        src: raw.to_string(),
        mode: BlockMode::Live,
        language: None,
        complete: None,
    }
}

fn live_src(raw: &str, src: &str) -> Block {
    Block {
        raw: raw.to_string(),
        src: src.to_string(),
        mode: BlockMode::Live,
        language: None,
        complete: None,
    }
}

fn full_src(raw: &str, src: &str) -> Block {
    Block {
        raw: raw.to_string(),
        src: src.to_string(),
        mode: BlockMode::Full,
        language: None,
        complete: None,
    }
}

fn code(raw: &str, src: &str, language: &str, complete: Option<bool>) -> Block {
    Block {
        raw: raw.to_string(),
        src: src.to_string(),
        mode: BlockMode::Code,
        language: Some(language.to_string()),
        complete,
    }
}

#[test]
fn heals_incomplete_emphasis_while_streaming() {
    assert_eq!(
        stream("hello **world", true),
        vec![live_src("hello **world", "hello **world**")]
    );
    assert_eq!(
        stream("say `code", true),
        vec![live_src("say `code", "say `code`")]
    );
}

#[test]
fn keeps_incomplete_links_non_clickable_until_they_finish() {
    assert_eq!(
        stream("see [docs](https://example.com/gu", true),
        vec![live_src("see [docs](https://example.com/gu", "see docs")]
    );
}

#[test]
fn splits_an_unfinished_trailing_code_fence_from_stable_content() {
    assert_eq!(
        stream("before\n\n```ts\nconst x = 1", true),
        vec![
            full("before\n\n"),
            code("```ts\nconst x = 1", "const x = 1", "ts", None)
        ]
    );
}

#[test]
fn fully_parses_a_code_fence_once_it_closes() {
    let text = "before\n\n```ts\nconst x = 1\n```";
    assert_eq!(
        stream(text, true),
        vec![
            full("before\n\n"),
            code("```ts\nconst x = 1\n```", "const x = 1", "ts", Some(true)),
        ]
    );
}

#[test]
fn keeps_a_completed_code_fence_in_worker_rendered_code_mode_when_prose_follows() {
    assert_eq!(
        stream("```ts\nconst x = 1\n```\n\nafter", true),
        vec![
            code(
                "```ts\nconst x = 1\n```\n\n",
                "const x = 1",
                "ts",
                Some(true)
            ),
            live("after"),
        ]
    );
}

#[test]
fn freezes_completed_top_level_blocks_and_only_keeps_the_tail_live() {
    assert_eq!(
        stream("# Plan\n\nFinished paragraph.\n\n- live item", true),
        vec![
            full("# Plan\n\n"),
            full("Finished paragraph.\n\n"),
            live("- live item"),
        ]
    );
}

#[test]
fn keeps_a_growing_table_together_until_a_later_block_freezes_it() {
    assert_eq!(
        stream("| a | b |\n|---|---|\n| 1 | 2 |", true),
        vec![live("| a | b |\n|---|---|\n| 1 | 2 |")]
    );
}

#[test]
fn reprojects_non_prefix_replacements_from_current_content() {
    assert_eq!(
        stream("# Replacement\n\nNew body", true),
        vec![full("# Replacement\n\n"), live("New body")]
    );
}

#[test]
fn reprojects_truncation_without_retaining_removed_blocks() {
    assert_eq!(
        stream("Only the restored prefix", true),
        vec![live("Only the restored prefix")]
    );
}

#[test]
fn shifts_later_blocks_when_an_earlier_block_is_inserted() {
    assert_eq!(
        stream("# Inserted\n\nFirst body\n\nSecond body", true),
        vec![
            full("# Inserted\n\n"),
            full("First body\n\n"),
            live("Second body"),
        ]
    );
}

#[test]
fn keeps_reference_style_markdown_as_one_block() {
    assert_eq!(
        stream("[docs][1]\n\n[1]: https://example.com", true),
        vec![live("[docs][1]\n\n[1]: https://example.com")]
    );
}

#[test]
fn keeps_compact_and_indented_reference_definitions_with_their_uses() {
    assert_eq!(
        stream("[docs]\n\n   [docs]:/guide", true),
        vec![live("[docs]\n\n   [docs]:/guide")]
    );
}

#[test]
fn keeps_multiline_reference_definitions_with_their_uses() {
    assert_eq!(
        stream("[docs][id]\n\n[id]:\n  /guide", true),
        vec![live("[docs][id]\n\n[id]:\n  /guide")]
    );
}

#[test]
fn uses_only_the_language_portion_of_fence_metadata() {
    assert_eq!(
        stream("```ts title=example\nconst x = 1", true),
        vec![code(
            "```ts title=example\nconst x = 1",
            "const x = 1",
            "ts",
            None
        )]
    );
}

#[test]
fn preserves_trailing_newlines_in_open_code_fences() {
    assert_eq!(
        stream("```ts\nconst x = 1\n", true),
        vec![code("```ts\nconst x = 1\n", "const x = 1\n", "ts", None)]
    );
}

#[test]
fn only_reuses_pending_blocks_with_compatible_identity_and_content() {
    assert!(!can_reuse_pending_block(
        &full("First\n\n"),
        &full_src("# Inserted\n\n", "")
    ));
    assert!(can_reuse_pending_block(
        &code("```ts\none", "", "ts", None),
        &code("```ts\none two", "", "ts", None)
    ));
    assert!(can_reuse_pending_block(
        &live("partial"),
        &live_src("partial text", "")
    ));
    assert!(!can_reuse_pending_block(
        &code("```ts\none", "", "ts", None),
        &live_src("one", "")
    ));
}

#[test]
fn appends_plain_code_deltas_without_reprojecting_frozen_blocks() {
    let previous = project(None, "# Plan\n\n```ts\nconst one = 1\n", true);
    let next = project(
        Some(&previous),
        &format!("{}const two = 2\n", previous.text),
        true,
    );

    assert_eq!(next.blocks[0], previous.blocks[0]);
    assert_eq!(
        next.blocks.last().unwrap(),
        &code(
            "```ts\nconst one = 1\nconst two = 2\n",
            "const one = 1\nconst two = 2\n",
            "ts",
            None
        )
    );
}

#[test]
fn finalizes_only_the_live_tail_when_streaming_stops() {
    let live_projection = project(None, "# Plan\n\nFinished paragraph.\n\n- final item", true);
    let finalized = project(Some(&live_projection), &live_projection.text, false);

    assert_eq!(finalized.blocks[0], live_projection.blocks[0]);
    assert_eq!(finalized.blocks[1], live_projection.blocks[1]);
    assert_eq!(finalized.blocks[2], full("- final item"));
}

#[test]
fn catches_up_paced_text_before_finalizing() {
    let live_projection = project(None, "# Plan\n\nFinished paragraph.\n\n- final", true);
    let finalized = project(
        Some(&live_projection),
        &format!("{} item", live_projection.text),
        false,
    );

    assert!(can_reuse_pending_block(
        &live_projection.blocks[0],
        &finalized.blocks[0]
    ));
    assert!(can_reuse_pending_block(
        &live_projection.blocks[1],
        &finalized.blocks[1]
    ));
    assert_eq!(finalized.blocks[2], full("- final item"));
}

#[test]
fn completes_an_open_code_block_when_streaming_stops() {
    let live_projection = project(None, "```ts\nconst value = 1", true);
    let finalized = project(Some(&live_projection), &live_projection.text, false);

    assert_eq!(
        finalized.blocks,
        vec![code(
            "```ts\nconst value = 1",
            "const value = 1",
            "ts",
            Some(true)
        )]
    );
}

#[test]
fn does_not_add_a_blank_line_before_the_first_streamed_code() {
    let previous = project(None, "```ts\n", true);
    let next = project(
        Some(&previous),
        &format!("{}const x = 1", previous.text),
        true,
    );

    assert_eq!(
        next.blocks.last().unwrap(),
        &code("```ts\nconst x = 1", "const x = 1", "ts", None)
    );
}

#[test]
fn closes_code_fences_split_across_provider_deltas() {
    let open = project(None, "```ts\nconst x = 1\n", true);
    let one = project(Some(&open), &format!("{}`", open.text), true);
    let two = project(Some(&one), &format!("{}`", one.text), true);
    let closed = project(Some(&two), &format!("{}`", two.text), true);
    let prose = project(Some(&closed), &format!("{}\nafter", closed.text), true);

    assert_eq!(
        closed.blocks.last().unwrap(),
        &code("```ts\nconst x = 1\n```", "const x = 1", "ts", Some(true))
    );
    assert_eq!(
        prose.blocks,
        vec![
            code("```ts\nconst x = 1\n```\n", "const x = 1", "ts", Some(true)),
            live("after"),
        ]
    );
}

#[test]
fn closes_tilde_fences_split_across_provider_deltas() {
    let open = project(None, "~~~ts\nconst x = 1\n", true);
    let one = project(Some(&open), &format!("{}~", open.text), true);
    let two = project(Some(&one), &format!("{}~", one.text), true);
    let closed = project(Some(&two), &format!("{}~", two.text), true);

    assert_eq!(
        closed.blocks.last().unwrap(),
        &code("~~~ts\nconst x = 1\n~~~", "const x = 1", "ts", Some(true))
    );
}
