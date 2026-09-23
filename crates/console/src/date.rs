//! Calendar bounds used by the console subscription logic.
//!
//! Port of `packages/console/core/src/util/date.ts` (upstream 18ef3cc):
//! `get_week_bounds` returns a Monday-based seven-day window and
//! `get_monthly_bounds` anchors to the subscription day/time, rolling back a
//! month before the anchor and clamping short months.

/// A UTC calendar instant at second precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl DateTime {
    pub fn new(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    /// Convert to milliseconds since the Unix epoch.
    pub fn to_epoch_ms(self) -> i64 {
        let days = days_from_civil(self.year as i64, self.month as i64, self.day as i64);
        days * 86_400_000
            + self.hour as i64 * 3_600_000
            + self.minute as i64 * 60_000
            + self.second as i64 * 1_000
    }

    /// Build from milliseconds since the Unix epoch.
    pub fn from_epoch_ms(ms: i64) -> Self {
        let days = ms.div_euclid(86_400_000);
        let rem = ms.rem_euclid(86_400_000);
        let (year, month, day) = civil_from_days(days);
        Self {
            year: year as i32,
            month: month as u32,
            day: day as u32,
            hour: (rem / 3_600_000) as u32,
            minute: ((rem / 60_000) % 60) as u32,
            second: ((rem / 1_000) % 60) as u32,
        }
    }

    /// Day of week with Sunday = 0.
    pub fn weekday(self) -> u32 {
        let days = days_from_civil(self.year as i64, self.month as i64, self.day as i64);
        (days + 4).rem_euclid(7) as u32
    }
}

/// Monday-based, seven-day window containing `date`.
pub fn get_week_bounds(date: DateTime) -> (DateTime, DateTime) {
    let offset = (date.weekday() + 6) % 7;
    let start_days =
        days_from_civil(date.year as i64, date.month as i64, date.day as i64) - offset as i64;
    let (sy, sm, sd) = civil_from_days(start_days);
    let (ey, em, ed) = civil_from_days(start_days + 7);
    (
        DateTime::new(sy as i32, sm as u32, sd as u32, 0, 0, 0),
        DateTime::new(ey as i32, em as u32, ed as u32, 0, 0, 0),
    )
}

/// The subscription-anchored monthly window containing `now`.
pub fn get_monthly_bounds(now: DateTime, subscribed: DateTime) -> (DateTime, DateTime) {
    let day = subscribed.day;
    let (hh, mm, ss) = (subscribed.hour, subscribed.minute, subscribed.second);

    let anchor = |year: i64, month: i64| {
        let max = days_in_month(year, month) as u32;
        let day = day.min(max);
        DateTime::new(year as i32, month as u32, day, hh, mm, ss)
    };

    let mut y = now.year as i64;
    let mut m = now.month as i64;
    let mut start = anchor(y, m);
    if start > now {
        let total = y * 12 + (m - 1) - 1;
        y = total.div_euclid(12);
        m = total.rem_euclid(12) + 1;
        start = anchor(y, m);
    }
    let total = y * 12 + (m - 1) + 1;
    let (ny, nm) = (total.div_euclid(12), total.rem_euclid(12) + 1);
    let end = anchor(ny, nm);
    (start, end)
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Days since 1970-01-01 for a proleptic Gregorian date (Howard Hinnant).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Inverse of [`days_from_civil`].
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
