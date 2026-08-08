//! The application API. The desktop binary and the launcher depend on this crate
//! and on nothing beneath it.

mod private;

pub use chrono::NaiveDate;
pub use planning_core::{
    Association, AssociationEnd, AssociationId, Cadence, CalendarWeek, Classification, Clock,
    FixedClock, Goal, GoalId, Habit, HabitId, HabitStrength, HomeCalendar, Lifecycle,
    Occurrence, OccurrenceId, Recurrence, RecurringTask, RecurringTaskId, SystemClock, Task,
    TaskId, Value, ValueId,
};
pub use planning_store::{DeviceSettings, DeviceSettingsFile, SetupGap, StoreHealth};
pub use private::associations::LinkEnds;
pub use private::entity_lifecycle::{ClassifyTask, SetDeadline};
pub use private::error::AppError;
pub use private::habit_lifecycle::{SetCadence, SetPinned, SetStrength};
pub use private::library::{NewGoal, NewHabit};
pub use private::materialization::NewRecurringTask;
pub use private::service::{PlanningApp, StartRequest};
pub use private::views::{LibraryFilter, LibraryView, TaskState, TaskView};
pub use private::views_entities::{GoalView, HabitView, ValueView};
