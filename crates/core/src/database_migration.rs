//! Database migration helpers.
//!
//! Ports the pure, decidable parts of `packages/core/src/database-migration.ts`:
//! storage-path normalization, assistant-usage backfill, legacy Drizzle journal
//! timestamp mapping, the non-empty-database guard, Context Epoch agent
//! backfill, Drizzle-import gating, and migration-id canonicalization. The live
//! `SqliteClient`/Drizzle wiring is not reproduced here.

use serde_json::Value;

/// Error raised by migration helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    /// The database has tables but no session table.
    DatabaseNotEmpty,
    /// A legacy journal timestamp is not recognized.
    UnknownTimestamp,
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::DatabaseNotEmpty => write!(f, "database is not empty"),
            MigrationError::UnknownTimestamp => write!(f, "unknown migration timestamp"),
        }
    }
}

impl std::error::Error for MigrationError {}

/// Backfilled assistant session usage.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionUsage {
    /// Cost.
    pub cost: f64,
    /// Input tokens.
    pub input: i64,
    /// Output tokens.
    pub output: i64,
    /// Reasoning tokens.
    pub reasoning: i64,
    /// Cache-read tokens.
    pub cache_read: i64,
    /// Cache-write tokens.
    pub cache_write: i64,
}

fn is_windows_shaped(path: &str) -> bool {
    let bytes = path.as_bytes();
    if path.starts_with(r"\\") {
        return true;
    }
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// Normalize a Windows drive/UNC storage path to forward slashes; leave POSIX
/// and relative paths byte-for-byte.
pub fn normalize_storage_path(path: &str) -> Result<String, MigrationError> {
    if is_windows_shaped(path) {
        Ok(path.replace('\\', "/"))
    } else {
        Ok(path.to_string())
    }
}

/// Normalize a storage row's directory and optional path.
pub fn normalize_storage_row(
    directory: &str,
    path: Option<&str>,
) -> Result<(String, Option<String>), MigrationError> {
    let directory = normalize_storage_path(directory)?;
    let path = match path {
        Some(value) if is_windows_shaped(value) || is_windows_shaped(&directory) => {
            Some(value.replace('\\', "/"))
        }
        other => other.map(str::to_string),
    };
    Ok((directory, path))
}

fn number(value: Option<&Value>) -> f64 {
    value.and_then(Value::as_f64).unwrap_or(0.0)
}

/// Backfill session usage from an assistant message's data.
pub fn session_usage_backfill(message: &Value) -> Result<SessionUsage, MigrationError> {
    let tokens = message.get("tokens");
    let cache = tokens.and_then(|value| value.get("cache"));
    Ok(SessionUsage {
        cost: number(message.get("cost")),
        input: number(tokens.and_then(|value| value.get("input"))) as i64,
        output: number(tokens.and_then(|value| value.get("output"))) as i64,
        reasoning: number(tokens.and_then(|value| value.get("reasoning"))) as i64,
        cache_read: number(cache.and_then(|value| value.get("read"))) as i64,
        cache_write: number(cache.and_then(|value| value.get("write"))) as i64,
    })
}

/// Format epoch milliseconds as a `YYYYMMDDHHMMSS` UTC prefix.
pub fn timestamp_prefix(millis: i64) -> Result<String, MigrationError> {
    let seconds = millis.div_euclid(1000);
    let days = seconds.div_euclid(86_400);
    let remainder = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = remainder / 3_600;
    let minute = (remainder % 3_600) / 60;
    let second = remainder % 60;
    Ok(format!(
        "{year:04}{month:02}{day:02}{hour:02}{minute:02}{second:02}"
    ))
}

/// Map a legacy journal timestamp to a named migration id.
pub fn migration_id_for_timestamp(millis: i64) -> Result<&'static str, MigrationError> {
    match millis {
        1_775_843_113_000 => Ok("20260410174513_workspace-name"),
        _ => Err(MigrationError::UnknownTimestamp),
    }
}

/// Refuse a non-empty database that has no `session` table.
pub fn can_apply_migrations(
    table_count: usize,
    has_session_table: bool,
) -> Result<(), MigrationError> {
    if table_count == 0 || has_session_table {
        Ok(())
    } else {
        Err(MigrationError::DatabaseNotEmpty)
    }
}

/// The agent assigned to backfilled Context Epoch rows.
pub fn context_epoch_default_agent() -> Result<&'static str, MigrationError> {
    Ok("build")
}

/// Whether the Drizzle journal should be imported.
pub fn should_import_drizzle(migration_rows: usize) -> Result<bool, MigrationError> {
    Ok(migration_rows == 0)
}

/// Canonicalize a temporary migration id to its durable name.
pub fn canonical_migration_id(id: &str) -> Result<String, MigrationError> {
    if id == "20260530232709_lovely_romulus" {
        Ok("20260511173437_session-metadata".to_string())
    } else {
        Ok(id.to_string())
    }
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if month <= 2 { year + 1 } else { year };
    (year, month, day)
}
