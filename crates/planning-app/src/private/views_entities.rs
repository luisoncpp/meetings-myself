use chrono::NaiveDate;
use planning_core::{Cadence, Goal, GoalId, Habit, HabitId, HabitStrength, Value, ValueId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueView {
    pub id: ValueId,
    pub title: String,
    pub archived: bool,
}

impl ValueView {
    pub fn project(value: &Value) -> Self {
        Self {
            id: value.id.clone(),
            title: value.title.clone(),
            archived: !value.lifecycle.is_active(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalView {
    pub id: GoalId,
    pub title: String,
    pub achieved: bool,
    pub target_date: Option<NaiveDate>,
    pub archived: bool,
}

impl GoalView {
    pub fn project(goal: &Goal) -> Self {
        Self {
            id: goal.id.clone(),
            title: goal.title.clone(),
            achieved: goal.achievement.is_achieved(),
            target_date: goal.target_date,
            archived: !goal.lifecycle.is_active(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HabitView {
    pub id: HabitId,
    pub title: String,
    pub cadence: Cadence,
    pub strength: HabitStrength,
    pub pinned: bool,
    pub archived: bool,
}

impl HabitView {
    pub fn project(habit: &Habit) -> Self {
        Self {
            id: habit.id.clone(),
            title: habit.title.clone(),
            cadence: habit.cadence,
            strength: habit.strength,
            pinned: habit.pinned,
            archived: !habit.lifecycle.is_active(),
        }
    }
}
