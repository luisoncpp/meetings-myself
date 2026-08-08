mod private;

use planning_app::{DeviceSettingsFile, PlanningApp, StartRequest, SystemClock};
use private::commands::{app_version, choose_sync_folder, set_home_zone, store_health, AppState};
use private::library_commands::{
    associations_for, create_goal, create_habit, create_task, create_value, library,
};
use private::lifecycle_commands::{
    achieve_goal, archive_entity, classify_task, complete_task, link, reopen_task, restore_entity,
    set_habit_cadence, set_habit_pinned, set_habit_strength, set_task_deadline, unachieve_goal,
    unlink,
};
use std::sync::Arc;

/// Builds and runs the desktop application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings_path = DeviceSettingsFile::default_path().expect("device settings path");
    let app = tauri::async_runtime::block_on(PlanningApp::start(StartRequest {
        settings_path,
        clock: Arc::new(SystemClock),
    }))
    .expect("failed to start planning app");

    tauri::Builder::default()
        .manage(AppState(Arc::new(tokio::sync::Mutex::new(app))))
        .invoke_handler(tauri::generate_handler![
            app_version,
            store_health,
            choose_sync_folder,
            set_home_zone,
            library,
            create_value,
            create_task,
            create_goal,
            create_habit,
            archive_entity,
            restore_entity,
            complete_task,
            reopen_task,
            achieve_goal,
            unachieve_goal,
            classify_task,
            set_task_deadline,
            set_habit_cadence,
            set_habit_pinned,
            set_habit_strength,
            link,
            unlink,
            associations_for,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Self-Planning application");
}
