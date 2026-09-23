//! Config entry naming from scanned relative paths.
//!
//! Ports the observable behaviour of `packages/opencode/src/config/entry-name.ts`:
//! strip a known `agent/`/`agents/` prefix from a caller-relative path, fall
//! back to the basename, then drop the file extension.

use std::path::Path;

/// Derive a config entry key from a relative `path` and known `prefixes`.
pub fn config_entry_name_from_path(path: &str, prefixes: &[&str]) -> String {
    let normalized = path.replace('\\', "/");
    let candidate = prefixes
        .iter()
        .find_map(|prefix| normalized.strip_prefix(prefix))
        .map(str::to_string)
        .unwrap_or_else(|| {
            Path::new(&normalized)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default()
        });
    if let Some(position) = candidate.rfind('.') {
        if position > 0 {
            return candidate[..position].to_string();
        }
    }
    candidate
}
