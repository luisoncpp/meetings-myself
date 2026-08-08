use serde::{Deserialize, Serialize};

/// Declares a transparent, table-aware identifier newtype.
///
/// Distinct types per entity make it a compile error to pass a `GoalId` where a
/// `TaskId` belongs — the cheapest guard available against link-table mistakes.
macro_rules! define_ids {
    ($($name:ident => $table:literal),+ $(,)?) => { $(
        #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub const TABLE: &'static str = $table;

            pub fn new(raw: impl Into<String>) -> Self {
                Self(raw.into())
            }

            /// UUID v7 keeps ids time-ordered, which keeps storage key locality sane.
            pub fn generate() -> Self {
                Self(uuid::Uuid::now_v7().to_string())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    )+ };
}

define_ids! {
    ValueId => "value",
    GoalId => "goal",
    HabitId => "habit",
    TaskId => "task",
    RecurringTaskId => "recurring_task",
    AssociationId => "association",
    DailyPlanId => "daily_plan",
    WeeklyFocusId => "weekly_focus",
    WeeklyReviewId => "weekly_review",
    HabitCheckInId => "habit_check_in",
    OccurrenceId => "occurrence",
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_carry_their_table() {
        assert_eq!(TaskId::TABLE, "task");
        assert_ne!(TaskId::generate(), TaskId::generate());
    }

    #[test]
    fn ids_serialize_as_bare_strings() {
        let id = TaskId::new("abc");
        assert_eq!(serde_json::to_string(&id).unwrap(), "\"abc\"");
        assert_eq!(id.to_string(), "abc");
    }
}
