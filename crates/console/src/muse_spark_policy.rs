//! Muse Spark country restrictions and training consent.
//!
//! Port of `packages/console/app/src/lib/request-country.ts` and
//! `src/routes/zen/util/trainingConsent.ts` (upstream 18ef3cc).

const MUSE_SPARK_BLOCKED_COUNTRIES: [&str; 22] = [
    "AF", "BY", "CN", "CU", "EH", "ER", "ET", "HK", "HT", "IQ", "IR", "KH", "KP", "LY", "MM", "MO",
    "NI", "PK", "RU", "SO", "SY", "VE",
];

const RESTRICTED_MODELS: [&str; 4] = [
    "muse-spark-1.3-contributor",
    "muse-spark-1.3-contributor-free",
    "muse-spark-1.2-contributor",
    "muse-spark-1.2-contributor-free",
];

const CONSENT_MODELS: [&str; 2] = ["muse-spark-1.3-contributor", "muse-spark-1.2-contributor"];

/// Whether `model` is country-restricted for `country`.
pub fn is_model_country_restricted(model: &str, country: &str) -> bool {
    RESTRICTED_MODELS.contains(&model)
        && MUSE_SPARK_BLOCKED_COUNTRIES.contains(&country.to_uppercase().as_str())
}

/// Whether `model` requires Go training consent.
pub fn requires_go_training_consent(model: &str) -> bool {
    CONSENT_MODELS.contains(&model)
}
