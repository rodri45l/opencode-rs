//! Duration formatting.
//!
//! Port of packages/tui/src/util/format.ts `formatDuration` (upstream 18ef3cc).

/// Format a duration in seconds for display.
pub fn format_duration(secs: i64) -> String {
    if secs <= 0 {
        return String::new();
    }
    if secs < 60 {
        return format!("{secs}s");
    }
    if secs < 3600 {
        let mins = secs / 60;
        let remaining = secs % 60;
        return if remaining > 0 {
            format!("{mins}m {remaining}s")
        } else {
            format!("{mins}m")
        };
    }
    if secs < 86400 {
        let hours = secs / 3600;
        let remaining = (secs % 3600) / 60;
        return if remaining > 0 {
            format!("{hours}h {remaining}m")
        } else {
            format!("{hours}h")
        };
    }
    if secs < 604800 {
        let days = secs / 86400;
        return if days == 1 {
            "~1 day".to_string()
        } else {
            format!("~{days} days")
        };
    }
    let weeks = secs / 604800;
    if weeks == 1 {
        "~1 week".to_string()
    } else {
        format!("~{weeks} weeks")
    }
}
