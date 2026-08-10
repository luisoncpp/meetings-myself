use super::commands::{app_error_message, AppState};
use planning_app::{
    Association, AssociationEnd, Cadence, Goal, Habit, LibraryFilter, LibraryView, NaiveDate,
    NewGoal, NewHabit, Task, Value,
};

#[tauri::command]
pub async fn library(
    state: tauri::State<'_, AppState>,
    include_archived: bool,
) -> Result<LibraryView, String> {
    state
        .0
        .lock()
        .await
        .library(LibraryFilter { include_archived })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn create_value(
    state: tauri::State<'_, AppState>,
    title: String,
) -> Result<Value, String> {
    state
        .0
        .lock()
        .await
        .create_value(title)
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn create_task(
    state: tauri::State<'_, AppState>,
    title: String,
    one_off: Option<bool>,
) -> Result<Task, String> {
    state
        .0
        .lock()
        .await
        .create_task(title, one_off.unwrap_or(/*one_off=*/ true))
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn create_goal(
    state: tauri::State<'_, AppState>,
    title: String,
    target_date: Option<NaiveDate>,
) -> Result<Goal, String> {
    state
        .0
        .lock()
        .await
        .create_goal(NewGoal { title, target_date })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn create_habit(
    state: tauri::State<'_, AppState>,
    title: String,
    cadence: Cadence,
) -> Result<Habit, String> {
    state
        .0
        .lock()
        .await
        .create_habit(NewHabit { title, cadence })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn associations_for(
    state: tauri::State<'_, AppState>,
    end: AssociationEnd,
) -> Result<Vec<Association>, String> {
    state
        .0
        .lock()
        .await
        .associations_for(&end)
        .await
        .map_err(app_error_message)
}
