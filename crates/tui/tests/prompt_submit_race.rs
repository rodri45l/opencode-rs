//! Port of packages/tui/test/cli/tui/prompt-submit-race.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the `submit` in-flight guard in
//! packages/tui/src/component/prompt/index.tsx; see docs/TEST-PORT.md.

use opencode_tui::prompt_submit::submit_prompt;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

type Submissions = Arc<Mutex<Vec<(String, String)>>>;

#[tokio::test]
async fn concurrent_submits_must_not_lose_the_users_text() {
    let guard = AtomicBool::new(false);
    let counter = Arc::new(AtomicUsize::new(0));
    let submissions: Submissions = Arc::new(Mutex::new(Vec::new()));

    let (c1, s1) = (counter.clone(), submissions.clone());
    let first = submit_prompt(
        &guard,
        "Hello there.".to_string(),
        move || {
            let counter = c1.clone();
            async move {
                let id = format!("ses_{}", counter.fetch_add(1, Ordering::SeqCst) + 1);
                tokio::time::sleep(Duration::from_millis(5)).await;
                id
            }
        },
        move |session, text| {
            let submissions = s1.clone();
            async move {
                submissions.lock().unwrap().push((session, text));
            }
        },
    );

    let (c2, s2) = (counter.clone(), submissions.clone());
    let second = submit_prompt(
        &guard,
        "Hello there.".to_string(),
        move || {
            let counter = c2.clone();
            async move {
                let id = format!("ses_{}", counter.fetch_add(1, Ordering::SeqCst) + 1);
                tokio::time::sleep(Duration::from_millis(5)).await;
                id
            }
        },
        move |session, text| {
            let submissions = s2.clone();
            async move {
                submissions.lock().unwrap().push((session, text));
            }
        },
    );

    let _ = tokio::join!(first, second);

    let submissions = submissions.lock().unwrap();
    assert!(submissions.iter().all(|(_, text)| text == "Hello there."));
    assert!(!submissions.iter().any(|(_, text)| text.is_empty()));
}

#[tokio::test]
async fn a_sequential_second_submit_after_clear_is_a_no_op_not_a_phantom_session() {
    let guard = AtomicBool::new(false);
    let counter = Arc::new(AtomicUsize::new(0));
    let submissions: Submissions = Arc::new(Mutex::new(Vec::new()));

    let (c1, s1) = (counter.clone(), submissions.clone());
    let first = submit_prompt(
        &guard,
        "Hello there.".to_string(),
        move || {
            let counter = c1.clone();
            async move {
                let id = format!("ses_{}", counter.fetch_add(1, Ordering::SeqCst) + 1);
                tokio::time::sleep(Duration::from_millis(1)).await;
                id
            }
        },
        move |session, text| {
            let submissions = s1.clone();
            async move {
                submissions.lock().unwrap().push((session, text));
            }
        },
    )
    .await;

    let (c2, s2) = (counter.clone(), submissions.clone());
    let second = submit_prompt(
        &guard,
        String::new(),
        move || {
            let counter = c2.clone();
            async move { format!("ses_{}", counter.fetch_add(1, Ordering::SeqCst) + 1) }
        },
        move |session, text| {
            let submissions = s2.clone();
            async move {
                submissions.lock().unwrap().push((session, text));
            }
        },
    )
    .await;

    assert!(first);
    assert!(!second);
    let submissions = submissions.lock().unwrap();
    assert_eq!(submissions.len(), 1);
    assert_eq!(submissions[0].1, "Hello there.");
}
