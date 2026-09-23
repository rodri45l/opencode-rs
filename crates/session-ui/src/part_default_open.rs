//! Default open/closed state for a tool part.
//!
//! Port of packages/session-ui/src/components/part-default-open.ts behaviour
//! (upstream 18ef3cc).

use serde_json::Value;

/// Whether a tool part should be expanded by default.
///
/// `shell_default` is used for non-edit tools; `edit_enabled` gates the
/// edit/apply_patch heuristics that collapse deletion-only changes.
pub fn part_default_open(
    tool: &str,
    metadata: &Value,
    shell_default: bool,
    edit_enabled: bool,
) -> bool {
    match tool {
        "edit" => {
            if !edit_enabled {
                return false;
            }
            let filediff = metadata.get("filediff");
            let additions = filediff
                .and_then(|value| value.get("additions"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            additions > 0
        }
        "apply_patch" => {
            if !edit_enabled {
                return false;
            }
            match metadata.get("files").and_then(Value::as_array) {
                Some(files) => files
                    .iter()
                    .any(|file| file.get("type").and_then(Value::as_str) != Some("delete")),
                None => true,
            }
        }
        _ => shell_default,
    }
}
