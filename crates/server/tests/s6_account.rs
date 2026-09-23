//! Port of packages/opencode/test/cli/account.test.ts (upstream 18ef3cc).
//!
//! RED-first: the console-account display helpers in `cli/cmd/account` are not
//! implemented in this crate. The reference assertions are pinned against local
//! typed stubs; the fixer points them at the real API.

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
struct Account {
    email: String,
    url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Org {
    id: String,
    name: String,
}

fn account(email: &str, url: &str) -> Account {
    Account {
        email: email.to_string(),
        url: url.to_string(),
    }
}

fn default_console_url() -> &'static str {
    ""
}

fn format_account_label(_account: &Account, _active: bool) -> String {
    String::new()
}

fn format_org_line(_account: &Account, _org: &Org, _active: bool) -> String {
    String::new()
}

#[test]
#[ignore = "porting: cli account display not implemented"]
fn uses_opencode_ai_console_as_the_default_login_url() {
    assert_eq!(default_console_url(), "https://opencode.ai/console");
}

#[test]
#[ignore = "porting: cli account display not implemented"]
fn includes_the_account_url_in_account_labels() {
    let acct = account("one@example.com", "https://one.example.com");
    assert_eq!(
        format_account_label(&acct, false),
        "one@example.com https://one.example.com"
    );
}

#[test]
#[ignore = "porting: cli account display not implemented"]
fn includes_the_active_marker_in_account_labels() {
    let acct = account("one@example.com", "https://one.example.com");
    assert_eq!(
        format_account_label(&acct, true),
        "one@example.com https://one.example.com (active)"
    );
}

#[test]
#[ignore = "porting: cli account display not implemented"]
fn includes_the_account_url_in_org_rows() {
    let acct = account("one@example.com", "https://one.example.com");
    let org = Org {
        id: "org-1".to_string(),
        name: "One".to_string(),
    };
    assert_eq!(
        format_org_line(&acct, &org, true),
        "  ● One  one@example.com  https://one.example.com  org-1"
    );
}
