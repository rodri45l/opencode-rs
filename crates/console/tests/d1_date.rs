//! Port of packages/console/core/test/date.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/console/core/src/util/date.ts: `getWeekBounds`
//! returns a Monday-based, seven-day window, and `getMonthlyBounds` anchors to
//! the subscription day/time, rolling back a month before the anchor and
//! clamping short months.
//! Re-derived: dates are explicit UTC calendar values rather than `Date` objects.

#[allow(dead_code)]
mod date {
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

    pub const NOTE: &str = "porting: console date bounds not implemented";

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DateTime {
        pub year: i32,
        pub month: u32,
        pub day: u32,
        pub hour: u32,
        pub minute: u32,
        pub second: u32,
    }

    pub fn get_week_bounds(_date: DateTime) -> PortResult<(DateTime, DateTime)> {
        Err(NotImplemented(NOTE))
    }

    pub fn get_monthly_bounds(
        _now: DateTime,
        _subscribed: DateTime,
    ) -> PortResult<(DateTime, DateTime)> {
        Err(NotImplemented(NOTE))
    }
}

use date::{get_monthly_bounds, get_week_bounds, DateTime, NOTE};

fn dt(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> DateTime {
    DateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
    }
}

#[test]
#[ignore = "porting: console date bounds not implemented"]
fn returns_a_monday_based_week_for_sunday_dates() {
    let (start, end) = get_week_bounds(dt(2026, 1, 18, 12, 0, 0)).expect(NOTE);
    assert_eq!(start, dt(2026, 1, 12, 0, 0, 0));
    assert_eq!(end, dt(2026, 1, 19, 0, 0, 0));
}

#[test]
#[ignore = "porting: console date bounds not implemented"]
fn returns_a_seven_day_window() {
    let (start, end) = get_week_bounds(dt(2026, 1, 14, 12, 0, 0)).expect(NOTE);
    assert_eq!(start, dt(2026, 1, 12, 0, 0, 0));
    assert_eq!(end, dt(2026, 1, 19, 0, 0, 0));
}

#[test]
#[ignore = "porting: console date bounds not implemented"]
fn resets_on_subscription_day_mid_month() {
    let (start, end) =
        get_monthly_bounds(dt(2026, 3, 20, 10, 0, 0), dt(2026, 1, 15, 8, 0, 0)).expect(NOTE);
    assert_eq!(start, dt(2026, 3, 15, 8, 0, 0));
    assert_eq!(end, dt(2026, 4, 15, 8, 0, 0));
}

#[test]
#[ignore = "porting: console date bounds not implemented"]
fn before_subscription_day_in_current_month_uses_previous_month_anchor() {
    let (start, end) =
        get_monthly_bounds(dt(2026, 3, 10, 10, 0, 0), dt(2026, 1, 15, 8, 0, 0)).expect(NOTE);
    assert_eq!(start, dt(2026, 2, 15, 8, 0, 0));
    assert_eq!(end, dt(2026, 3, 15, 8, 0, 0));
}

#[test]
#[ignore = "porting: console date bounds not implemented"]
fn clamps_day_for_short_months() {
    let (start, end) =
        get_monthly_bounds(dt(2026, 3, 1, 10, 0, 0), dt(2026, 1, 31, 12, 0, 0)).expect(NOTE);
    assert_eq!(start, dt(2026, 2, 28, 12, 0, 0));
    assert_eq!(end, dt(2026, 3, 31, 12, 0, 0));
}

#[test]
#[ignore = "porting: console date bounds not implemented"]
fn handles_subscription_on_the_first() {
    let (start, end) =
        get_monthly_bounds(dt(2026, 4, 15, 0, 0, 0), dt(2026, 1, 1, 0, 0, 0)).expect(NOTE);
    assert_eq!(start, dt(2026, 4, 1, 0, 0, 0));
    assert_eq!(end, dt(2026, 5, 1, 0, 0, 0));
}

#[test]
#[ignore = "porting: console date bounds not implemented"]
fn exactly_on_the_reset_boundary_uses_the_current_period() {
    let (start, end) =
        get_monthly_bounds(dt(2026, 3, 15, 8, 0, 0), dt(2026, 1, 15, 8, 0, 0)).expect(NOTE);
    assert_eq!(start, dt(2026, 3, 15, 8, 0, 0));
    assert_eq!(end, dt(2026, 4, 15, 8, 0, 0));
}

#[test]
#[ignore = "porting: console date bounds not implemented"]
fn february_to_march_with_day_30_subscription() {
    let (start, end) =
        get_monthly_bounds(dt(2026, 2, 15, 6, 0, 0), dt(2025, 12, 30, 6, 0, 0)).expect(NOTE);
    assert_eq!(start, dt(2026, 1, 30, 6, 0, 0));
    assert_eq!(end, dt(2026, 2, 28, 6, 0, 0));
}
