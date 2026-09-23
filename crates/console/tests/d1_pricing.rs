//! Port of packages/console/app/test/pricing.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/console/app/src/routes/zen/util/pricing.ts:
//! peak pricing applies on Beijing (UTC+8) weekdays during 09:00–12:00 and
//! 14:00–18:00.
//! Re-derived: the timestamp is an explicit UTC calendar value.

use opencode_console::date::DateTime;
use opencode_console::pricing::is_peak_pricing;

fn dt(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> DateTime {
    DateTime::new(year, month, day, hour, minute, second)
}

#[test]
fn handles_weekday_peak_window_boundaries_in_beijing_time() {
    assert!(is_peak_pricing(dt(2026, 8, 27, 1, 0, 0)));
    assert!(!is_peak_pricing(dt(2026, 8, 27, 4, 0, 0)));
    assert!(is_peak_pricing(dt(2026, 8, 27, 6, 0, 0)));
    assert!(!is_peak_pricing(dt(2026, 8, 27, 10, 0, 0)));
}

#[test]
fn ignores_weekends_in_beijing_time() {
    assert!(!is_peak_pricing(dt(2026, 8, 29, 1, 0, 0)));
    assert!(!is_peak_pricing(dt(2026, 8, 30, 6, 0, 0)));
    assert!(is_peak_pricing(dt(2026, 8, 31, 1, 0, 0)));
}
