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

use opencode_core::database_migration::{
    can_apply_migrations, canonical_migration_id, context_epoch_default_agent,
    migration_id_for_timestamp, normalize_storage_path, normalize_storage_row,
    session_usage_backfill, should_import_drizzle, timestamp_prefix, MigrationError, SessionUsage,
};
use serde_json::json;

#[test]
fn normalizes_windows_storage_paths_and_leaves_posix_untouched() {
    assert_eq!(
        normalize_storage_path(r"C:\Repo\Thing").expect("path"),
        "C:/Repo/Thing"
    );
    assert_eq!(
        normalize_storage_path(r"C:\Repo\Thing\sandbox").expect("path"),
        "C:/Repo/Thing/sandbox"
    );
    assert_eq!(
        normalize_storage_path(r"\\server\share").expect("path"),
        "//server/share"
    );
    assert_eq!(
        normalize_storage_path(r"\\server\share\sandbox").expect("path"),
        "//server/share/sandbox"
    );
    assert_eq!(normalize_storage_path("/").expect("path"), "/");
    assert_eq!(
        normalize_storage_path(r"/home/me/we\ird").expect("path"),
        r"/home/me/we\ird"
    );
    assert_eq!(
        normalize_storage_path(r"src\weird").expect("path"),
        r"src\weird"
    );
}

#[test]
fn normalizes_row_paths_only_when_the_directory_is_windows_shaped() {
    let (directory, path) =
        normalize_storage_row(r"C:\Repo\Thing\packages\api", Some(r"packages\api")).expect("row");
    assert_eq!(directory, "C:/Repo/Thing/packages/api");
    assert_eq!(path.as_deref(), Some("packages/api"));

    let (directory, path) =
        normalize_storage_row(r"/home/me/we\ird", Some(r"src\weird")).expect("row");
    assert_eq!(directory, r"/home/me/we\ird");
    assert_eq!(path.as_deref(), Some(r"src\weird"));
}

#[test]
fn backfills_session_usage_from_assistant_message_data() {
    let usage = session_usage_backfill(&json!({
        "role": "assistant",
        "cost": 1.25,
        "tokens": { "input": 2, "output": 3, "reasoning": 4, "cache": { "read": 5, "write": 6 } }
    }))
    .expect("usage");

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
fn maps_unnamed_legacy_drizzle_journal_rows_by_their_real_timestamp() {
    assert_eq!(
        timestamp_prefix(1_775_843_113_000).expect("prefix"),
        "20260410174513"
    );
    assert_eq!(
        migration_id_for_timestamp(1_775_843_113_000).expect("id"),
        "20260410174513_workspace-name"
    );
    assert_eq!(
        migration_id_for_timestamp(1_234_567_890_000),
        Err(MigrationError::UnknownTimestamp)
    );
}

#[test]
fn rejects_a_non_empty_database_without_a_session_table() {
    assert!(can_apply_migrations(0, false).is_ok());
    assert!(can_apply_migrations(1, true).is_ok());
    assert_eq!(
        can_apply_migrations(1, false),
        Err(MigrationError::DatabaseNotEmpty)
    );
}

#[test]
fn backfills_context_epoch_rows_to_the_build_agent() {
    assert_eq!(context_epoch_default_agent().expect("agent"), "build");
}

#[test]
fn skips_the_drizzle_import_when_the_migration_table_has_state() {
    assert!(should_import_drizzle(0).expect("import"));
    assert!(!should_import_drizzle(1).expect("import"));
}

#[test]
fn canonicalizes_the_temporary_session_metadata_migration_id() {
    assert_eq!(
        canonical_migration_id("20260530232709_lovely_romulus").expect("canonical"),
        "20260511173437_session-metadata"
    );
    assert_eq!(
        canonical_migration_id("20260127222353_familiar_lady_ursula").expect("canonical"),
        "20260127222353_familiar_lady_ursula"
    );
}
