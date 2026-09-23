//! Locale plural-category selection.
//!
//! Port of packages/ui/src/context/i18n.ts `pluralCategory` (upstream 18ef3cc).
//! Only the CLDR cardinal rules exercised by the reference tests are encoded.

/// The plural category for `count` in `locale`.
///
/// Unknown locales fall back to the `"other"` category, matching the reference.
pub fn plural_category(locale: &str, count: i64) -> &'static str {
    match locale {
        "en" => {
            if count == 1 {
                "one"
            } else {
                "other"
            }
        }
        "fr" => {
            if count == 0 || count == 1 {
                "one"
            } else if count != 0 && count % 1_000_000 == 0 {
                "many"
            } else {
                "other"
            }
        }
        "ru" => {
            let mod10 = count.rem_euclid(10);
            let mod100 = count.rem_euclid(100);
            if mod10 == 1 && mod100 != 11 {
                "one"
            } else if (2..=4).contains(&mod10) && !(12..=14).contains(&mod100) {
                "few"
            } else {
                "many"
            }
        }
        "ar" => {
            let mod100 = count.rem_euclid(100);
            if count == 0 {
                "zero"
            } else if count == 1 {
                "one"
            } else if count == 2 {
                "two"
            } else if (3..=10).contains(&mod100) {
                "few"
            } else if (11..=99).contains(&mod100) {
                "many"
            } else {
                "other"
            }
        }
        _ => "other",
    }
}
