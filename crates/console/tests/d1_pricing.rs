//! Port of packages/console/app/test/pricing.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/console/app/src/routes/zen/util/pricing.ts:
//! peak pricing applies on Beijing (UTC+8) weekdays during 09:00–12:00 and
//! 14:00–18:00.
//! Re-derived: the timestamp is an explicit UTC calendar value.

#[allow(dead_code)]
mod pricing {
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

    pub const NOTE: &str = "porting: console peak pricing not implemented";

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DateTime {
        pub year: i32,
        pub month: u32,
        pub day: u32,
        pub hour: u32,
        pub minute: u32,
        pub second: u32,
    }

    pub fn is_peak_pricing(_now: DateTime) -> PortResult<bool> {
        Err(NotImplemented(NOTE))
    }
}

use pricing::{is_peak_pricing, DateTime, NOTE};

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
#[ignore = "porting: console peak pricing not implemented"]
fn handles_weekday_peak_window_boundaries_in_beijing_time() {
    assert!(is_peak_pricing(dt(2026, 8, 27, 1, 0, 0)).expect(NOTE));
    assert!(!is_peak_pricing(dt(2026, 8, 27, 4, 0, 0)).expect(NOTE));
    assert!(is_peak_pricing(dt(2026, 8, 27, 6, 0, 0)).expect(NOTE));
    assert!(!is_peak_pricing(dt(2026, 8, 27, 10, 0, 0)).expect(NOTE));
}

#[test]
#[ignore = "porting: console peak pricing not implemented"]
fn ignores_weekends_in_beijing_time() {
    assert!(!is_peak_pricing(dt(2026, 8, 29, 1, 0, 0)).expect(NOTE));
    assert!(!is_peak_pricing(dt(2026, 8, 30, 6, 0, 0)).expect(NOTE));
    assert!(is_peak_pricing(dt(2026, 8, 31, 1, 0, 0)).expect(NOTE));
}
