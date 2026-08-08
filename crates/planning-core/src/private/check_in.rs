use super::clock::Clock;
use super::ids::{HabitCheckInId, HabitId};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CheckInOutcome {
    Done,
    Skipped,
    NotCompleted,
}

pub struct RecordCheckIn<'a> {
    pub habit: HabitId,
    pub date: NaiveDate,
    pub outcome: CheckInOutcome,
    pub clock: &'a dyn Clock,
}

/// One outcome for one habit on one calendar day. Correcting a check-in is an
/// upsert keyed by habit and date, not a second record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitCheckIn {
    pub id: HabitCheckInId,
    pub habit: HabitId,
    pub date: NaiveDate,
    pub outcome: CheckInOutcome,
    pub recorded_at: DateTime<Utc>,
}

impl HabitCheckIn {
    /// The record key pairs habit and date, so "one outcome per habit per day"
    /// is a property of the store rather than something every caller must check.
    pub fn key(habit: &HabitId, date: NaiveDate) -> String {
        format!("{}:{}", habit, date.format("%Y-%m-%d"))
    }

    pub fn record(request: RecordCheckIn<'_>) -> Self {
        Self {
            id: HabitCheckInId::new(Self::key(&request.habit, request.date)),
            habit: request.habit,
            date: request.date,
            outcome: request.outcome,
            recorded_at: request.clock.now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::clock::FixedClock;
    use chrono::TimeZone;

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 7).unwrap()
    }

    fn clock() -> FixedClock {
        FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap())
    }

    #[test]
    fn the_key_pairs_habit_and_date_so_one_day_holds_one_outcome() {
        let habit = HabitId::new("h1");
        assert_eq!(HabitCheckIn::key(&habit, day()), "h1:2026-08-07");
    }

    #[test]
    fn recording_the_same_day_twice_produces_the_same_key_so_it_corrects_rather_than_appends()
    {
        let habit = HabitId::new("h1");
        let first = HabitCheckIn::record(RecordCheckIn {
            habit: habit.clone(),
            date: day(),
            outcome: CheckInOutcome::Done,
            clock: &clock(),
        });
        let corrected = HabitCheckIn::record(RecordCheckIn {
            habit: habit.clone(),
            date: day(),
            outcome: CheckInOutcome::Skipped,
            clock: &clock(),
        });
        assert_eq!(first.id, corrected.id);
        assert_eq!(corrected.outcome, CheckInOutcome::Skipped);
    }

    #[test]
    fn outcomes_serialize_as_camel_case_for_the_frontend() {
        assert_eq!(
            serde_json::to_string(&CheckInOutcome::NotCompleted).unwrap(),
            r#""notCompleted""#
        );
    }
}
