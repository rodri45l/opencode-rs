//! Port of packages/session-ui/src/components/markdown-inline-code-kind.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/session-ui/src/components/markdown-inline-code-kind.ts:
//! inline code that is a file/directory path or an http(s) URL is highlighted, code
//! expressions and other schemes are left as normal inline code.
//! Red-first: the classifier is not implemented.

#[allow(dead_code)]
mod inline_code_kind {
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

    pub const NOTE: &str = "porting: session-ui inline code kind not implemented";

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum InlineCodeKind {
        Path,
        Url,
    }

    pub fn inline_code_kind(_input: &str) -> PortResult<Option<InlineCodeKind>> {
        Err(NotImplemented(NOTE))
    }
}

use inline_code_kind::{inline_code_kind, InlineCodeKind, NOTE};

#[test]
#[ignore = "porting: session-ui inline code kind not implemented"]
fn leaves_code_expressions_as_normal_inline_code() {
    for input in [
        "case \"question.asked\": ... input.setStore(\"question\", question.sessionID, [question]) / splice/insert",
        "<SessionQuestionDock request={request} ... />",
        "from sync.data.question + sync.data.session.",
        "@opencode-ai/app <StatusPopover />)",
        "sync.data.session",
        "window.api",
        "1.2",
    ] {
        assert_eq!(inline_code_kind(input).expect(NOTE), None);
    }
}

#[test]
#[ignore = "porting: session-ui inline code kind not implemented"]
fn detects_file_and_directory_paths() {
    for input in [
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
        assert_eq!(
            inline_code_kind(input).expect(NOTE),
            Some(InlineCodeKind::Path),
            "expected path: {input}"
        );
    }
}

#[test]
#[ignore = "porting: session-ui inline code kind not implemented"]
fn detects_urls() {
    assert_eq!(
        inline_code_kind("https://opencode.ai/docs").expect(NOTE),
        Some(InlineCodeKind::Url)
    );
    assert_eq!(
        inline_code_kind("http://localhost:4444").expect(NOTE),
        Some(InlineCodeKind::Url)
    );
    assert_eq!(inline_code_kind("file:///tmp/opencode").expect(NOTE), None);
    assert_eq!(
        inline_code_kind("ftp://opencode.ai/docs").expect(NOTE),
        None
    );
}
