//! Port of packages/app/src/i18n/desktop-native.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeMap;

const MAX_PAYLOAD_BYTES: usize = 65_536;

fn native_locales() -> Vec<&'static str> {
    vec![
        "en", "zh", "zht", "ko", "de", "es", "fr", "da", "ja", "pl", "ru", "uk", "bs", "ar", "no",
        "br", "th", "tr", "hi", "nl", "id", "vi", "it", "ur", "pa", "az", "fi", "sv", "am", "bg",
        "bn", "ca", "cs", "dv", "dz", "el", "et", "fa", "fo", "hr", "hu", "hy", "is", "ka", "km",
        "lo", "lt", "lv", "mk", "mn", "ms", "my", "ne", "ro", "si", "sk", "sl", "sq", "sr", "tg",
        "tk", "uz",
    ]
}

fn native_labels() -> BTreeMap<&'static str, &'static str> {
    [
        ("en", "English"),
        ("zh", "简体中文"),
        ("zht", "繁體中文"),
        ("ko", "한국어"),
        ("de", "Deutsch"),
        ("es", "Español"),
        ("fr", "Français"),
        ("da", "Dansk"),
        ("ja", "日本語"),
        ("pl", "Polski"),
        ("ru", "Русский"),
        ("uk", "Українська"),
        ("bs", "Bosanski"),
        ("ar", "العربية"),
        ("no", "Norsk"),
        ("br", "Português (Brasil)"),
        ("th", "ไทย"),
        ("tr", "Türkçe"),
        ("hi", "हिन्दी"),
        ("nl", "Nederlands"),
        ("id", "Bahasa Indonesia"),
        ("vi", "Tiếng Việt"),
        ("it", "Italiano"),
        ("ur", "اردو"),
        ("pa", "پنجابی"),
        ("az", "Azərbaycanca"),
        ("fi", "Suomi"),
        ("sv", "Svenska"),
        ("am", "አማርኛ"),
        ("bg", "Български"),
        ("bn", "বাংলা"),
        ("ca", "Català"),
        ("cs", "Čeština"),
        ("dv", "ދިވެހި"),
        ("dz", "རྫོང་ཁ"),
        ("el", "Ελληνικά"),
        ("et", "Eesti"),
        ("fa", "فارسی"),
        ("fo", "Føroyskt"),
        ("hr", "Hrvatski"),
        ("hu", "Magyar"),
        ("hy", "Հայերեն"),
        ("is", "Íslenska"),
        ("ka", "ქართული"),
        ("km", "ខ្មែរ"),
        ("lo", "ລາວ"),
        ("lt", "Lietuvių"),
        ("lv", "Latviešu"),
        ("mk", "Македонски"),
        ("mn", "Монгол"),
        ("ms", "Bahasa Melayu"),
        ("my", "မြန်မာ"),
        ("ne", "नेपाली"),
        ("ro", "Română"),
        ("si", "සිංහල"),
        ("sk", "Slovenčina"),
        ("sl", "Slovenščina"),
        ("sq", "Shqip"),
        ("sr", "Српски"),
        ("tg", "Тоҷикӣ"),
        ("tk", "Türkmençe"),
        ("uz", "Oʻzbekcha"),
    ]
    .into_iter()
    .collect()
}

#[derive(Clone, Debug, PartialEq)]
struct Bundle {
    locale: String,
    messages: BTreeMap<String, String>,
}

// Local stubs (fast wave): real module lands later.
fn detect_desktop_native_locale(_preferences: &[&str]) -> Option<String> {
    None
}

fn parse_desktop_native_bundle(_bundle: &Bundle) -> Option<Bundle> {
    None
}

fn format_desktop_native_message(_template: &str, _values: &BTreeMap<String, String>) -> String {
    String::new()
}

fn create_desktop_native_bundle(_locale: &str) -> Bundle {
    Bundle {
        locale: _locale.to_string(),
        messages: BTreeMap::new(),
    }
}

#[test]
#[ignore = "porting: i18n/desktop-native not implemented"]
fn uses_native_language_names_independent_of_the_active_locale() {
    let labels = native_labels();
    let actual: Vec<&str> = native_locales().iter().map(|l| labels[l]).collect();
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
#[ignore = "porting: i18n/desktop-native not implemented"]
fn accepts_the_exact_typed_bundle() {
    let bundle = create_desktop_native_bundle("en");
    assert_eq!(parse_desktop_native_bundle(&bundle), Some(bundle));
}

#[test]
#[ignore = "porting: i18n/desktop-native not implemented"]
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
        .insert("key".into(), "x".repeat(MAX_PAYLOAD_BYTES));
    assert_eq!(parse_desktop_native_bundle(&oversized), None);
}

#[test]
#[ignore = "porting: i18n/desktop-native not implemented"]
fn interpolates_native_templates_without_changing_unknown_placeholders() {
    let mut values = BTreeMap::new();
    values.insert("known".to_string(), "yes".to_string());
    assert_eq!(
        format_desktop_native_message("{{known}} {{unknown}}", &values),
        "yes {{unknown}}"
    );
}

#[test]
#[ignore = "porting: i18n/desktop-native not implemented"]
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
#[ignore = "porting: i18n/desktop-native not implemented"]
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
#[ignore = "porting: i18n/desktop-native not implemented"]
fn recognizes_norwegian_language_tags() {
    assert_eq!(detect_desktop_native_locale(&["no"]), Some("no".into()));
    assert_eq!(detect_desktop_native_locale(&["nb-NO"]), Some("no".into()));
    assert_eq!(detect_desktop_native_locale(&["nn-NO"]), Some("no".into()));
}
