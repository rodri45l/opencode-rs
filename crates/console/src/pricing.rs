//! DeepSeek peak-pricing window in Beijing time.
//!
//! Port of `packages/console/app/src/routes/zen/util/pricing.ts`
//! (upstream 18ef3cc): peak on Beijing (UTC+8) weekdays, 09:00-12:00 and
//! 14:00-18:00.

use crate::date::DateTime;

/// Whether `now` (UTC) falls inside the Beijing peak window.
pub fn is_peak_pricing(now: DateTime) -> bool {
    let total_hours = now.hour + 8;
    let day_shift = total_hours / 24;
    let hour_cn = total_hours % 24;
    let weekday = (DateTime::new(now.year, now.month, now.day, 0, 0, 0).weekday() + day_shift) % 7;
    if weekday == 0 || weekday == 6 {
        return false;
    }
    (9..12).contains(&hour_cn) || (14..18).contains(&hour_cn)
}
