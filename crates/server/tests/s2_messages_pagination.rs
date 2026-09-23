#![allow(dead_code)]

//! Port of packages/opencode/test/session/messages-pagination.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: pure cursor codec, backward pagination ordering, and
//! `MessageV2.filterCompacted` compaction-boundary selection. The same file's
//! database-coupled `page`/`stream`/`get`/`parts` hydration cases need the live
//! session store and are dropped here (noted in PORT-STATUS.s2.json). Stubs are
//! local per the fast-wave protocol and return a typed error until the module
//! lands.

#[derive(Debug, Clone, PartialEq, Eq)]
enum S2Error {
    NotImplemented(&'static str),
}

impl std::fmt::Display for S2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S2Error::NotImplemented(what) => write!(f, "not implemented: {what}"),
        }
    }
}

impl std::error::Error for S2Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Part {
    Text,
    Compaction {
        auto: bool,
        tail_start_id: Option<String>,
    },
}

#[derive(Debug, Clone)]
struct Info {
    id: String,
    role: Role,
    created: f64,
    finish: Option<String>,
    summary: bool,
    has_error: bool,
    parent_id: Option<String>,
}

#[derive(Debug, Clone)]
struct WithParts {
    info: Info,
    parts: Vec<Part>,
}

#[derive(Debug, Clone, PartialEq)]
struct PageItem {
    id: String,
    created: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct Page {
    items: Vec<PageItem>,
    more: bool,
    cursor: Option<String>,
}

fn cursor_encode(_id: &str, _time: f64) -> Result<String, S2Error> {
    Err(S2Error::NotImplemented("MessageV2.cursor.encode"))
}

fn cursor_decode(_encoded: &str) -> Result<(String, f64), S2Error> {
    Err(S2Error::NotImplemented("MessageV2.cursor.decode"))
}

fn page(_items: &[PageItem], _limit: usize, _before: Option<&str>) -> Result<Page, S2Error> {
    Err(S2Error::NotImplemented("MessageV2.page"))
}

fn filter_compacted(_msgs: &[WithParts]) -> Result<Vec<WithParts>, S2Error> {
    Err(S2Error::NotImplemented("MessageV2.filterCompacted"))
}

fn page_item(i: usize) -> PageItem {
    PageItem {
        id: format!("msg_{i}"),
        created: i as f64,
    }
}

fn fill_page(count: usize) -> Vec<PageItem> {
    (0..count).map(page_item).collect()
}

fn page_ids(page: &Page) -> Vec<String> {
    page.items.iter().map(|i| i.id.clone()).collect()
}

fn user(id: &str, created: f64) -> Info {
    Info {
        id: id.to_string(),
        role: Role::User,
        created,
        finish: None,
        summary: false,
        has_error: false,
        parent_id: None,
    }
}

fn assistant(id: &str, parent: &str, created: f64) -> Info {
    Info {
        id: id.to_string(),
        role: Role::Assistant,
        created,
        finish: Some("end_turn".to_string()),
        summary: false,
        has_error: false,
        parent_id: Some(parent.to_string()),
    }
}

fn summary_assistant(id: &str, parent: &str, created: f64) -> Info {
    Info {
        summary: true,
        ..assistant(id, parent, created)
    }
}

fn msg(info: Info, parts: Vec<Part>) -> WithParts {
    WithParts { info, parts }
}

fn text() -> Part {
    Part::Text
}

fn compaction(tail: Option<&str>) -> Part {
    Part::Compaction {
        auto: true,
        tail_start_id: tail.map(str::to_string),
    }
}

fn filtered_ids(result: &[WithParts]) -> Vec<String> {
    result.iter().map(|m| m.info.id.clone()).collect()
}

#[test]
#[ignore = "porting: MessageV2.cursor not implemented"]
fn cursor_encode_decode_roundtrip() {
    let encoded = cursor_encode("msg_123", 1_234_567_890.0).expect("encode");
    let (id, time) = cursor_decode(&encoded).expect("decode");
    assert_eq!(id, "msg_123");
    assert_eq!(time, 1_234_567_890.0);
}

#[test]
#[ignore = "porting: MessageV2.cursor not implemented"]
fn cursor_encode_decode_with_fractional_time() {
    let encoded = cursor_encode("msg_123", 1_234_567_890.5).expect("encode");
    let (_, time) = cursor_decode(&encoded).expect("decode");
    assert_eq!(time, 1_234_567_890.5);
}

#[test]
#[ignore = "porting: MessageV2.cursor not implemented"]
fn encoded_cursor_is_base64url() {
    let encoded = cursor_encode("msg_123", 0.0).expect("encode");
    assert!(!encoded.is_empty());
    assert!(encoded
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
}

#[test]
#[ignore = "porting: MessageV2.page not implemented"]
fn page_returns_page_result() {
    let items = fill_page(2);
    let result = page(&items, 10, None).expect("page");
    assert_eq!(result.items.len(), 2);
}

#[test]
#[ignore = "porting: MessageV2.page not implemented"]
fn pages_backward_with_opaque_cursors() {
    let items = fill_page(6);
    let ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();

    let a = page(&items, 2, None).expect("page a");
    assert_eq!(page_ids(&a), ids[4..6].to_vec());
    assert!(a.items.iter().all(|i| i.id.starts_with("msg_")));
    assert!(a.more);
    let a_cursor = a.cursor.expect("cursor a");

    let b = page(&items, 2, Some(&a_cursor)).expect("page b");
    assert_eq!(page_ids(&b), ids[2..4].to_vec());
    assert!(b.more);
    let b_cursor = b.cursor.expect("cursor b");

    let c = page(&items, 2, Some(&b_cursor)).expect("page c");
    assert_eq!(page_ids(&c), ids[0..2].to_vec());
    assert!(!c.more);
    assert!(c.cursor.is_none());
}

#[test]
#[ignore = "porting: MessageV2.page not implemented"]
fn page_returns_items_in_chronological_order_within_a_page() {
    let items = fill_page(4);
    let ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
    let result = page(&items, 4, None).expect("page");
    assert_eq!(page_ids(&result), ids);
}

#[test]
#[ignore = "porting: MessageV2.page not implemented"]
fn page_returns_empty_for_no_messages() {
    let result = page(&[], 10, None).expect("page");
    assert!(result.items.is_empty());
    assert!(!result.more);
    assert!(result.cursor.is_none());
}

#[test]
#[ignore = "porting: MessageV2.page not implemented"]
fn page_handles_exact_limit_boundary() {
    let items = fill_page(3);
    let ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
    let result = page(&items, 3, None).expect("page");
    assert_eq!(page_ids(&result), ids);
    assert!(!result.more);
    assert!(result.cursor.is_none());
}

#[test]
#[ignore = "porting: MessageV2.page not implemented"]
fn page_limit_of_one_returns_single_newest_message() {
    let items = fill_page(5);
    let result = page(&items, 1, None).expect("page");
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].id, "msg_4");
    assert!(result.more);
}

#[test]
#[ignore = "porting: MessageV2.page not implemented"]
fn page_accepts_cursors_from_fractional_timestamps() {
    let items: Vec<PageItem> = (0..4)
        .map(|i| PageItem {
            id: format!("msg_{i}"),
            created: 1000.5 + i as f64,
        })
        .collect();
    let ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();

    let a = page(&items, 2, None).expect("page a");
    let b = page(&items, 2, a.cursor.as_deref()).expect("page b");

    assert_eq!(page_ids(&a), ids[2..4].to_vec());
    assert_eq!(page_ids(&b), ids[0..2].to_vec());
}

#[test]
#[ignore = "porting: MessageV2.page not implemented"]
fn page_orders_equal_timestamps_by_id() {
    let items: Vec<PageItem> = (0..4)
        .map(|i| PageItem {
            id: format!("msg_{i}"),
            created: 1000.0,
        })
        .collect();
    let ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();

    let a = page(&items, 2, None).expect("page a");
    assert_eq!(page_ids(&a), ids[2..4].to_vec());
    assert!(a.more);

    let b = page(&items, 2, a.cursor.as_deref()).expect("page b");
    assert_eq!(page_ids(&b), ids[0..2].to_vec());
    assert!(!b.more);
}

#[test]
#[ignore = "porting: MessageV2.filterCompacted not implemented"]
fn filter_compacted_returns_all_messages_when_no_compaction() {
    let msgs: Vec<WithParts> = (0..5)
        .rev()
        .map(|i| msg(user(&format!("msg_{i}"), i as f64), vec![text()]))
        .collect();

    let result = filter_compacted(&msgs).expect("filter");
    let expected: Vec<String> = (0..5).map(|i| format!("msg_{i}")).collect();
    assert_eq!(filtered_ids(&result), expected);
}

#[test]
#[ignore = "porting: MessageV2.filterCompacted not implemented"]
fn filter_compacted_stops_at_boundary_and_returns_chronological_order() {
    let msgs = vec![
        msg(assistant("a2", "u2", 4.0), vec![text()]),
        msg(user("u2", 3.0), vec![text()]),
        msg(summary_assistant("a1", "u1", 2.0), vec![text()]),
        msg(user("u1", 1.0), vec![compaction(None)]),
    ];

    let result = filter_compacted(&msgs).expect("filter");
    assert_eq!(result[0].info.id, "u1");
    assert_eq!(result.len(), 4);
}

#[test]
#[ignore = "porting: MessageV2.filterCompacted not implemented"]
fn filter_compacted_handles_empty_iterable() {
    let result = filter_compacted(&[]).expect("filter");
    assert!(result.is_empty());
}

#[test]
#[ignore = "porting: MessageV2.filterCompacted not implemented"]
fn filter_compacted_does_not_break_on_compaction_without_matching_summary() {
    let msgs = vec![
        msg(user("u2", 2.0), vec![text()]),
        msg(user("u1", 1.0), vec![compaction(None)]),
    ];

    let result = filter_compacted(&msgs).expect("filter");
    assert_eq!(result.len(), 2);
}

#[test]
#[ignore = "porting: MessageV2.filterCompacted not implemented"]
fn filter_compacted_skips_assistant_with_error_even_if_summary() {
    let errored = Info {
        has_error: true,
        ..summary_assistant("a1", "u1", 2.0)
    };
    let msgs = vec![
        msg(user("u2", 3.0), vec![text()]),
        msg(errored, vec![text()]),
        msg(user("u1", 1.0), vec![compaction(None)]),
    ];

    let result = filter_compacted(&msgs).expect("filter");
    assert_eq!(result.len(), 3);
}

#[test]
#[ignore = "porting: MessageV2.filterCompacted not implemented"]
fn filter_compacted_skips_assistant_without_finish_even_if_summary() {
    let no_finish = Info {
        finish: None,
        ..summary_assistant("a1", "u1", 2.0)
    };
    let msgs = vec![
        msg(user("u2", 3.0), vec![text()]),
        msg(no_finish, vec![text()]),
        msg(user("u1", 1.0), vec![compaction(None)]),
    ];

    let result = filter_compacted(&msgs).expect("filter");
    assert_eq!(result.len(), 3);
}

#[test]
#[ignore = "porting: MessageV2.filterCompacted not implemented"]
fn filter_compacted_retains_original_tail_when_compaction_stores_tail_start_id() {
    let msgs = vec![
        msg(assistant("a3", "u3", 8.0), vec![text()]),
        msg(user("u3", 7.0), vec![text()]),
        msg(summary_assistant("s1", "c1", 6.0), vec![text()]),
        msg(user("c1", 5.0), vec![compaction(Some("u2"))]),
        msg(assistant("a2", "u2", 4.0), vec![text()]),
        msg(user("u2", 3.0), vec![text()]),
        msg(assistant("a1", "u1", 2.0), vec![text()]),
        msg(user("u1", 1.0), vec![text()]),
    ];

    let result = filter_compacted(&msgs).expect("filter");
    let expected = ["c1", "s1", "u2", "a2", "u3", "a3"];
    assert_eq!(filtered_ids(&result), expected);
}

#[test]
#[ignore = "porting: MessageV2.filterCompacted not implemented"]
fn filter_compacted_retains_an_assistant_tail_when_compaction_starts_inside_a_turn() {
    let msgs = vec![
        msg(assistant("a4", "u3", 9.0), vec![text()]),
        msg(user("u3", 8.0), vec![text()]),
        msg(summary_assistant("s1", "c1", 7.0), vec![text()]),
        msg(user("c1", 6.0), vec![compaction(Some("a3"))]),
        msg(assistant("a3", "u2", 5.0), vec![text()]),
        msg(assistant("a2", "u2", 4.0), vec![text()]),
        msg(user("u2", 3.0), vec![text()]),
        msg(assistant("a1", "u1", 2.0), vec![text()]),
        msg(user("u1", 1.0), vec![text()]),
    ];

    let result = filter_compacted(&msgs).expect("filter");
    let expected = ["c1", "s1", "a3", "u3", "a4"];
    assert_eq!(filtered_ids(&result), expected);
}

#[test]
#[ignore = "porting: MessageV2.filterCompacted not implemented"]
fn filter_compacted_prefers_latest_boundary_when_repeated_compactions_exist() {
    let msgs = vec![
        msg(assistant("a4", "u4", 12.0), vec![text()]),
        msg(user("u4", 11.0), vec![text()]),
        msg(summary_assistant("s2", "c2", 10.0), vec![text()]),
        msg(user("c2", 9.0), vec![compaction(Some("u3"))]),
        msg(assistant("a3", "u3", 8.0), vec![text()]),
        msg(user("u3", 7.0), vec![text()]),
        msg(summary_assistant("s1", "c1", 6.0), vec![text()]),
        msg(user("c1", 5.0), vec![compaction(Some("u2"))]),
        msg(assistant("a2", "u2", 4.0), vec![text()]),
        msg(user("u2", 3.0), vec![text()]),
        msg(assistant("a1", "u1", 2.0), vec![text()]),
        msg(user("u1", 1.0), vec![text()]),
    ];

    let result = filter_compacted(&msgs).expect("filter");
    let expected = ["c2", "s2", "u3", "a3", "u4", "a4"];
    assert_eq!(filtered_ids(&result), expected);
}

#[test]
#[ignore = "porting: MessageV2.filterCompacted not implemented"]
fn filter_compacted_works_with_array_input() {
    let msgs = vec![msg(user("msg_1", 1.0), vec![text()])];
    let result = filter_compacted(&msgs).expect("filter");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].info.id, "msg_1");
}
