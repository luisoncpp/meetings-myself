use super::error::AppError;
use super::service::PlanningApp;
use planning_core::{Cadence, Habit, HabitId, HabitStrength, Lifecycle, Value, ValueId};

pub struct SetCadence<'a> {
    pub habit: &'a HabitId,
    pub cadence: Cadence,
}

pub struct SetPinned<'a> {
    pub habit: &'a HabitId,
    pub pinned: bool,
}

pub struct SetStrength<'a> {
    pub habit: &'a HabitId,
    pub strength: HabitStrength,
}

impl PlanningApp {
    pub async fn habit(&self, habit: &HabitId) -> Result<Option<Habit>, AppError> {
        self.load_one(HabitId::TABLE, habit.as_str()).await
    }

    pub async fn value(&self, value: &ValueId) -> Result<Option<Value>, AppError> {
        self.load_one(ValueId::TABLE, value.as_str()).await
    }

    pub async fn archive_habit(&self, habit: &HabitId) -> Result<(), AppError> {
        self.set_habit_lifecycle(habit, Lifecycle::Archived).await
    }

    pub async fn restore_habit(&self, habit: &HabitId) -> Result<(), AppError> {
        self.set_habit_lifecycle(habit, Lifecycle::Active).await
    }

    async fn set_habit_lifecycle(&self, habit: &HabitId, to: Lifecycle) -> Result<(), AppError> {
        self.mutate::<Habit>((HabitId::TABLE, habit.to_string()), |found| {
            found.lifecycle = to
        })
        .await?;
        Ok(())
    }

    pub async fn archive_value(&self, value: &ValueId) -> Result<(), AppError> {
        self.set_value_lifecycle(value, Lifecycle::Archived).await
    }

    pub async fn restore_value(&self, value: &ValueId) -> Result<(), AppError> {
        self.set_value_lifecycle(value, Lifecycle::Active).await
    }

    async fn set_value_lifecycle(&self, value: &ValueId, to: Lifecycle) -> Result<(), AppError> {
        self.mutate::<Value>((ValueId::TABLE, value.to_string()), |found| {
            found.lifecycle = to
        })
        .await?;
        Ok(())
    }

    /// Cadence changes apply from the next Daily Plan — nothing here rewrites an
    /// existing plan (ADR 0002).
    pub async fn set_habit_cadence(&self, request: SetCadence<'_>) -> Result<(), AppError> {
        self.mutate::<Habit>((HabitId::TABLE, request.habit.to_string()), |found| {
            found.cadence = request.cadence;
        })
        .await?;
        Ok(())
    }

    pub async fn set_habit_pinned(&self, request: SetPinned<'_>) -> Result<(), AppError> {
        self.mutate::<Habit>((HabitId::TABLE, request.habit.to_string()), |found| {
            found.pinned = request.pinned;
        })
        .await?;
        Ok(())
    }

    pub async fn set_habit_strength(&self, request: SetStrength<'_>) -> Result<(), AppError> {
        self.mutate::<Habit>((HabitId::TABLE, request.habit.to_string()), |found| {
            found.strength = request.strength;
        })
        .await?;
        Ok(())
    }
}
