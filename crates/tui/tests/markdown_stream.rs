//! Port of packages/session-ui/src/components/markdown-stream.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/markdown-stream.ts and
//! markdown-projection.ts: streaming heals incomplete emphasis/links, splits open code
//! fences, freezes completed top-level blocks, keeps reference definitions with their
//! uses, and reuses only compatible pending blocks while appending code deltas.
//! Red-first: the markdown stream projection is not implemented.

#[allow(dead_code)]
mod markdown_stream {
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

    pub const NOTE: &str = "porting: session-ui markdown stream not implemented";

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BlockMode {
        Full,
        Live,
        Code,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Block {
        pub raw: String,
        pub src: String,
        pub mode: BlockMode,
        pub language: Option<String>,
        pub complete: bool,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Projection {
        pub text: String,
        pub blocks: Vec<Block>,
    }

    pub fn stream(_text: &str, _streaming: bool) -> PortResult<Vec<Block>> {
        Err(NotImplemented(NOTE))
    }

    pub fn project(
        _previous: Option<&Projection>,
        _text: &str,
        _streaming: bool,
    ) -> PortResult<Projection> {
        Err(NotImplemented(NOTE))
    }

    pub fn can_reuse_pending_block(_previous: &Block, _next: &Block) -> PortResult<bool> {
        Err(NotImplemented(NOTE))
    }
}

use markdown_stream::{can_reuse_pending_block, project, stream, Block, BlockMode, NOTE};

fn full(raw: &str) -> Block {
    Block {
        raw: raw.into(),
        src: raw.into(),
        mode: BlockMode::Full,
        language: None,
        complete: false,
    }
}

fn live(raw: &str) -> Block {
    Block {
        raw: raw.into(),
        src: raw.into(),
        mode: BlockMode::Live,
        language: None,
        complete: false,
    }
}

fn live_src(raw: &str, src: &str) -> Block {
    Block {
        raw: raw.into(),
        src: src.into(),
        mode: BlockMode::Live,
        language: None,
        complete: false,
    }
}

fn code(raw: &str, src: &str, language: &str) -> Block {
    Block {
        raw: raw.into(),
        src: src.into(),
        mode: BlockMode::Code,
        language: Some(language.into()),
        complete: false,
    }
}

fn code_complete(raw: &str, src: &str, language: &str) -> Block {
    Block {
        raw: raw.into(),
        src: src.into(),
        mode: BlockMode::Code,
        language: Some(language.into()),
        complete: true,
    }
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn heals_incomplete_emphasis_while_streaming() {
    assert_eq!(
        stream("hello **world", true).expect(NOTE),
        vec![live_src("hello **world", "hello **world**")]
    );
    assert_eq!(
        stream("say `code", true).expect(NOTE),
        vec![live_src("say `code", "say `code`")]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn keeps_incomplete_links_non_clickable_until_they_finish() {
    assert_eq!(
        stream("see [docs](https://example.com/gu", true).expect(NOTE),
        vec![live_src("see [docs](https://example.com/gu", "see docs")]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn splits_an_unfinished_trailing_code_fence_from_stable_content() {
    assert_eq!(
        stream("before\n\n```ts\nconst x = 1", true).expect(NOTE),
        vec![
            full("before\n\n"),
            code("```ts\nconst x = 1", "const x = 1", "ts")
        ]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn fully_parses_a_code_fence_once_it_closes() {
    assert_eq!(
        stream("before\n\n```ts\nconst x = 1\n```", true).expect(NOTE),
        vec![
            full("before\n\n"),
            code_complete("```ts\nconst x = 1\n```", "const x = 1", "ts")
        ]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn keeps_a_completed_code_fence_in_worker_rendered_code_mode_when_prose_follows() {
    assert_eq!(
        stream("```ts\nconst x = 1\n```\n\nafter", true).expect(NOTE),
        vec![
            code_complete("```ts\nconst x = 1\n```\n\n", "const x = 1", "ts"),
            live("after")
        ]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn freezes_completed_top_level_blocks_and_only_keeps_the_tail_live() {
    assert_eq!(
        stream("# Plan\n\nFinished paragraph.\n\n- live item", true).expect(NOTE),
        vec![
            full("# Plan\n\n"),
            full("Finished paragraph.\n\n"),
            live("- live item")
        ]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn keeps_a_growing_table_together_until_a_later_block_freezes_it() {
    assert_eq!(
        stream("| a | b |\n|---|---|\n| 1 | 2 |", true).expect(NOTE),
        vec![live("| a | b |\n|---|---|\n| 1 | 2 |")]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn reprojects_non_prefix_replacements_from_current_content() {
    assert_eq!(
        stream("# Replacement\n\nNew body", true).expect(NOTE),
        vec![full("# Replacement\n\n"), live("New body")]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn reprojects_truncation_without_retaining_removed_blocks() {
    assert_eq!(
        stream("Only the restored prefix", true).expect(NOTE),
        vec![live("Only the restored prefix")]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn shifts_later_blocks_when_an_earlier_block_is_inserted() {
    assert_eq!(
        stream("# Inserted\n\nFirst body\n\nSecond body", true).expect(NOTE),
        vec![
            full("# Inserted\n\n"),
            full("First body\n\n"),
            live("Second body")
        ]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn keeps_reference_style_markdown_as_one_block() {
    assert_eq!(
        stream("[docs][1]\n\n[1]: https://example.com", true).expect(NOTE),
        vec![live("[docs][1]\n\n[1]: https://example.com")]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn keeps_compact_and_indented_reference_definitions_with_their_uses() {
    assert_eq!(
        stream("[docs]\n\n   [docs]:/guide", true).expect(NOTE),
        vec![live("[docs]\n\n   [docs]:/guide")]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn keeps_multiline_reference_definitions_with_their_uses() {
    assert_eq!(
        stream("[docs][id]\n\n[id]:\n  /guide", true).expect(NOTE),
        vec![live("[docs][id]\n\n[id]:\n  /guide")]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn uses_only_the_language_portion_of_fence_metadata() {
    assert_eq!(
        stream("```ts title=example\nconst x = 1", true).expect(NOTE),
        vec![code(
            "```ts title=example\nconst x = 1",
            "const x = 1",
            "ts"
        )]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn preserves_trailing_newlines_in_open_code_fences() {
    assert_eq!(
        stream("```ts\nconst x = 1\n", true).expect(NOTE),
        vec![code("```ts\nconst x = 1\n", "const x = 1\n", "ts")]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn only_reuses_pending_blocks_with_compatible_identity_and_content() {
    assert!(!can_reuse_pending_block(
        &full("First\n\n"),
        &Block {
            raw: "# Inserted\n\n".into(),
            src: String::new(),
            mode: BlockMode::Full,
            language: None,
            complete: false,
        }
    )
    .expect(NOTE));
    assert!(can_reuse_pending_block(
        &code("```ts\none", "", "ts"),
        &Block {
            raw: "```ts\none two".into(),
            src: String::new(),
            mode: BlockMode::Code,
            language: Some("ts".into()),
            complete: false,
        }
    )
    .expect(NOTE));
    assert!(can_reuse_pending_block(
        &live("partial"),
        &Block {
            raw: "partial text".into(),
            src: String::new(),
            mode: BlockMode::Live,
            language: None,
            complete: false,
        }
    )
    .expect(NOTE));
    assert!(!can_reuse_pending_block(
        &code("```ts\none", "", "ts"),
        &Block {
            raw: "one".into(),
            src: String::new(),
            mode: BlockMode::Live,
            language: None,
            complete: false,
        }
    )
    .expect(NOTE));
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn appends_plain_code_deltas_without_reprojecting_frozen_blocks() {
    let previous = project(None, "# Plan\n\n```ts\nconst one = 1\n", true).expect(NOTE);
    let next = project(
        Some(&previous),
        &format!("{}const two = 2\n", previous.text),
        true,
    )
    .expect(NOTE);

    assert_eq!(next.blocks[0], previous.blocks[0]);
    assert_eq!(
        next.blocks.last().cloned(),
        Some(code(
            "```ts\nconst one = 1\nconst two = 2\n",
            "const one = 1\nconst two = 2\n",
            "ts"
        ))
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn finalizes_only_the_live_tail_when_streaming_stops() {
    let live = project(None, "# Plan\n\nFinished paragraph.\n\n- final item", true).expect(NOTE);
    let final_projection = project(Some(&live), &live.text, false).expect(NOTE);

    assert_eq!(final_projection.blocks[0], live.blocks[0]);
    assert_eq!(final_projection.blocks[1], live.blocks[1]);
    assert_eq!(final_projection.blocks[2], full("- final item"));
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn catches_up_paced_text_before_finalizing() {
    let live = project(None, "# Plan\n\nFinished paragraph.\n\n- final", true).expect(NOTE);
    let final_projection = project(Some(&live), &format!("{} item", live.text), false).expect(NOTE);

    assert!(can_reuse_pending_block(&live.blocks[0], &final_projection.blocks[0]).expect(NOTE));
    assert!(can_reuse_pending_block(&live.blocks[1], &final_projection.blocks[1]).expect(NOTE));
    assert_eq!(final_projection.blocks[2], full("- final item"));
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn completes_an_open_code_block_when_streaming_stops() {
    let live = project(None, "```ts\nconst value = 1", true).expect(NOTE);
    let final_projection = project(Some(&live), &live.text, false).expect(NOTE);

    assert_eq!(
        final_projection.blocks,
        vec![code_complete(
            "```ts\nconst value = 1",
            "const value = 1",
            "ts"
        )]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn does_not_add_a_blank_line_before_the_first_streamed_code() {
    let previous = project(None, "```ts\n", true).expect(NOTE);
    let next = project(
        Some(&previous),
        &format!("{}const x = 1", previous.text),
        true,
    )
    .expect(NOTE);

    assert_eq!(
        next.blocks.last().cloned(),
        Some(code("```ts\nconst x = 1", "const x = 1", "ts"))
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn closes_code_fences_split_across_provider_deltas() {
    let open = project(None, "```ts\nconst x = 1\n", true).expect(NOTE);
    let one = project(Some(&open), &format!("{}`", open.text), true).expect(NOTE);
    let two = project(Some(&one), &format!("{}`", one.text), true).expect(NOTE);
    let closed = project(Some(&two), &format!("{}`", two.text), true).expect(NOTE);
    let prose = project(Some(&closed), &format!("{}\nafter", closed.text), true).expect(NOTE);

    assert_eq!(
        closed.blocks.last().cloned(),
        Some(code_complete(
            "```ts\nconst x = 1\n```",
            "const x = 1",
            "ts"
        ))
    );
    assert_eq!(
        prose.blocks,
        vec![
            code_complete("```ts\nconst x = 1\n```\n", "const x = 1", "ts"),
            live("after")
        ]
    );
}

#[test]
#[ignore = "porting: session-ui markdown stream not implemented"]
fn closes_tilde_fences_split_across_provider_deltas() {
    let open = project(None, "~~~ts\nconst x = 1\n", true).expect(NOTE);
    let one = project(Some(&open), &format!("{}~", open.text), true).expect(NOTE);
    let two = project(Some(&one), &format!("{}~", one.text), true).expect(NOTE);
    let closed = project(Some(&two), &format!("{}~", two.text), true).expect(NOTE);

    assert_eq!(
        closed.blocks.last().cloned(),
        Some(code_complete(
            "~~~ts\nconst x = 1\n~~~",
            "const x = 1",
            "ts"
        ))
    );
}
