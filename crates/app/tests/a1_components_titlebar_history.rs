//! Port of packages/app/src/components/titlebar-history.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::titlebar_history::{apply_path, back_path, forward_path, TitlebarHistory};

fn history() -> TitlebarHistory {
    TitlebarHistory {
        stack: Vec::new(),
        index: 0,
        action: None,
    }
}

#[test]
fn append_and_trim_keeps_max_bounded() {
    let mut state = history();
    state = apply_path(state, "/", 3);
    state = apply_path(state, "/a", 3);
    state = apply_path(state, "/b", 3);
    state = apply_path(state, "/c", 3);

    assert_eq!(
        state.stack,
        vec!["/a".to_string(), "/b".to_string(), "/c".to_string()]
    );
    assert_eq!(state.stack.len(), 3);
    assert_eq!(state.index, 2);
}

#[test]
fn back_and_forward_indexes_stay_correct_after_trimming() {
    let mut state = history();
    state = apply_path(state, "/", 3);
    state = apply_path(state, "/a", 3);
    state = apply_path(state, "/b", 3);
    state = apply_path(state, "/c", 3);

    assert_eq!(
        state.stack,
        vec!["/a".to_string(), "/b".to_string(), "/c".to_string()]
    );
    assert_eq!(state.index, 2);

    let back = back_path(state).expect("back");
    assert_eq!(back.to, "/b");
    assert_eq!(back.state.index, 1);

    let after_back = apply_path(back.state, &back.to, 3);
    assert_eq!(
        after_back.stack,
        vec!["/a".to_string(), "/b".to_string(), "/c".to_string()]
    );
    assert_eq!(after_back.index, 1);

    let forward = forward_path(after_back).expect("forward");
    assert_eq!(forward.to, "/c");
    assert_eq!(forward.state.index, 2);

    let after_forward = apply_path(forward.state, &forward.to, 3);
    assert_eq!(
        after_forward.stack,
        vec!["/a".to_string(), "/b".to_string(), "/c".to_string()]
    );
    assert_eq!(after_forward.index, 2);
}

#[test]
fn action_driven_navigation_does_not_push_duplicate_history_entries() {
    let state = TitlebarHistory {
        stack: vec!["/".into(), "/a".into(), "/b".into()],
        index: 2,
        action: None,
    };
    let back = back_path(state).expect("back");
    assert_eq!(back.to, "/a");

    let next = apply_path(back.state, &back.to, 10);
    assert_eq!(
        next.stack,
        vec!["/".to_string(), "/a".to_string(), "/b".to_string()]
    );
    assert_eq!(next.index, 1);
    assert_eq!(next.action, None);
}
