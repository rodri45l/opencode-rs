//! Port of packages/app/src/utils/uuid.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::uuid::{uuid, RandomUuid};

#[test]
fn uses_random_uuid_in_secure_contexts() {
    assert_eq!(
        uuid(
            true,
            RandomUuid::Available("00000000-0000-0000-0000-000000000000"),
            0.0
        ),
        "00000000-0000-0000-0000-000000000000"
    );
}

#[test]
fn falls_back_in_insecure_contexts() {
    assert_eq!(
        uuid(
            false,
            RandomUuid::Available("00000000-0000-0000-0000-000000000000"),
            0.5
        ),
        "8"
    );
}

#[test]
fn falls_back_when_random_uuid_throws() {
    assert_eq!(uuid(true, RandomUuid::Throws, 0.5), "8");
}

#[test]
fn falls_back_when_random_uuid_is_unavailable() {
    assert_eq!(uuid(true, RandomUuid::Missing, 0.5), "8");
}
