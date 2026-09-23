//! Port of packages/session-ui/src/v2/components/prompt-input/store.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/v2/components/prompt-input/store.ts; see docs/TEST-PORT.md.

use opencode_session_ui::prompt_input_store::PromptStore;
use opencode_session_ui::prompt_types::{
    ContextItems, PersistedState, PromptComment, PromptModel, PromptPart,
};

fn text(content: &str, start: usize, end: usize) -> PromptPart {
    PromptPart::Text {
        content: content.to_string(),
        start,
        end,
    }
}

fn file(path: &str, content: &str, start: usize, end: usize) -> PromptPart {
    PromptPart::File {
        path: path.to_string(),
        content: content.to_string(),
        start,
        end,
    }
}

fn image() -> PromptPart {
    PromptPart::Image {
        id: "attachment-1".to_string(),
        filename: "notes.txt".to_string(),
        mime: "text/plain".to_string(),
        blob_id: "a".to_string(),
        blob_url: "blob:a".to_string(),
    }
}

fn model() -> PromptModel {
    PromptModel {
        provider_id: "anthropic".to_string(),
        model_id: "claude-sonnet".to_string(),
        variant: None,
    }
}

#[test]
fn accepts_an_accessor_for_the_backing_store() {
    let state = PersistedState {
        prompt: vec![text("", 0, 0)],
        cursor: Some(0),
        model: None,
        context: ContextItems::default(),
    };
    let mut store = PromptStore::new(state);

    store.set_text("accessed");

    assert_eq!(store.state().prompt, vec![text("accessed", 0, 8)]);
    assert_eq!(store.state().cursor, Some(8));
}

#[test]
fn updates_prompt_text_and_cursor_together_while_preserving_attachments() {
    let state = PersistedState {
        prompt: vec![text("old", 0, 3), image()],
        cursor: Some(3),
        model: Some(model()),
        context: ContextItems::default(),
    };
    let mut store = PromptStore::new(state);

    store.set_text("updated");

    assert_eq!(store.state().prompt, vec![text("updated", 0, 7), image()]);
    assert_eq!(store.state().cursor, Some(7));
}

#[test]
fn inserts_text_without_flattening_structured_mentions() {
    let state = PersistedState {
        prompt: vec![
            text("A ", 0, 2),
            file("one", "@one", 2, 6),
            text(" B", 6, 8),
        ],
        cursor: Some(2),
        model: None,
        context: ContextItems::default(),
    };
    let mut store = PromptStore::new(state);

    store.add_text("X\nY");

    assert_eq!(
        store.state().prompt,
        vec![
            text("A X\nY", 0, 5),
            file("one", "@one", 5, 9),
            text(" B", 9, 11),
        ]
    );
    assert_eq!(store.state().cursor, Some(5));
}

#[test]
fn mutates_context_attachments_and_model_through_shared_actions() {
    let state = PersistedState {
        prompt: vec![text("old", 0, 3), image()],
        cursor: Some(3),
        model: Some(model()),
        context: ContextItems::default(),
    };
    let mut store = PromptStore::new(state);
    let context = PromptComment {
        key: "file:src/index.ts".to_string(),
        path: "src/index.ts".to_string(),
    };

    store.add_context(context.clone());
    store.add_context(context.clone());
    store.add_mention(file("src/app.ts", "@src/app.ts", 0, 0));
    store.remove_attachment("attachment-1");
    store.set_variant(Some("thinking".to_string()));

    assert_eq!(store.state().context.items, vec![context.clone()]);
    assert_eq!(
        store.state().prompt,
        vec![
            text("old", 0, 3),
            file("src/app.ts", "@src/app.ts", 3, 14),
            text(" ", 14, 15),
        ]
    );
    assert_eq!(
        store
            .state()
            .model
            .as_ref()
            .and_then(|model| model.variant.as_deref()),
        Some("thinking")
    );

    store.remove_context("file:src/index.ts");
    store.set_prompt(vec![text("old", 0, 3)], Some(3));
    store.set_model(None);

    assert!(store.state().context.items.is_empty());
    assert_eq!(store.state().prompt, vec![text("old", 0, 3)]);
    assert!(store.state().model.is_none());
}

#[test]
fn resets_the_prompt_and_cursor() {
    let state = PersistedState {
        prompt: vec![text("old", 0, 3), image()],
        cursor: Some(3),
        model: Some(model()),
        context: ContextItems::default(),
    };
    let mut store = PromptStore::new(state);

    store.reset();

    assert_eq!(store.state().prompt, vec![text("", 0, 0)]);
    assert_eq!(store.state().cursor, Some(0));
}
