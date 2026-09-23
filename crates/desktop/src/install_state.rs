//! Fresh-install detection.
//!
//! Port of `packages/desktop/src/main/install-state.ts` (upstream 18ef3cc).

/// A directory entry used for install-state detection.
#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub is_directory: bool,
}

/// A file entry.
pub fn file(name: &str) -> Entry {
    Entry {
        name: name.to_string(),
        is_directory: false,
    }
}

/// A directory entry.
pub fn directory(name: &str) -> Entry {
    Entry {
        name: name.to_string(),
        is_directory: true,
    }
}

/// Whether any entry indicates state written by an earlier OpenCode launch.
pub fn has_existing_app_state(entries: &[Entry]) -> bool {
    entries.iter().any(|entry| {
        if entry.name == "opencode.settings" {
            return true;
        }
        if entry.name.ends_with(".dat") {
            return true;
        }
        if is_window_state(&entry.name) {
            return true;
        }
        entry.is_directory && entry.name == "opencode"
    })
}

fn is_window_state(name: &str) -> bool {
    let prefix = "window-state-";
    let suffix = ".json";
    name.starts_with(prefix) && name.ends_with(suffix) && name.len() > prefix.len() + suffix.len()
}
