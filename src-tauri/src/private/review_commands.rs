use super::commands::{app_error_message, AppState};
use planning_app::{CalendarWeek, SaveReflection, WeeklyReviewView, WeeklySummary};

#[tauri::command]
pub async fn open_weekly_review(
    state: tauri::State<'_, AppState>,
    week: String,
) -> Result<WeeklyReviewView, String> {
    let week = CalendarWeek::parse(&week).map_err(|error| error.to_string())?;
    state
        .0
        .lock()
        .await
        .open_weekly_review(week)
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn open_current_review(
    state: tauri::State<'_, AppState>,
) -> Result<WeeklyReviewView, String> {
    state
        .0
        .lock()
        .await
        .open_current_review()
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn save_reflection(
    state: tauri::State<'_, AppState>,
    week: String,
    reflection: String,
) -> Result<(), String> {
    let week = CalendarWeek::parse(&week).map_err(|error| error.to_string())?;
    state
        .0
        .lock()
        .await
        .save_reflection(SaveReflection { week, reflection })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn weekly_summary(
    state: tauri::State<'_, AppState>,
    week: String,
) -> Result<WeeklySummary, String> {
    let week = CalendarWeek::parse(&week).map_err(|error| error.to_string())?;
    state
        .0
        .lock()
        .await
        .weekly_summary(week)
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn report_path(
    state: tauri::State<'_, AppState>,
    week: String,
) -> Result<String, String> {
    let week = CalendarWeek::parse(&week).map_err(|error| error.to_string())?;
    state
        .0
        .lock()
        .await
        .report_path(week)
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(app_error_message)
}
