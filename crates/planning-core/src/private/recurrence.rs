use super::domain_error::DomainError;
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

/// The recurrence patterns agreed for v1. A rule is a factory: editing it never
/// touches occurrences that already materialized (ADR 0002).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Recurrence {
    Daily,
    Weekdays,
    Weekly { weekday: Weekday },
    MonthlyDay { day: u8 },
}

impl Recurrence {
    pub fn monthly(day: u8) -> Result<Self, DomainError> {
        if day == 0 || day > 31 {
            return Err(DomainError::InvalidMonthDay);
        }
        Ok(Self::MonthlyDay { day })
    }

    pub fn occurs_on(&self, date: NaiveDate) -> bool {
        match self {
            Self::Daily => true,
            Self::Weekdays => !matches!(date.weekday(), Weekday::Sat | Weekday::Sun),
            Self::Weekly { weekday } => date.weekday() == *weekday,
            Self::MonthlyDay { day } => date.day() == effective_day(*day, date),
        }
    }
}

/// "The 31st" in a 30-day month means the 30th, not "skipped". Skipping would
/// make a monthly rule silently vanish for five months a year.
fn effective_day(wanted: u8, in_month_of: NaiveDate) -> u32 {
    let last = last_day_of_month(in_month_of);
    u32::from(wanted).min(last)
}

fn last_day_of_month(date: NaiveDate) -> u32 {
    let first_of_this = date.with_day(1).expect("day 1 always exists");
    let first_of_next = match first_of_this.month() {
        12 => NaiveDate::from_ymd_opt(first_of_this.year() + 1, 1, 1),
        month => NaiveDate::from_ymd_opt(first_of_this.year(), month + 1, 1),
    }
    .expect("the first of the next month always exists");
    (first_of_next - Duration::days(1)).day()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn daily_occurs_every_day() {
        assert!(Recurrence::Daily.occurs_on(date(2026, 8, 9))); // a Sunday
    }

    #[test]
    fn weekdays_skips_the_weekend() {
        assert!(Recurrence::Weekdays.occurs_on(date(2026, 8, 7))); // Friday
        assert!(!Recurrence::Weekdays.occurs_on(date(2026, 8, 8))); // Saturday
        assert!(!Recurrence::Weekdays.occurs_on(date(2026, 8, 9))); // Sunday
        assert!(Recurrence::Weekdays.occurs_on(date(2026, 8, 10))); // Monday
    }

    #[test]
    fn weekly_occurs_on_its_weekday_only() {
        let rule = Recurrence::Weekly {
            weekday: Weekday::Thu,
        };
        assert!(rule.occurs_on(date(2026, 8, 6)));
        assert!(!rule.occurs_on(date(2026, 8, 7)));
        assert!(rule.occurs_on(date(2026, 8, 13)));
    }

    #[test]
    fn monthly_clamps_to_the_last_day_of_shorter_months() {
        let rule = Recurrence::monthly(31).unwrap();
        assert!(rule.occurs_on(date(2026, 8, 31)));
        assert!(!rule.occurs_on(date(2026, 8, 30)));
        // 2026 is not a leap year: February has 28 days.
        assert!(rule.occurs_on(date(2026, 2, 28)));
        assert!(rule.occurs_on(date(2026, 4, 30)));
        assert!(!rule.occurs_on(date(2026, 4, 29)));
        // A leap year moves the clamp.
        assert!(rule.occurs_on(date(2028, 2, 29)));
        assert!(!rule.occurs_on(date(2028, 2, 28)));
    }

    #[test]
    fn monthly_rejects_impossible_days() {
        assert!(Recurrence::monthly(0).is_err());
        assert!(Recurrence::monthly(32).is_err());
        assert!(Recurrence::monthly(1).is_ok());
    }
}
