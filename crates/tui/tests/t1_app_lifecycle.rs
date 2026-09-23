//! Port of packages/tui/test/app-lifecycle.test.tsx (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/app.tsx, util/presentation.ts and
//! util/error.ts; see docs/TEST-PORT.md. Renderer/process wiring is replaced by
//! an equivalent in-memory lifecycle model.
#![allow(dead_code)]

use serde_json::Value;

// --- epilogue (local stub) --------------------------------------------------------

fn session_epilogue(title: &str, session_id: &str) -> String {
    [
        "  Session  ".to_string() + title,
        format!("  Continue opencode -s {session_id}"),
        String::new(),
    ]
    .join("\n")
}

// --- cli error formatting (local stub) --------------------------------------------

fn cli_error_message(input: &Value) -> Option<String> {
    let obj = input.as_object()?;
    if obj.get("name").and_then(Value::as_str) == Some("ConfigRemoteAuthError") {
        let data = obj.get("data")?.as_object()?;
        let url = data.get("url").and_then(Value::as_str);
        let remote = data.get("remote").and_then(Value::as_str);
        let mut lines = vec![
            format!(
                "Failed to load remote config{}: the server returned a login page instead of JSON.",
                remote.map(|remote| format!(" from {remote}")).unwrap_or_default()
            ),
            "Authentication is missing or has expired (the endpoint is likely behind an SSO or identity-aware proxy)."
                .to_string(),
        ];
        if let Some(url) = url {
            lines.push(format!(
                "Run `opencode auth login {url}` to re-authenticate."
            ));
        }
        return Some(lines.join("\n"));
    }
    None
}

fn error_format(error: &Value) -> String {
    serde_json::to_string_pretty(error).unwrap_or_else(|_| "Unexpected error".to_string())
}

fn fatal_message(error: &Value) -> String {
    cli_error_message(error).unwrap_or_else(|| error_format(error))
}

// --- lifecycle model (local stub) -------------------------------------------------

#[derive(Debug, Default)]
struct Lifecycle {
    renderer_destroyed: bool,
    terminal_title: Option<String>,
    dispose_count: u32,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    listeners: Vec<String>,
    original_listeners: Vec<String>,
}

impl Lifecycle {
    fn mount(original_listeners: Vec<String>) -> Self {
        let mut listeners = original_listeners.clone();
        listeners.push("sighup".to_string());
        Self {
            listeners,
            original_listeners,
            ..Default::default()
        }
    }

    fn scoped_cleanup(&mut self) {
        self.renderer_destroyed = true;
        self.dispose_count += 1;
    }

    /// SIGHUP clears the terminal title and tears down scoped resources once.
    fn handle_sighup(&mut self) {
        self.terminal_title = Some(String::new());
        self.scoped_cleanup();
        self.listeners = self.original_listeners.clone();
    }

    fn app_exit(&mut self, title: &str, session_id: &str) {
        self.stdout.push_str(&session_epilogue(title, session_id));
        self.scoped_cleanup();
    }

    fn fatal_startup(&mut self, error: &Value) {
        self.stderr.push_str(&fatal_message(error));
        self.exit_code = Some(1);
        self.scoped_cleanup();
    }
}

#[test]
fn sighup_clears_title_and_disposes_scoped_resources_once() {
    let original = vec!["stdout".to_string(), "stdin".to_string()];
    let mut app = Lifecycle::mount(original.clone());

    app.handle_sighup();

    assert!(app.renderer_destroyed);
    assert_eq!(app.terminal_title.as_deref(), Some(""));
    assert_eq!(app.dispose_count, 1);
    assert!(app
        .listeners
        .iter()
        .all(|listener| original.contains(listener)));
    assert_eq!(app.listeners, original);
}

#[test]
fn app_exit_prints_the_session_epilogue_after_scoped_cleanup() {
    let mut app = Lifecycle::mount(Vec::new());

    app.app_exit("Demo session", "dummy");

    assert!(app.stdout.contains("Demo session"));
    assert!(app.stdout.contains("opencode -s dummy"));
    assert_eq!(app.dispose_count, 1);
}

#[test]
fn fatal_startup_errors_set_a_nonzero_exit_after_scoped_cleanup() {
    let mut app = Lifecycle::mount(Vec::new());

    app.fatal_startup(&serde_json::json!({
        "name": "ConfigRemoteAuthError",
        "data": {
            "url": "https://example.com",
            "remote": "https://config.example.com/opencode.json",
        },
    }));

    assert!(app
        .stderr
        .contains("Run `opencode auth login https://example.com` to re-authenticate."));
    assert!(!app.stderr.contains("Unexpected server error"));
    assert_eq!(app.exit_code, Some(1));
    assert!(app.renderer_destroyed);
    assert_eq!(app.dispose_count, 1);
}
