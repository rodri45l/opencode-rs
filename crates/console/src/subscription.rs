//! Subscription usage analysis.
//!
//! Port of `packages/console/core/src/subscription.ts`
//! (`analyzeMonthlyUsage`, upstream 18ef3cc): monthly usage is measured against
//! the current subscription-anchored period; usage recorded before the current
//! period counts as zero; the percentage is capped at 100.

use crate::date::{get_monthly_bounds, DateTime};

/// The rate-limit status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok,
    RateLimited,
}

/// The result of a monthly usage analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct Analysis {
    pub status: Status,
    pub usage_percent: f64,
    pub reset_in_sec: i64,
}

/// Convert cents to micro-cents.
pub fn cents_to_micro_cents(cents: i64) -> i64 {
    cents * 1_000_000
}

/// Analyze monthly usage for the subscription-anchored period.
pub fn analyze_monthly_usage(
    limit_micro_cents: i64,
    usage_micro_cents: i64,
    time_updated_ms: i64,
    time_subscribed_ms: i64,
    now_ms: i64,
) -> Analysis {
    let now = DateTime::from_epoch_ms(now_ms);
    let subscribed = DateTime::from_epoch_ms(time_subscribed_ms);
    let (start, end) = get_monthly_bounds(now, subscribed);
    let reset_in_sec = ((end.to_epoch_ms() - now_ms) as f64 / 1000.0).ceil() as i64;

    if time_updated_ms < start.to_epoch_ms() {
        return Analysis {
            status: Status::Ok,
            usage_percent: 0.0,
            reset_in_sec,
        };
    }
    if usage_micro_cents < limit_micro_cents {
        let percent = ((usage_micro_cents as f64 / limit_micro_cents as f64) * 100.0)
            .min(100.0)
            .floor();
        return Analysis {
            status: Status::Ok,
            usage_percent: percent,
            reset_in_sec,
        };
    }
    Analysis {
        status: Status::RateLimited,
        usage_percent: 100.0,
        reset_in_sec,
    }
}
