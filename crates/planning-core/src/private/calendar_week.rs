use chrono::{Datelike, Duration, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CalendarError {
    #[error("'{0}' is not a valid ISO week label such as 2026-W32")]
    InvalidWeekLabel(String),
}

/// A Monday-through-Sunday ISO-8601 week. Serialized as its label so report
/// front matter and database records stay human-readable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CalendarWeek {
    iso_year: i32,
    iso_week: u32,
}

impl CalendarWeek {
    pub fn containing(date: NaiveDate) -> Self {
        let iso = date.iso_week();
        Self {
            iso_year: iso.year(),
            iso_week: iso.week(),
        }
    }

    pub fn monday(&self) -> NaiveDate {
        NaiveDate::from_isoywd_opt(self.iso_year, self.iso_week, Weekday::Mon)
            .expect("CalendarWeek can only hold weeks that exist")
    }

    pub fn sunday(&self) -> NaiveDate {
        self.monday() + Duration::days(6)
    }

    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.monday() && date <= self.sunday()
    }

    pub fn label(&self) -> String {
        format!("{}-W{:02}", self.iso_year, self.iso_week)
    }

    pub fn next(&self) -> Self {
        Self::containing(self.monday() + Duration::days(7))
    }

    pub fn previous(&self) -> Self {
        Self::containing(self.monday() - Duration::days(7))
    }

    pub fn parse(label: &str) -> Result<Self, CalendarError> {
        let invalid = || CalendarError::InvalidWeekLabel(label.to_string());
        let (year, week) = label.split_once("-W").ok_or_else(invalid)?;
        let iso_year: i32 = year.parse().map_err(|_| invalid())?;
        let iso_week: u32 = week.parse().map_err(|_| invalid())?;
        NaiveDate::from_isoywd_opt(iso_year, iso_week, Weekday::Mon).ok_or_else(invalid)?;
        Ok(Self { iso_year, iso_week })
    }
}

impl TryFrom<String> for CalendarWeek {
    type Error = CalendarError;

    fn try_from(label: String) -> Result<Self, Self::Error> {
        Self::parse(&label)
    }
}

impl From<CalendarWeek> for String {
    fn from(week: CalendarWeek) -> Self {
        week.label()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    #[test]
    fn weeks_run_monday_to_sunday() {
        let week = CalendarWeek::containing(date(2026, 8, 6));
        assert_eq!(week.label(), "2026-W32");
        assert_eq!(week.monday(), date(2026, 8, 3));
        assert_eq!(week.sunday(), date(2026, 8, 9));
        assert!(week.contains(date(2026, 8, 9)));
        assert!(!week.contains(date(2026, 8, 10)));
    }

    #[test]
    fn iso_years_do_not_follow_the_calendar_year() {
        assert_eq!(
            CalendarWeek::containing(date(2025, 12, 29)).label(),
            "2026-W01"
        );
        assert_eq!(
            CalendarWeek::containing(date(2025, 12, 28)).label(),
            "2025-W52"
        );
        assert_eq!(
            CalendarWeek::containing(date(2027, 1, 3)).label(),
            "2026-W53"
        );
    }

    #[test]
    fn next_and_previous_cross_the_year_boundary() {
        let last = CalendarWeek::containing(date(2027, 1, 3));
        assert_eq!(last.label(), "2026-W53");
        assert_eq!(last.next().label(), "2027-W01");
        assert_eq!(
            CalendarWeek::containing(date(2026, 1, 5))
                .previous()
                .label(),
            "2026-W01"
        );
    }

    #[test]
    fn labels_round_trip() {
        let week = CalendarWeek::containing(date(2026, 8, 6));
        assert_eq!(CalendarWeek::parse("2026-W32").unwrap(), week);
        assert!(CalendarWeek::parse("2026-32").is_err());
        assert!(CalendarWeek::parse("2026-W99").is_err());
    }
}
