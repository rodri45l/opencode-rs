//! Port of packages/app/src/utils/uuid.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq)]
enum RandomUuid {
    Available(&'static str),
    Throws,
    Missing,
}

// Local stub (fast wave): real module lands later.
fn uuid(_secure: bool, _random_uuid: RandomUuid, _random: f64) -> String {
    String::new()
}

#[test]
#[ignore = "porting: utils/uuid not implemented"]
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
#[ignore = "porting: utils/uuid not implemented"]
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
#[ignore = "porting: utils/uuid not implemented"]
fn falls_back_when_random_uuid_throws() {
    assert_eq!(uuid(true, RandomUuid::Throws, 0.5), "8");
}

#[test]
#[ignore = "porting: utils/uuid not implemented"]
fn falls_back_when_random_uuid_is_unavailable() {
    assert_eq!(uuid(true, RandomUuid::Missing, 0.5), "8");
}
