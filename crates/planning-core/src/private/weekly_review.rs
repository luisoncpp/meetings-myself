use super::calendar_week::CalendarWeek;
use super::clock::Clock;
use super::ids::WeeklyReviewId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub struct StartReview<'a> {
    pub week: CalendarWeek,
    pub clock: &'a dyn Clock,
}

/// One review session per ISO calendar week. The record key is the week label,
/// so reopening never creates a duplicate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyReview {
    pub id: WeeklyReviewId,
    pub week: CalendarWeek,
    pub created_at: DateTime<Utc>,
    pub last_opened_at: DateTime<Utc>,
}

impl WeeklyReview {
    /// The record key IS the week label, so "one review per week" is a property
    /// of the store rather than something every caller must remember to check.
    pub fn key(week: CalendarWeek) -> String {
        week.label()
    }

    pub fn start(request: StartReview<'_>) -> Self {
        let now = request.clock.now();
        Self {
            id: WeeklyReviewId::new(Self::key(request.week)),
            week: request.week,
            created_at: now,
            last_opened_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::calendar_week::CalendarWeek;
    use crate::private::clock::FixedClock;
    use chrono::{NaiveDate, TimeZone, Utc};

    fn week() -> CalendarWeek {
        CalendarWeek::containing(NaiveDate::from_ymd_opt(2026, 8, 7).unwrap())
    }

    fn clock() -> FixedClock {
        FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap())
    }

    #[test]
    fn the_key_is_the_week_label_so_a_week_can_only_have_one_review() {
        assert_eq!(WeeklyReview::key(week()), "2026-W32");
    }

    #[test]
    fn starting_a_review_stamps_both_timestamps() {
        let review = WeeklyReview::start(StartReview {
            week: week(),
            clock: &clock(),
        });
        assert_eq!(review.created_at, review.last_opened_at);
        assert_eq!(review.id, WeeklyReviewId::new("2026-W32"));
    }
}
