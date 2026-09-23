//! Port of packages/tui/test/util/format.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/tui/src/util/format.ts; see docs/TEST-PORT.md.

use opencode_tui::format::format_duration;

#[test]
fn returns_empty_string_for_zero_or_negative_values() {
    assert_eq!(format_duration(0), "");
    assert_eq!(format_duration(-1), "");
    assert_eq!(format_duration(-100), "");
}

#[test]
fn formats_seconds_under_a_minute() {
    assert_eq!(format_duration(1), "1s");
    assert_eq!(format_duration(30), "30s");
    assert_eq!(format_duration(59), "59s");
}

#[test]
fn formats_minutes_under_an_hour() {
    assert_eq!(format_duration(60), "1m");
    assert_eq!(format_duration(61), "1m 1s");
    assert_eq!(format_duration(90), "1m 30s");
    assert_eq!(format_duration(120), "2m");
    assert_eq!(format_duration(330), "5m 30s");
    assert_eq!(format_duration(3599), "59m 59s");
}

#[test]
fn formats_hours_under_a_day() {
    assert_eq!(format_duration(3600), "1h");
    assert_eq!(format_duration(3660), "1h 1m");
    assert_eq!(format_duration(7200), "2h");
    assert_eq!(format_duration(8100), "2h 15m");
    assert_eq!(format_duration(86399), "23h 59m");
}

#[test]
fn formats_days_under_a_week() {
    assert_eq!(format_duration(86400), "~1 day");
    assert_eq!(format_duration(172800), "~2 days");
    assert_eq!(format_duration(259200), "~3 days");
    assert_eq!(format_duration(604799), "~6 days");
}

#[test]
fn formats_weeks() {
    assert_eq!(format_duration(604800), "~1 week");
    assert_eq!(format_duration(1209600), "~2 weeks");
    assert_eq!(format_duration(1609200), "~2 weeks");
}

#[test]
fn handles_boundary_values_correctly() {
    assert_eq!(format_duration(59), "59s");
    assert_eq!(format_duration(60), "1m");
    assert_eq!(format_duration(3599), "59m 59s");
    assert_eq!(format_duration(3600), "1h");
    assert_eq!(format_duration(86399), "23h 59m");
    assert_eq!(format_duration(86400), "~1 day");
    assert_eq!(format_duration(604799), "~6 days");
    assert_eq!(format_duration(604800), "~1 week");
}
