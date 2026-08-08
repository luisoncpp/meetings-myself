use super::calendar_week::CalendarWeek;
use super::clock::Clock;
use chrono::{Datelike, NaiveDate, Weekday};
use chrono_tz::Tz;

/// Projects instants onto dates in the synchronized home time zone.
///
/// Every date, week, deadline, and recurrence decision in the app goes through
/// here. Device time zones are never consulted (ADR 0001).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomeCalendar {
    zone: Tz,
}

impl HomeCalendar {
    pub fn new(zone: Tz) -> Self {
        Self { zone }
    }

    pub fn zone(&self) -> Tz {
        self.zone
    }

    pub fn today(&self, clock: &dyn Clock) -> NaiveDate {
        clock.now().with_timezone(&self.zone).date_naive()
    }

    pub fn current_week(&self, clock: &dyn Clock) -> CalendarWeek {
        CalendarWeek::containing(self.today(clock))
    }

    pub fn weekday(&self, date: NaiveDate) -> Weekday {
        date.weekday()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::clock::FixedClock;
    use chrono::{NaiveDate, TimeZone, Utc};
    use chrono_tz::Tz;

    fn clock() -> FixedClock {
        FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 1, 30, 0).unwrap())
    }

    #[test]
    fn today_follows_the_home_zone_not_utc() {
        let madrid = HomeCalendar::new(Tz::Europe__Madrid);
        assert_eq!(madrid.today(&clock()), NaiveDate::from_ymd_opt(2026, 8, 7).unwrap());

        let los_angeles = HomeCalendar::new(Tz::America__Los_Angeles);
        assert_eq!(los_angeles.today(&clock()), NaiveDate::from_ymd_opt(2026, 8, 6).unwrap());
    }

    #[test]
    fn the_home_zone_can_move_the_calendar_week() {
        let instant = FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 10, 0, 30, 0).unwrap());
        assert_eq!(HomeCalendar::new(Tz::Europe__Madrid).current_week(&instant).label(), "2026-W33");
        assert_eq!(
            HomeCalendar::new(Tz::America__Los_Angeles).current_week(&instant).label(),
            "2026-W32"
        );
    }
}
