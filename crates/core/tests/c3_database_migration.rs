//! Port of packages/core/test/database-migration.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: Windows drive/UNC storage paths normalize to forward
//! slashes while POSIX paths (including a pathological backslash) survive
//! byte-for-byte; a relative `path` column normalizes only when its `directory`
//! is Windows-shaped; assistant usage backfills from message data; unnamed
//! legacy Drizzle journal rows map to migrations by their real timestamp and an
//! unknown timestamp is rejected; a non-empty database without a `session`
//! table is refused; Context Epoch rows backfill to the `build` agent; an
//! existing migration table suppresses the Drizzle import; and the temporary
//! metadata migration id canonicalizes to the durable one.
//!
//! Re-derived (dropped): the live `SqliteClient`/Drizzle/`Effect` wiring, the
//! generated migration list, the schema/index creation assertions, the
//! concurrent embedded-initialization test, and the Windows-only codec test.

#![allow(dead_code)]

use serde_json::{json, Value};

const NOTE: &str = "porting: database migration not implemented";

#[derive(Debug, PartialEq, Eq)]
enum MigrationError {
    NotImplemented,
    DatabaseNotEmpty,
    UnknownTimestamp,
}

#[derive(Debug, PartialEq)]
struct SessionUsage {
    cost: f64,
    input: i64,
    output: i64,
    reasoning: i64,
    cache_read: i64,
    cache_write: i64,
}

fn normalize_storage_path(_path: &str) -> Result<String, MigrationError> {
    Err(MigrationError::NotImplemented)
}

fn normalize_storage_row(
    _directory: &str,
    _path: Option<&str>,
) -> Result<(String, Option<String>), MigrationError> {
    Err(MigrationError::NotImplemented)
}

fn session_usage_backfill(_message: &Value) -> Result<SessionUsage, MigrationError> {
    Err(MigrationError::NotImplemented)
}

fn timestamp_prefix(_millis: i64) -> Result<String, MigrationError> {
    Err(MigrationError::NotImplemented)
}

fn migration_id_for_timestamp(_millis: i64) -> Result<&'static str, MigrationError> {
    Err(MigrationError::NotImplemented)
}

fn can_apply_migrations(
    _table_count: usize,
    _has_session_table: bool,
) -> Result<(), MigrationError> {
    Err(MigrationError::NotImplemented)
}

fn context_epoch_default_agent() -> Result<&'static str, MigrationError> {
    Err(MigrationError::NotImplemented)
}

fn should_import_drizzle(_migration_rows: usize) -> Result<bool, MigrationError> {
    Err(MigrationError::NotImplemented)
}

fn canonical_migration_id(_id: &str) -> Result<String, MigrationError> {
    Err(MigrationError::NotImplemented)
}

#[test]
#[ignore = "porting: database migration not implemented"]
fn normalizes_windows_storage_paths_and_leaves_posix_untouched() {
    assert_eq!(
        normalize_storage_path(r"C:\Repo\Thing").expect(NOTE),
        "C:/Repo/Thing"
    );
    assert_eq!(
        normalize_storage_path(r"C:\Repo\Thing\sandbox").expect(NOTE),
        "C:/Repo/Thing/sandbox"
    );
    assert_eq!(
        normalize_storage_path(r"\\server\share").expect(NOTE),
        "//server/share"
    );
    assert_eq!(
        normalize_storage_path(r"\\server\share\sandbox").expect(NOTE),
        "//server/share/sandbox"
    );
    assert_eq!(normalize_storage_path("/").expect(NOTE), "/");
    assert_eq!(
        normalize_storage_path(r"/home/me/we\ird").expect(NOTE),
        r"/home/me/we\ird"
    );
    assert_eq!(
        normalize_storage_path(r"src\weird").expect(NOTE),
        r"src\weird"
    );
}

#[test]
#[ignore = "porting: database migration not implemented"]
fn normalizes_row_paths_only_when_the_directory_is_windows_shaped() {
    let (directory, path) =
        normalize_storage_row(r"C:\Repo\Thing\packages\api", Some(r"packages\api")).expect(NOTE);
    assert_eq!(directory, "C:/Repo/Thing/packages/api");
    assert_eq!(path.as_deref(), Some("packages/api"));

    let (directory, path) =
        normalize_storage_row(r"/home/me/we\ird", Some(r"src\weird")).expect(NOTE);
    assert_eq!(directory, r"/home/me/we\ird");
    assert_eq!(path.as_deref(), Some(r"src\weird"));
}

#[test]
#[ignore = "porting: database migration not implemented"]
fn backfills_session_usage_from_assistant_message_data() {
    let usage = session_usage_backfill(&json!({
        "role": "assistant",
        "cost": 1.25,
        "tokens": { "input": 2, "output": 3, "reasoning": 4, "cache": { "read": 5, "write": 6 } }
    }))
    .expect(NOTE);

    assert_eq!(
        usage,
        SessionUsage {
            cost: 1.25,
            input: 2,
            output: 3,
            reasoning: 4,
            cache_read: 5,
            cache_write: 6,
        }
    );
}

#[test]
#[ignore = "porting: database migration not implemented"]
fn maps_unnamed_legacy_drizzle_journal_rows_by_their_real_timestamp() {
    assert_eq!(
        timestamp_prefix(1_775_843_113_000).expect(NOTE),
        "20260410174513"
    );
    assert_eq!(
        migration_id_for_timestamp(1_775_843_113_000).expect(NOTE),
        "20260410174513_workspace-name"
    );
    assert_eq!(
        migration_id_for_timestamp(1_234_567_890_000),
        Err(MigrationError::UnknownTimestamp)
    );
}

#[test]
#[ignore = "porting: database migration not implemented"]
fn rejects_a_non_empty_database_without_a_session_table() {
    assert!(can_apply_migrations(0, false).is_ok());
    assert!(can_apply_migrations(1, true).is_ok());
    assert_eq!(
        can_apply_migrations(1, false),
        Err(MigrationError::DatabaseNotEmpty)
    );
}

#[test]
#[ignore = "porting: database migration not implemented"]
fn backfills_context_epoch_rows_to_the_build_agent() {
    assert_eq!(context_epoch_default_agent().expect(NOTE), "build");
}

#[test]
#[ignore = "porting: database migration not implemented"]
fn skips_the_drizzle_import_when_the_migration_table_has_state() {
    assert!(should_import_drizzle(0).expect(NOTE));
    assert!(!should_import_drizzle(1).expect(NOTE));
}

#[test]
#[ignore = "porting: database migration not implemented"]
fn canonicalizes_the_temporary_session_metadata_migration_id() {
    assert_eq!(
        canonical_migration_id("20260530232709_lovely_romulus").expect(NOTE),
        "20260511173437_session-metadata"
    );
    assert_eq!(
        canonical_migration_id("20260127222353_familiar_lady_ursula").expect(NOTE),
        "20260127222353_familiar_lady_ursula"
    );
}
