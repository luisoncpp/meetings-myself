use super::clock::Clock;
use super::domain_error::DomainError;
use super::ids::GoalId;
use super::lifecycle::{Achievement, Lifecycle};
use super::task::clean_title;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

pub struct CreateGoal<'a> {
    pub title: String,
    pub target_date: Option<NaiveDate>,
    pub clock: &'a dyn Clock,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: GoalId,
    pub title: String,
    pub achievement: Achievement,
    pub lifecycle: Lifecycle,
    pub target_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

impl Goal {
    pub fn create(request: CreateGoal<'_>) -> Result<Self, DomainError> {
        Ok(Self {
            id: GoalId::generate(),
            title: clean_title(request.title)?,
            achievement: Achievement::Pursuing,
            lifecycle: Lifecycle::Active,
            target_date: request.target_date,
            created_at: request.clock.now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::clock::FixedClock;
    use chrono::TimeZone;

    fn clock() -> FixedClock {
        FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap())
    }

    fn day(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, day).unwrap()
    }

    #[test]
    fn a_title_alone_is_enough_to_create_a_goal() {
        let goal = Goal::create(CreateGoal {
            title: "Career transition".into(),
            target_date: None,
            clock: &clock(),
        })
        .unwrap();
        assert_eq!(goal.title, "Career transition");
        assert_eq!(goal.achievement, Achievement::Pursuing);
        assert_eq!(goal.lifecycle, Lifecycle::Active);
        assert_eq!(goal.target_date, None);
    }

    #[test]
    fn blank_titles_are_rejected_and_whitespace_is_trimmed() {
        assert_eq!(
            Goal::create(CreateGoal {
                title: "   ".into(),
                target_date: None,
                clock: &clock(),
            }),
            Err(DomainError::BlankTitle)
        );
        let goal = Goal::create(CreateGoal {
            title: "  Ship it  ".into(),
            target_date: None,
            clock: &clock(),
        })
        .unwrap();
        assert_eq!(goal.title, "Ship it");
    }

    #[test]
    fn target_date_is_passed_through_as_given() {
        let with_date = Goal::create(CreateGoal {
            title: "Launch".into(),
            target_date: Some(day(15)),
            clock: &clock(),
        })
        .unwrap();
        assert_eq!(with_date.target_date, Some(day(15)));

        let without = Goal::create(CreateGoal {
            title: "Launch".into(),
            target_date: None,
            clock: &clock(),
        })
        .unwrap();
        assert_eq!(without.target_date, None);
    }
}
