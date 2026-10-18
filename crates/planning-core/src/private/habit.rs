use super::cadence::Cadence;
use super::clock::Clock;
use super::domain_error::DomainError;
use super::ids::HabitId;
use super::lifecycle::Lifecycle;
use super::task::clean_title;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The qualitative stage of a Habit. Manual — the app never scores it (PRODUCT.md:
/// reflect, don't score).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HabitStrength {
    #[default]
    ReminderDependent,
    CueTriggered,
    Strengthening,
    Established,
}

pub struct CreateHabit<'a> {
    pub title: String,
    pub cadence: Cadence,
    pub clock: &'a dyn Clock,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Habit {
    pub id: HabitId,
    pub title: String,
    pub lifecycle: Lifecycle,
    pub strength: HabitStrength,
    pub cadence: Cadence,
    pub pinned: bool,
    pub created_at: DateTime<Utc>,
}

impl Habit {
    /// Unlike a Task, a Habit cannot be created from a title alone — a cadence is
    /// required, because a Habit with no due days would never appear anywhere.
    pub fn create(request: CreateHabit<'_>) -> Result<Self, DomainError> {
        Ok(Self {
            id: HabitId::generate(),
            title: Self::accept_title(request.title)?,
            lifecycle: Lifecycle::Active,
            strength: HabitStrength::ReminderDependent,
            cadence: request.cadence,
            pinned: true,
            created_at: request.clock.now(),
        })
    }

    /// The title that may be stored: trimmed, and never blank. Creation and
    /// rename share this rule so a habit cannot lose its name.
    pub fn accept_title(title: String) -> Result<String, DomainError> {
        clean_title(title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::clock::FixedClock;
    use chrono::{TimeZone, Weekday};

    #[test]
    fn a_new_habit_is_pinned_and_reminder_dependent() {
        let clock = FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap());
        let habit = Habit::create(CreateHabit {
            title: "Writing practice".into(),
            cadence: Cadence::new_on_weekdays(&[Weekday::Mon, Weekday::Thu]).unwrap(),
            clock: &clock,
        })
        .unwrap();

        assert!(habit.pinned, "new habits are pinned by default");
        assert_eq!(habit.strength, HabitStrength::ReminderDependent);
        assert_eq!(habit.lifecycle, Lifecycle::Active);
        assert!(habit.cadence.is_due(Weekday::Thu));
    }

    #[test]
    fn a_renamed_title_is_trimmed() {
        let title = Habit::accept_title("  Evening pages  ".into()).unwrap();
        assert_eq!(title, "Evening pages");
    }

    #[test]
    fn a_blank_title_is_rejected() {
        let error = Habit::accept_title("   ".into()).unwrap_err();
        assert_eq!(error, DomainError::BlankTitle);
    }
}
