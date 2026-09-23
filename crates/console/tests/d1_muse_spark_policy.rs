//! Port of packages/console/app/test/museSparkPolicy.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/console/app/src/lib/request-country.ts and
//! src/routes/zen/util/trainingConsent.ts: the `muse-spark-*-contributor` models
//! are country-restricted (blocked in `CN`), while the `-free` and similar ids
//! are not; Go training consent is required only for the non-free contributor
//! ids.

#[allow(dead_code)]
mod muse_spark_policy {
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: console muse-spark policy not implemented";

    pub fn is_model_country_restricted(_model: &str, _country: &str) -> PortResult<bool> {
        Err(NotImplemented(NOTE))
    }

    pub fn requires_go_training_consent(_model: &str) -> PortResult<bool> {
        Err(NotImplemented(NOTE))
    }
}

use muse_spark_policy::{is_model_country_restricted, requires_go_training_consent, NOTE};

const RESTRICTED: [&str; 4] = [
    "muse-spark-1.3-contributor",
    "muse-spark-1.3-contributor-free",
    "muse-spark-1.2-contributor",
    "muse-spark-1.2-contributor-free",
];

#[test]
#[ignore = "porting: console muse-spark policy not implemented"]
fn restricts_contributor_models_in_blocked_countries() {
    for model in RESTRICTED {
        assert!(is_model_country_restricted(model, "CN").expect(NOTE));
        assert!(!is_model_country_restricted(model, "US").expect(NOTE));
    }
}

#[test]
#[ignore = "porting: console muse-spark policy not implemented"]
fn does_not_apply_the_country_restriction_to_similar_model_ids() {
    assert!(!is_model_country_restricted("muse-spark-1.3-contributor-preview", "CN").expect(NOTE));
}

#[test]
#[ignore = "porting: console muse-spark policy not implemented"]
fn requires_go_training_consent_for_non_free_contributor_models() {
    assert!(requires_go_training_consent("muse-spark-1.3-contributor").expect(NOTE));
    assert!(requires_go_training_consent("muse-spark-1.2-contributor").expect(NOTE));
}

#[test]
#[ignore = "porting: console muse-spark policy not implemented"]
fn does_not_require_go_training_consent_for_free_or_similar_model_ids() {
    assert!(!requires_go_training_consent("muse-spark-1.3-contributor-free").expect(NOTE));
    assert!(!requires_go_training_consent("muse-spark-1.3-contributor-preview").expect(NOTE));
}
