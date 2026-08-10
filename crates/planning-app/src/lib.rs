//! The application API. The desktop binary and the launcher depend on this crate
//! and on nothing beneath it.

mod private;

pub use chrono::NaiveDate;
pub use planning_core::{
    Association, AssociationEnd, AssociationId, Cadence, CalendarWeek, CheckInOutcome,
    Classification, Clock, DailyPlan, DailyPlanId, FixedClock, Goal, GoalId, Habit, HabitCheckIn,
    HabitCheckInId, HabitId, HabitStrength, HomeCalendar, Lifecycle, Occurrence, OccurrenceId,
    RecordCheckIn, Recurrence, RecurringTask, RecurringTaskId, StartFocus, StartPlan, SystemClock,
    Task, TaskId, Value, ValueId, WeeklyFocus, WeeklyFocusId,
};
pub use planning_store::{DeviceSettings, DeviceSettingsFile, SetupGap, StoreHealth, UiLanguage};
pub use private::associations::LinkEnds;
pub use private::check_in_use_cases::{CheckInRequest, DateRange};
pub use private::daily_plan_use_cases::{PlanChange, PlanHabitChange, ReorderPlan};
pub use private::entity_lifecycle::{ClassifyTask, SetDeadline, SetOneOff};
pub use private::error::AppError;
pub use private::error_payload::{app_error_payload, AppErrorPayload};
pub use private::habit_lifecycle::{SetCadence, SetPinned, SetStrength};
pub use private::library::{NewGoal, NewHabit};
pub use private::materialization::NewRecurringTask;
pub use private::plan_views::{DailyPlanView, PlanHabitView, PlanTaskView, TaskPoolView};
pub use private::service::{PlanningApp, StartRequest};
pub use private::views::{LibraryFilter, LibraryView, TaskState, TaskView};
pub use private::views_entities::{GoalView, HabitView, ValueView};
pub use private::weekly_focus_use_cases::FocusChange;
pub use private::weekly_review_use_cases::{SaveReflection, WeeklyReviewView};
pub use private::weekly_summary::{HabitSummary, WeeklySummary};
