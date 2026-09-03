use super::commands::{app_error_message, AppState};
use planning_app::{
    Association, AssociationEnd, AssociationId, Cadence, Classification, ClassifyTask, GoalId,
    HabitId, HabitStrength, LinkEnds, NaiveDate, SetCadence, SetDeadline, SetOneOff, SetPinned,
    SetStrength, TaskId,
};

#[tauri::command]
pub async fn archive_entity(
    state: tauri::State<'_, AppState>,
    end: AssociationEnd,
) -> Result<(), String> {
    let app = state.0.lock().await;
    match end {
        AssociationEnd::Task(id) => app.archive_task(&id).await,
        AssociationEnd::Goal(id) => app.archive_goal(&id).await,
        AssociationEnd::Habit(id) => app.archive_habit(&id).await,
        AssociationEnd::Value(id) => app.archive_value(&id).await,
    }
    .map_err(app_error_message)
}

#[tauri::command]
pub async fn restore_entity(
    state: tauri::State<'_, AppState>,
    end: AssociationEnd,
) -> Result<(), String> {
    let app = state.0.lock().await;
    match end {
        AssociationEnd::Task(id) => app.restore_task(&id).await,
        AssociationEnd::Goal(id) => app.restore_goal(&id).await,
        AssociationEnd::Habit(id) => app.restore_habit(&id).await,
        AssociationEnd::Value(id) => app.restore_value(&id).await,
    }
    .map_err(app_error_message)
}

#[tauri::command]
pub async fn complete_task(
    state: tauri::State<'_, AppState>,
    task: String,
    on: Option<NaiveDate>,
) -> Result<(), String> {
    let app = state.0.lock().await;
    let id = TaskId::new(task);
    match on {
        Some(date) => app.complete_task_on(&id, date).await,
        None => app.complete_task(&id).await,
    }
    .map_err(app_error_message)
}

#[tauri::command]
pub async fn reopen_task(state: tauri::State<'_, AppState>, task: String) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .reopen_task(&TaskId::new(task))
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn achieve_goal(state: tauri::State<'_, AppState>, goal: String) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .achieve_goal(&GoalId::new(goal))
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn unachieve_goal(state: tauri::State<'_, AppState>, goal: String) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .unachieve_goal(&GoalId::new(goal))
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn classify_task(
    state: tauri::State<'_, AppState>,
    task: String,
    importance: Classification,
    urgency: Classification,
) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .set_task_classification(ClassifyTask {
            task: &TaskId::new(task),
            importance,
            urgency,
        })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn set_task_deadline(
    state: tauri::State<'_, AppState>,
    task: String,
    deadline: Option<NaiveDate>,
) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .set_task_deadline(SetDeadline {
            task: &TaskId::new(task),
            deadline,
        })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn set_task_one_off(
    state: tauri::State<'_, AppState>,
    task: String,
    one_off: bool,
) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .set_task_one_off(SetOneOff {
            task: &TaskId::new(task),
            one_off,
        })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn set_habit_cadence(
    state: tauri::State<'_, AppState>,
    habit: String,
    cadence: Cadence,
) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .set_habit_cadence(SetCadence {
            habit: &HabitId::new(habit),
            cadence,
        })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn set_habit_pinned(
    state: tauri::State<'_, AppState>,
    habit: String,
    pinned: bool,
) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .set_habit_pinned(SetPinned {
            habit: &HabitId::new(habit),
            pinned,
        })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn set_habit_strength(
    state: tauri::State<'_, AppState>,
    habit: String,
    strength: HabitStrength,
) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .set_habit_strength(SetStrength {
            habit: &HabitId::new(habit),
            strength,
        })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn link(
    state: tauri::State<'_, AppState>,
    left: AssociationEnd,
    right: AssociationEnd,
) -> Result<Association, String> {
    state
        .0
        .lock()
        .await
        .link(LinkEnds { left, right })
        .await
        .map_err(app_error_message)
}

#[tauri::command]
pub async fn unlink(state: tauri::State<'_, AppState>, association: String) -> Result<(), String> {
    state
        .0
        .lock()
        .await
        .unlink(&AssociationId::new(association))
        .await
        .map_err(app_error_message)
}
