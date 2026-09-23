//! Runtime path helpers.
//!
//! Port of packages/tui/src/runtime.tsx `abbreviateHome` (upstream 18ef3cc).

use std::path::Path;

/// Abbreviate a path under `home` to `~`, leaving paths outside it unchanged.
pub fn abbreviate_home(input: &str, home: &str) -> String {
    if home.is_empty() {
        return input.to_string();
    }
    if input == home {
        return "~".to_string();
    }
    match Path::new(input).strip_prefix(home) {
        Ok(relative) => format!("~/{}", relative.to_string_lossy()),
        Err(_) => input.to_string(),
    }
}
