//! Port of packages/console/app/test/museSparkPolicy.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/console/app/src/lib/request-country.ts and
//! src/routes/zen/util/trainingConsent.ts: the `muse-spark-*-contributor` models
//! are country-restricted (blocked in `CN`), while the `-free` and similar ids
//! are not; Go training consent is required only for the non-free contributor
//! ids.

use opencode_console::muse_spark_policy::{
    is_model_country_restricted, requires_go_training_consent,
};

const RESTRICTED: [&str; 4] = [
    "muse-spark-1.3-contributor",
    "muse-spark-1.3-contributor-free",
    "muse-spark-1.2-contributor",
    "muse-spark-1.2-contributor-free",
];

#[test]
fn restricts_contributor_models_in_blocked_countries() {
    for model in RESTRICTED {
        assert!(is_model_country_restricted(model, "CN"));
        assert!(!is_model_country_restricted(model, "US"));
    }
}

#[test]
fn does_not_apply_the_country_restriction_to_similar_model_ids() {
    assert!(!is_model_country_restricted(
        "muse-spark-1.3-contributor-preview",
        "CN"
    ));
}

#[test]
fn requires_go_training_consent_for_non_free_contributor_models() {
    assert!(requires_go_training_consent("muse-spark-1.3-contributor"));
    assert!(requires_go_training_consent("muse-spark-1.2-contributor"));
}

#[test]
fn does_not_require_go_training_consent_for_free_or_similar_model_ids() {
    assert!(!requires_go_training_consent(
        "muse-spark-1.3-contributor-free"
    ));
    assert!(!requires_go_training_consent(
        "muse-spark-1.3-contributor-preview"
    ));
}
