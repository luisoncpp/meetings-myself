use super::commands::{app_error_message, AppState};
use planning_app::{
    CalendarWeek, CheckInOutcome, CheckInRequest, DailyPlanView, FocusChange, HabitId, NaiveDate,
    NewRecurringTask, PlanChange, PlanHabitChange, Recurrence, RecurringTask, RecurringTaskId,
    ReorderPlan, Task, TaskId, TaskPoolView, WeeklyFocus,
};

#[tauri::command]
pub async fn today_view(state: tauri::State<'_, AppState>) -> Result<DailyPlanView, String> {
    state
        .0
        .lock()
        .await
        .today_view()
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn task_pool(state: tauri::State<'_, AppState>) -> Result<TaskPoolView, String> {
    state
        .0
        .lock()
        .await
        .task_pool()
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn select_into_plan(
    state: tauri::State<'_, AppState>,
    date: NaiveDate,
    task: String,
) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .select_into_plan(PlanChange {
            date,
            task: TaskId::new(task),
        })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn remove_from_plan(
    state: tauri::State<'_, AppState>,
    date: NaiveDate,
    task: String,
) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .remove_from_plan(PlanChange {
            date,
            task: TaskId::new(task),
        })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn reorder_plan(
    state: tauri::State<'_, AppState>,
    date: NaiveDate,
    order: Vec<String>,
) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .reorder_plan(ReorderPlan {
            date,
            order: order.into_iter().map(TaskId::new).collect(),
        })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn add_habit_to_plan(
    state: tauri::State<'_, AppState>,
    date: NaiveDate,
    habit: String,
) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .add_habit_to_plan(PlanHabitChange {
            date,
            habit: HabitId::new(habit),
        })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn quick_add_task(
    state: tauri::State<'_, AppState>,
    title: String,
) -> Result<Task, String> {
    state
        .0
        .lock()
        .await
        .quick_add_task(title)
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn record_check_in(
    state: tauri::State<'_, AppState>,
    habit: String,
    date: NaiveDate,
    outcome: CheckInOutcome,
) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .record_check_in(CheckInRequest {
            habit: HabitId::new(habit),
            date,
            outcome,
        })
        .await
        .map(|_| ())
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn weekly_focus(
    state: tauri::State<'_, AppState>,
    week: String,
) -> Result<WeeklyFocus, String> {
    let week = CalendarWeek::parse(&week).map_err(|error| error.to_string())?;
    state
        .0
        .lock()
        .await
        .weekly_focus(week)
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn add_to_focus(
    state: tauri::State<'_, AppState>,
    week: String,
    task: String,
) -> Result<(), String> {
    let week = CalendarWeek::parse(&week).map_err(|error| error.to_string())?;
    state
        .0
        .lock()
        .await
        .add_to_focus(FocusChange {
            week,
            task: TaskId::new(task),
        })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn remove_from_focus(
    state: tauri::State<'_, AppState>,
    week: String,
    task: String,
) -> Result<(), String> {
    let week = CalendarWeek::parse(&week).map_err(|error| error.to_string())?;
    state
        .0
        .lock()
        .await
        .remove_from_focus(FocusChange {
            week,
            task: TaskId::new(task),
        })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn create_recurring_task(
    state: tauri::State<'_, AppState>,
    title: String,
    recurrence: Recurrence,
) -> Result<RecurringTask, String> {
    state
        .0
        .lock()
        .await
        .create_recurring_task(NewRecurringTask { title, recurrence })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn recurring_tasks(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<RecurringTask>, String> {
    state
        .0
        .lock()
        .await
        .recurring_tasks()
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn archive_recurring_task(
    state: tauri::State<'_, AppState>,
    rule: String,
) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .archive_recurring_task(&RecurringTaskId::new(rule))
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn restore_recurring_task(
    state: tauri::State<'_, AppState>,
    rule: String,
) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .restore_recurring_task(&RecurringTaskId::new(rule))
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn rename_recurring_task(
    state: tauri::State<'_, AppState>,
    rule: String,
    title: String,
) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .rename_recurring_task(&RecurringTaskId::new(rule), title)
        .await
        .map_err(app_error_message)
}
