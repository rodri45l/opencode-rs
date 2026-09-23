//! Port of packages/app/test-browser/prompt-scope.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct Tab {
    server: String,
    session_id: String,
}

// Local stub (fast wave): real module lands later.
fn select_prompt_tab<'a>(
    _tabs: &'a [Tab],
    _dir: &str,
    _id: &str,
    _server: &str,
) -> Option<&'a Tab> {
    None
}

#[test]
#[ignore = "porting: context/prompt selectPromptTab not implemented"]
fn selects_the_explicitly_scoped_session_tab_instead_of_the_active_tab() {
    let server = "local";
    let tabs = vec![
        Tab {
            server: server.into(),
            session_id: "A".into(),
        },
        Tab {
            server: server.into(),
            session_id: "B".into(),
        },
    ];

    assert_eq!(
        select_prompt_tab(&tabs, "repo", "B", server),
        Some(&tabs[1])
    );
}
