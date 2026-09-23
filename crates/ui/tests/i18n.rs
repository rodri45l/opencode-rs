//! Port of packages/ui/src/context/i18n.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/ui/src/context/i18n.ts; see docs/TEST-PORT.md.

use opencode_ui::i18n::plural_category;

#[test]
fn selects_plural_categories() {
    let cases: &[(&str, i64, &str)] = &[
        ("en", 0, "other"),
        ("en", 1, "one"),
        ("fr", 0, "one"),
        ("fr", 1_000_000, "many"),
        ("ru", 1, "one"),
        ("ru", 2, "few"),
        ("ru", 5, "many"),
        ("ru", 21, "one"),
        ("ar", 0, "zero"),
        ("ar", 1, "one"),
        ("ar", 2, "two"),
        ("ar", 3, "few"),
        ("ar", 11, "many"),
        ("ar", 100, "other"),
        ("ja", 1, "other"),
    ];
    for (locale, count, expected) in cases {
        assert_eq!(
            plural_category(locale, *count),
            *expected,
            "locale={locale} count={count}"
        );
    }
}
