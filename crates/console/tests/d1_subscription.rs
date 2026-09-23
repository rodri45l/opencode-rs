//! Port of packages/console/core/test/subscription.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/console/core/src/subscription.ts: monthly usage
//! is measured against the current subscription-anchored period; usage recorded
//! before the current period counts as zero; crossing the monthly boundary
//! resets usage; and the percentage is capped at 100.
//! Re-derived: `setSystemTime` is replaced by an explicit `now` input, and all
//! monetary values are integer micro-cents.

#[allow(dead_code)]
mod subscription {
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

    pub const NOTE: &str = "porting: console subscription usage not implemented";

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Status {
        Ok,
        RateLimited,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Analysis {
        pub status: Status,
        pub usage_percent: f64,
        pub reset_in_sec: i64,
    }

    pub fn cents_to_micro_cents(cents: i64) -> i64 {
        cents * 1_000_000
    }

    pub fn analyze_monthly_usage(
        _limit_micro_cents: i64,
        _usage_micro_cents: i64,
        _time_updated_ms: i64,
        _time_subscribed_ms: i64,
        _now_ms: i64,
    ) -> PortResult<Analysis> {
        Err(NotImplemented(NOTE))
    }
}

use subscription::{analyze_monthly_usage, cents_to_micro_cents, Status, NOTE};

const SUBSCRIBED: i64 = 1_768_464_000_000; // 2026-01-15T08:00:00Z

fn limit_micro_cents(dollars: i64) -> i64 {
    cents_to_micro_cents(dollars * 100)
}

#[test]
#[ignore = "porting: console subscription usage not implemented"]
fn returns_ok_with_zero_percent_when_usage_predates_the_current_period() {
    let now = 1_774_000_800_000; // 2026-03-20T10:00:00Z
    let result = analyze_monthly_usage(
        limit_micro_cents(10),
        cents_to_micro_cents(500),
        1_770_681_600_000, // 2026-02-10T00:00:00Z
        SUBSCRIBED,
        now,
    )
    .expect(NOTE);

    assert_eq!(result.status, Status::Ok);
    assert_eq!(result.usage_percent, 0.0);
    let expected = (1_776_240_000_000_i64 - now) / 1000;
    assert_eq!(result.reset_in_sec, expected);
}

#[test]
#[ignore = "porting: console subscription usage not implemented"]
fn returns_ok_with_usage_percent_when_under_limit() {
    let result = analyze_monthly_usage(
        limit_micro_cents(10),
        limit_micro_cents(10) / 2,
        1_773_792_000_000, // 2026-03-18T00:00:00Z
        SUBSCRIBED,
        1_774_000_800_000,
    )
    .expect(NOTE);

    assert_eq!(result.status, Status::Ok);
    assert_eq!(result.usage_percent, 50.0);
}

#[test]
#[ignore = "porting: console subscription usage not implemented"]
fn returns_rate_limited_when_at_or_over_limit() {
    let result = analyze_monthly_usage(
        limit_micro_cents(10),
        limit_micro_cents(10),
        1_773_792_000_000,
        SUBSCRIBED,
        1_774_000_800_000,
    )
    .expect(NOTE);

    assert_eq!(result.status, Status::RateLimited);
    assert_eq!(result.usage_percent, 100.0);
}

#[test]
#[ignore = "porting: console subscription usage not implemented"]
fn resets_usage_when_crossing_the_monthly_boundary() {
    let result = analyze_monthly_usage(
        limit_micro_cents(10),
        limit_micro_cents(10),
        1_773_964_800_000, // 2026-03-20T00:00:00Z (previous period)
        SUBSCRIBED,
        1_776_333_600_000, // 2026-04-16T10:00:00Z
    )
    .expect(NOTE);

    assert_eq!(result.status, Status::Ok);
    assert_eq!(result.usage_percent, 0.0);
}

#[test]
#[ignore = "porting: console subscription usage not implemented"]
fn caps_usage_percent_at_100() {
    let result = analyze_monthly_usage(
        limit_micro_cents(10),
        limit_micro_cents(10) - 1,
        1_773_792_000_000,
        SUBSCRIBED,
        1_774_000_800_000,
    )
    .expect(NOTE);

    assert_eq!(result.status, Status::Ok);
    assert!(result.usage_percent <= 100.0);
}

#[test]
#[ignore = "porting: console subscription usage not implemented"]
fn handles_subscription_day_31_in_a_short_month() {
    let sub31 = 1_769_860_800_000; // 2026-01-31T12:00:00Z
    let now = 1_772_359_200_000; // 2026-03-01T10:00:00Z
    let result = analyze_monthly_usage(
        limit_micro_cents(10),
        0,
        1_772_355_600_000, // 2026-03-01T09:00:00Z
        sub31,
        now,
    )
    .expect(NOTE);

    assert_eq!(result.status, Status::Ok);
    assert_eq!(result.usage_percent, 0.0);
    let expected = (1_774_958_400_000_i64 - now) / 1000;
    assert_eq!(result.reset_in_sec, expected);
}
