use super::error::AppError;
use super::service::PlanningApp;
use chrono::NaiveDate;
use planning_core::{CheckInOutcome, HabitCheckIn, HabitCheckInId, HabitId, RecordCheckIn};

pub struct CheckInRequest {
    pub habit: HabitId,
    pub date: NaiveDate,
    pub outcome: CheckInOutcome,
}

pub struct DateRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

impl PlanningApp {
    /// Records or corrects one Habit's outcome for one day. Deliberately does NOT
    /// check the habit's lifecycle or cadence: archived habits already in a plan
    /// stay completable, and any past day stays correctable (ADR 0002).
    pub async fn record_check_in(&self, request: CheckInRequest) -> Result<HabitCheckIn, AppError> {
        let record = HabitCheckIn::record(RecordCheckIn {
            habit: request.habit,
            date: request.date,
            outcome: request.outcome,
            clock: self.clock.as_ref(),
        });
        self.store(HabitCheckInId::TABLE, record.id.as_str(), &record)
            .await?;
        Ok(record)
    }

    pub async fn check_in_for(
        &self,
        habit: &HabitId,
        date: NaiveDate,
    ) -> Result<Option<HabitCheckIn>, AppError> {
        self.load_one(HabitCheckInId::TABLE, &HabitCheckIn::key(habit, date))
            .await
    }

    pub async fn check_ins_between(&self, range: DateRange) -> Result<Vec<HabitCheckIn>, AppError> {
        Ok(self
            .load_all::<HabitCheckIn>(HabitCheckInId::TABLE)
            .await?
            .into_iter()
            .filter(|found| found.date >= range.from && found.date <= range.to)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::library::NewHabit;
    use crate::private::test_support::ready_app_at;
    use chrono::{Duration, TimeZone, Utc};
    use planning_core::{Cadence, FixedClock};
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn app_on(day: u32) -> (TempDir, TempDir, PlanningApp, Arc<FixedClock>) {
        ready_app_at(Utc.with_ymd_and_hms(2026, 8, day, 9, 0, 0).unwrap()).await
    }

    #[tokio::test]
    async fn all_three_outcomes_are_recordable_and_nothing_else_exists() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let today = app.calendar().unwrap().today(app.clock_ref());
        let habit = app
            .create_habit(NewHabit {
                title: "Writing".into(),
                cadence: Cadence::EveryDay,
            })
            .await
            .unwrap();

        for outcome in [
            CheckInOutcome::Done,
            CheckInOutcome::Skipped,
            CheckInOutcome::NotCompleted,
        ] {
            app.record_check_in(CheckInRequest {
                habit: habit.id.clone(),
                date: today,
                outcome,
            })
            .await
            .unwrap();
            assert_eq!(
                app.check_in_for(&habit.id, today)
                    .await
                    .unwrap()
                    .unwrap()
                    .outcome,
                outcome
            );
        }
    }

    #[tokio::test]
    async fn correcting_a_past_day_replaces_the_outcome_rather_than_adding_one() {
        let (_home, _drive, app, clock) = app_on(7).await;
        let today = app.calendar().unwrap().today(app.clock_ref());
        let habit = app
            .create_habit(NewHabit {
                title: "Writing".into(),
                cadence: Cadence::EveryDay,
            })
            .await
            .unwrap();
        let yesterday = today - Duration::days(1);

        app.record_check_in(CheckInRequest {
            habit: habit.id.clone(),
            date: yesterday,
            outcome: CheckInOutcome::NotCompleted,
        })
        .await
        .unwrap();

        clock.advance(Duration::days(5));
        // Still correctable days later (ADR 0002).
        app.record_check_in(CheckInRequest {
            habit: habit.id.clone(),
            date: yesterday,
            outcome: CheckInOutcome::Done,
        })
        .await
        .unwrap();

        assert_eq!(
            app.check_in_for(&habit.id, yesterday)
                .await
                .unwrap()
                .unwrap()
                .outcome,
            CheckInOutcome::Done
        );
        assert_eq!(
            app.check_ins_between(DateRange {
                from: yesterday,
                to: yesterday
            })
            .await
            .unwrap()
            .len(),
            1
        );
    }

    #[tokio::test]
    async fn an_archived_habit_can_still_be_checked_in_for_a_day_it_already_appears_in() {
        let (_home, _drive, app, _clock) = app_on(7).await;
        let today = app.calendar().unwrap().today(app.clock_ref());
        let habit = app
            .create_habit(NewHabit {
                title: "Writing".into(),
                cadence: Cadence::EveryDay,
            })
            .await
            .unwrap();
        app.open_today().await.unwrap();

        app.archive_habit(&habit.id).await.unwrap();

        // A6: the entry remains and stays completable.
        assert!(app.open_today().await.unwrap().habits.contains(&habit.id));
        app.record_check_in(CheckInRequest {
            habit: habit.id.clone(),
            date: today,
            outcome: CheckInOutcome::Done,
        })
        .await
        .unwrap();
        assert!(app.check_in_for(&habit.id, today).await.unwrap().is_some());
    }
}
