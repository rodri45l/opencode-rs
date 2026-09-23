//! Port of packages/session-ui/src/components/markdown-inline-code-kind.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/markdown-inline-code-kind.ts; see docs/TEST-PORT.md.

use opencode_session_ui::markdown_inline_code_kind::{inline_code_kind, InlineCodeKind};

fn kind(code: &str) -> Option<&'static str> {
    inline_code_kind(code).map(InlineCodeKind::as_str)
}

#[test]
fn leaves_code_expressions_as_normal_inline_code() {
    assert_eq!(
        kind(
            r#"case "question.asked": ... input.setStore("question", question.sessionID, [question]) / splice/insert"#
        ),
        None
    );
    assert_eq!(kind("<SessionQuestionDock request={request} ... />"), None);
    assert_eq!(kind("from sync.data.question + sync.data.session."), None);
    assert_eq!(kind("@opencode-ai/app <StatusPopover />)"), None);
    assert_eq!(kind("sync.data.session"), None);
    assert_eq!(kind("window.api"), None);
    assert_eq!(kind("1.2"), None);
}

#[test]
fn detects_file_and_directory_paths() {
    for path in [
        "app.tsx",
        "vite.config.mjs",
        "eslint.config.cjs",
        "app.d.ts",
        "component.svelte",
        "schema.graphql",
        "Dockerfile",
        "Dockerfile.dev",
        ".gitignore",
        "Cargo.lock",
        "go.sum",
        "bun.lockb",
        "terraform.tfvars",
        "pnpm-lock.yaml",
        "packages/desktop-electron",
        "~/.config/opencode",
        "@opencode-ai/app",
        "session/status",
    ] {
        assert_eq!(kind(path), Some("path"), "{path}");
    }
}

#[test]
fn detects_urls() {
    assert_eq!(kind("https://opencode.ai/docs"), Some("url"));
    assert_eq!(kind("http://localhost:4444"), Some("url"));
    assert_eq!(kind("file:///tmp/opencode"), None);
    assert_eq!(kind("ftp://opencode.ai/docs"), None);
}
