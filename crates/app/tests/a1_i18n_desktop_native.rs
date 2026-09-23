//! Port of packages/app/src/i18n/desktop-native.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/app/src/i18n/desktop-native.ts.

use std::collections::BTreeMap;

use opencode_app::i18n_desktop_native::{
    create_desktop_native_bundle, detect_desktop_native_locale, format_desktop_native_message,
    native_labels, parse_desktop_native_bundle, Bundle, DESKTOP_NATIVE_LOCALES,
    DESKTOP_NATIVE_MAX_PAYLOAD_BYTES,
};

#[test]
fn uses_native_language_names_independent_of_the_active_locale() {
    let labels = native_labels();
    let actual: Vec<&str> = DESKTOP_NATIVE_LOCALES
        .iter()
        .map(|locale| labels[locale])
        .collect();
    assert_eq!(
        actual,
        vec![
            "English",
            "简体中文",
            "繁體中文",
            "한국어",
            "Deutsch",
            "Español",
            "Français",
            "Dansk",
            "日本語",
            "Polski",
            "Русский",
            "Українська",
            "Bosanski",
            "العربية",
            "Norsk",
            "Português (Brasil)",
            "ไทย",
            "Türkçe",
            "हिन्दी",
            "Nederlands",
            "Bahasa Indonesia",
            "Tiếng Việt",
            "Italiano",
            "اردو",
            "پنجابی",
            "Azərbaycanca",
            "Suomi",
            "Svenska",
            "አማርኛ",
            "Български",
            "বাংলা",
            "Català",
            "Čeština",
            "ދިވެހި",
            "རྫོང་ཁ",
            "Ελληνικά",
            "Eesti",
            "فارسی",
            "Føroyskt",
            "Hrvatski",
            "Magyar",
            "Հայերեն",
            "Íslenska",
            "ქართული",
            "ខ្មែរ",
            "ລາວ",
            "Lietuvių",
            "Latviešu",
            "Македонски",
            "Монгол",
            "Bahasa Melayu",
            "မြန်မာ",
            "नेपाली",
            "Română",
            "සිංහල",
            "Slovenčina",
            "Slovenščina",
            "Shqip",
            "Српски",
            "Тоҷикӣ",
            "Türkmençe",
            "Oʻzbekcha",
        ]
    );
}

#[test]
fn accepts_the_exact_typed_bundle() {
    let bundle = create_desktop_native_bundle("en");
    assert_eq!(parse_desktop_native_bundle(&bundle), Some(bundle));
}

#[test]
fn rejects_unsupported_locales_and_mismatched_key_sets() {
    let bundle = create_desktop_native_bundle("en");
    let mut wrong_locale = bundle.clone();
    wrong_locale.locale = "en-US".into();
    assert_eq!(parse_desktop_native_bundle(&wrong_locale), None);
    let mut extra = bundle.clone();
    extra.messages.insert("extra".into(), "no".into());
    assert_eq!(parse_desktop_native_bundle(&extra), None);
    let mut oversized = bundle.clone();
    oversized
        .messages
        .insert("key".into(), "x".repeat(DESKTOP_NATIVE_MAX_PAYLOAD_BYTES));
    assert_eq!(parse_desktop_native_bundle(&oversized), None);
}

#[test]
fn interpolates_native_templates_without_changing_unknown_placeholders() {
    let mut values = BTreeMap::new();
    values.insert("known".to_string(), "yes".to_string());
    assert_eq!(
        format_desktop_native_message("{{known}} {{unknown}}", &values),
        "yes {{unknown}}"
    );
}

#[test]
fn follows_preference_order_and_skips_invalid_or_unsupported_tags() {
    assert_eq!(
        detect_desktop_native_locale(&["not_a_locale", "fr-FR"]),
        Some("fr".into())
    );
    assert_eq!(
        detect_desktop_native_locale(&["eo", "de-DE"]),
        Some("de".into())
    );
}

#[test]
fn uses_unicode_likely_subtags_for_script_sensitive_bundles() {
    assert_eq!(detect_desktop_native_locale(&["zh-TW"]), Some("zht".into()));
    assert_eq!(detect_desktop_native_locale(&["zh-SG"]), Some("zh".into()));
    assert_eq!(detect_desktop_native_locale(&["pa-PK"]), Some("pa".into()));
    assert_eq!(
        detect_desktop_native_locale(&["pa-IN", "fr"]),
        Some("fr".into())
    );
    assert_eq!(
        detect_desktop_native_locale(&["az-Cyrl", "de"]),
        Some("de".into())
    );
    assert_eq!(
        detect_desktop_native_locale(&["sr-Cyrl"]),
        Some("sr".into())
    );
    assert_eq!(
        detect_desktop_native_locale(&["sr-Latn", "en"]),
        Some("en".into())
    );
    assert_eq!(
        detect_desktop_native_locale(&["uz-Latn"]),
        Some("uz".into())
    );
}

#[test]
fn recognizes_norwegian_language_tags() {
    assert_eq!(detect_desktop_native_locale(&["no"]), Some("no".into()));
    assert_eq!(detect_desktop_native_locale(&["nb-NO"]), Some("no".into()));
    assert_eq!(detect_desktop_native_locale(&["nn-NO"]), Some("no".into()));
}

#[allow(dead_code)]
fn _bundle_type(_bundle: Bundle) {}
